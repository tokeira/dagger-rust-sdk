//! The loopback transport's TCP keepalive on Linux, where the kernel enforces it.
//!
//! A module's request can wait minutes for the engine without a byte crossing its
//! connection, so a connection that gives up early loses the engine's answer. Reqwest's
//! default `TCP_USER_TIMEOUT` of 30 s did that: with it, Linux resets a connection after one
//! unanswered keepalive probe. These tests read the options back from a live socket and
//! keep a request waiting past that timeout.

use std::fs;
use std::os::fd::{BorrowedFd, RawFd};
use std::sync::Arc;
use std::time::Duration;

use rustix::net::sockopt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};

use crate::connection::EngineConnection;
use crate::graphql::{RawRequest, ResponseData};
use crate::preflight::PropagationEnvironment;
use crate::session::SecretString;
use crate::transport::{
    LoopbackTransportFactory, ReqwestLoopbackConnection, ReqwestLoopbackFactory,
};

/// The fixture's answer to any request.
fn response() -> String {
    let body = r#"{"data":{"version":"v0.0.0"}}"#;
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn loopback(port: u16) -> ReqwestLoopbackConnection {
    let token =
        SecretString::new(Arc::<str>::from("KEEPALIVE_SECRET_0123456789")).expect("non-empty");
    ReqwestLoopbackFactory
        .loopback(
            port,
            token,
            Duration::from_secs(1),
            PropagationEnvironment::for_test(None, None, None),
            None,
        )
        .expect("fixed loopback client construction succeeds")
}

/// Reads one request, headers and `Content-Length` body, from `stream`.
async fn read_request(stream: &mut TcpStream) {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = stream.read(&mut buffer).await.expect("fixture read");
        assert_ne!(count, 0, "the client closed before sending a whole request");
        bytes.extend_from_slice(&buffer[..count]);
        let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let length = String::from_utf8_lossy(&bytes[..end])
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .expect("the client sends a Content-Length");
        if bytes.len() >= end + 4 + length {
            return;
        }
    }
}

/// The keepalive options of one socket, as the kernel reports them.
#[derive(Debug, Eq, PartialEq)]
struct Keepalive {
    enabled: bool,
    idle: Duration,
    interval: Duration,
    probes: u32,
    user_timeout_ms: u32,
}

/// Reads the keepalive options of the socket that this process holds open as `fd`.
// Reqwest exposes neither its sockets nor their options, and `pidfd_getfd`, which would
// duplicate the descriptor safely, needs a capability that container runtimes withhold. So
// this one function borrows the descriptor by number.
#[allow(unsafe_code)]
fn keepalive(fd: RawFd) -> Keepalive {
    // SAFETY: `fd` is the client's pooled connection to a fixture that keeps its own end
    // open, and the client stays alive and idle while this function runs, so nothing
    // closes the descriptor or reuses its number before the borrow ends with the function.
    // The borrow only reads socket options.
    let socket = unsafe { BorrowedFd::borrow_raw(fd) };
    Keepalive {
        enabled: sockopt::socket_keepalive(socket).expect("the kernel reports SO_KEEPALIVE"),
        idle: sockopt::tcp_keepidle(socket).expect("the kernel reports TCP_KEEPIDLE"),
        interval: sockopt::tcp_keepintvl(socket).expect("the kernel reports TCP_KEEPINTVL"),
        probes: sockopt::tcp_keepcnt(socket).expect("the kernel reports TCP_KEEPCNT"),
        user_timeout_ms: sockopt::tcp_user_timeout(socket)
            .expect("the kernel reports TCP_USER_TIMEOUT"),
    }
}

/// The descriptor this process holds for its IPv4 loopback connection from port `local`
/// to port `remote`, found through the socket's inode.
fn loopback_socket(local: u16, remote: u16) -> RawFd {
    let local = format!("0100007F:{local:04X}");
    let remote = format!("0100007F:{remote:04X}");
    let table = fs::read_to_string("/proc/self/net/tcp").expect("the IPv4 socket table");
    let inode = table
        .lines()
        .skip(1)
        .find_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            (fields.get(1) == Some(&local.as_str()) && fields.get(2) == Some(&remote.as_str()))
                .then(|| fields.get(9).map(|inode| (*inode).to_owned()))
                .flatten()
        })
        .expect("the socket table lists the client's connection");
    let link = format!("socket:[{inode}]");
    fs::read_dir("/proc/self/fd")
        .expect("this process's descriptors")
        .filter_map(Result::ok)
        .find(|entry| {
            fs::read_link(entry.path()).is_ok_and(|target| target.as_os_str() == link.as_str())
        })
        .and_then(|entry| entry.file_name().to_str()?.parse().ok())
        .expect("this process holds the client's connection")
}

// The pooled connection that serves a request keeps Go's keepalive and no user timeout,
// which would override the probe count.
#[tokio::test]
async fn loopback_connection_keeps_the_go_sdk_keepalive_without_a_user_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("fixture binds");
    let port = listener.local_addr().expect("fixture address").port();
    let server = tokio::spawn(async move {
        let (mut stream, client) = listener.accept().await.expect("request connects");
        read_request(&mut stream).await;
        stream
            .write_all(response().as_bytes())
            .await
            .expect("response writes");
        // The stream goes back to the test, which holds it open while it reads the
        // client's socket.
        (stream, client.port())
    });

    let connection = loopback(port);
    connection
        .execute(RawRequest::new("query { version }"))
        .await
        .expect("the fixture answers");
    let (_stream, client_port) = server.await.expect("fixture joins");

    assert_eq!(
        keepalive(loopback_socket(client_port, port)),
        Keepalive {
            enabled: true,
            idle: Duration::from_secs(15),
            interval: Duration::from_secs(15),
            probes: 9,
            user_timeout_ms: 0,
        }
    );
}

// A request waits for as long as its peer keeps it waiting. The fixture accepts a few
// kilobytes and then reads nothing, so the client's kernel holds the rest of the body and
// probes the closed window, which the fixture's kernel answers. Linux resets a connection
// with a user timeout once the window has stayed closed that long, whatever the answers,
// so the request must still be pending after Reqwest's default of 30 s.
#[tokio::test]
async fn a_request_outlasts_a_closed_receive_window() {
    const PAST_THE_USER_TIMEOUT: Duration = Duration::from_secs(35);

    let socket = TcpSocket::new_v4().expect("fixture socket");
    // Accepted connections inherit the small buffer, which caps the receive window.
    socket
        .set_recv_buffer_size(16 * 1024)
        .expect("fixture buffer size");
    socket
        .bind(([127, 0, 0, 1], 0).into())
        .expect("fixture binds");
    let listener = socket.listen(1).expect("fixture listens");
    let port = listener.local_addr().expect("fixture address").port();
    let (open, opened) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("request connects");
        opened.await.expect("the test opens the window");
        read_request(&mut stream).await;
        stream
            .write_all(response().as_bytes())
            .await
            .expect("response writes");
        stream
    });

    let connection = loopback(port);
    let body = format!("query {{ version }} # {}", "x".repeat(1024 * 1024));
    let mut request = tokio::spawn(async move { connection.execute(RawRequest::new(body)).await });
    if let Ok(ended) = tokio::time::timeout(PAST_THE_USER_TIMEOUT, &mut request).await {
        panic!("the request ended while the window was closed: {ended:?}");
    }

    open.send(()).expect("the fixture waits");
    let response = request
        .await
        .expect("the request task joins")
        .expect("the request completes once the window opens");
    assert_eq!(
        response.data(),
        &ResponseData::Value(serde_json::json!({ "version": "v0.0.0" }))
    );
    server.await.expect("fixture joins");
}
