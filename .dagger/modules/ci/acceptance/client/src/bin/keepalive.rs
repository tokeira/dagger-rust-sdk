//! Keepalive acceptance: a query outlasts a lost reply to a TCP keepalive probe.
//!
//! The acceptance runs this under `dagger run` with insecure root capabilities, so it can filter
//! packets in its network namespace, which the session it reaches shares. While its query waits
//! 45 s for the engine, it drops the session's bare acknowledgements to it for a while, which
//! spans its first keepalive probe, 15 s into the silence. Linux resets a connection with a user
//! timeout, such as Reqwest's default of 30 s, once that timeout passes with a probe unanswered.

use std::error::Error;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// When the drop starts, measured from the query's start: its request and the session's
/// acknowledgement of it pass first.
const DROP_AFTER: Duration = Duration::from_secs(2);
/// How long the drop lasts: it spans the first keepalive probe, 15 s into the silence, and
/// ends before the second, 30 s in.
const DROP_FOR: Duration = Duration::from_secs(20);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = dagger_sdk::connect().await?;
    let query = client.query();
    let version = query.version().await?;
    // The session that `dagger run` serves this process on.
    let port = std::env::var("DAGGER_SESSION_PORT")?;

    // The value no other run used keeps the engine from answering from its cache.
    let run = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let sleeper = query
        .container()
        .from("alpine:3.22")
        .with_env_variable("RUST_KEEPALIVE_RUN", run.to_string())
        .with_exec(vec!["sleep", "45"]);
    let (held, counts) = tokio::join!(sleeper.sync(), drop_replies(&port));
    let (probes, dropped) = counts?;
    println!("ACCEPTANCE-KEEPALIVE-COUNTS {probes} probes, {dropped} replies dropped");
    held?;
    // Without a probe whose reply was dropped, the query proved nothing.
    if probes == 0 || dropped == 0 {
        return Err("no keepalive probe went unanswered".into());
    }
    client.close().await?;
    println!("ACCEPTANCE-KEEPALIVE-OK {version}");
    Ok(())
}

/// Drops the session's bare acknowledgements to this process for `DROP_FOR`, from `DROP_AFTER`
/// on, and returns how many keepalive probes this process sent meanwhile and how many replies
/// were dropped.
async fn drop_replies(port: &str) -> Result<(u64, u64), Box<dyn Error>> {
    tokio::time::sleep(DROP_AFTER).await;
    rules("-I", port)?;
    tokio::time::sleep(DROP_FOR).await;
    let listing = iptables(&["-L", "OUTPUT", "-v", "-x", "-n"]);
    rules("-D", port)?;
    let listing = listing?;
    Ok((
        packets(&listing, &format!("dpt:{port} ")),
        packets(&listing, &format!("spt:{port} ")),
    ))
}

/// Inserts (`-I`) or deletes (`-D`) two rules on loopback. One counts this process's bare
/// acknowledgements to the session, which are its keepalive probes; the other drops the
/// session's.
fn rules(action: &str, port: &str) -> Result<(), Box<dyn Error>> {
    for (direction, target) in [("--dport", None), ("--sport", Some("DROP"))] {
        let mut args = vec![
            action,
            "OUTPUT",
            "-o",
            "lo",
            "-p",
            "tcp",
            direction,
            port,
            "--tcp-flags",
            "ALL",
            "ACK",
        ];
        if let Some(target) = target {
            args.extend(["-j", target]);
        }
        iptables(&args)?;
    }
    Ok(())
}

/// Runs iptables with `args` and returns what it prints.
fn iptables(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("iptables").args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "iptables {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// The packet count of the rule that names `field` in an `iptables -L -v -x -n` listing.
fn packets(listing: &str, field: &str) -> u64 {
    listing
        .lines()
        .find(|line| line.contains(field))
        .and_then(|line| line.split_whitespace().next()?.parse().ok())
        .unwrap_or(0)
}
