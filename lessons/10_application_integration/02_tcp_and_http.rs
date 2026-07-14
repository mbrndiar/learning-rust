//! Lesson 10.2: a minimal localhost HTTP exchange over TCP.
//!
//! This intentionally handles one small, known request. Use an HTTP library for
//! production protocol parsing, limits, timeouts, TLS, and connection reuse.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn serve_once(listener: TcpListener) -> io::Result<()> {
    let (mut stream, _) = listener.accept()?;
    let mut request = [0_u8; 1_024];
    let bytes_read = stream.read(&mut request)?;
    let request = String::from_utf8_lossy(&request[..bytes_read]);
    let first_line = request.lines().next().unwrap_or_default();

    let (status, body) = if first_line == "GET /health HTTP/1.1" {
        ("200 OK", r#"{"status":"ok"}"#)
    } else {
        ("404 Not Found", r#"{"error":"not found"}"#)
    };

    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

fn request_health(address: std::net::SocketAddr) -> io::Result<String> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || serve_once(listener));

    let response = request_health(address)?;
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing HTTP separator"))?;
    let status = headers.lines().next().unwrap_or_default();
    println!("status={status}");
    println!("body={body}");

    server
        .join()
        .map_err(|_| io::Error::other("server thread panicked"))??;
    Ok(())
}
