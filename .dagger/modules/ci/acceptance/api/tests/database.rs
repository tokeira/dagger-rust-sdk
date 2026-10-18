//! Integration test that runs only where CI provides `DATABASE_URL`.

use std::net::TcpStream;

#[test]
fn database_accepts_connections() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let authority = url
        .split('@')
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .expect("DATABASE_URL names a host and port");
    println!("connecting to {authority}");
    TcpStream::connect(authority).expect("the database accepts TCP connections");
}
