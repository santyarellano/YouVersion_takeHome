use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use crate::service::VotdService;

/// Minimal standard library HTTP server using `std::net::TcpListener`.
///
/// Synchronous TCP socket listening and standard HTTP/1.1 response formatting.
pub struct SimpleHttpServer {
    /// Shared service instance dispatching parsed HTTP requests.
    service: Arc<VotdService>,
    /// Native TCP socket listener bound to the host and port.
    listener: TcpListener,
}

impl SimpleHttpServer {
    /// Binds the server to the specified address string (e.g., `"127.0.0.1:3000"` or `"127.0.0.1:0"`).
    ///
    /// # Errors
    /// Returns [`std::io::Error`] if binding the TCP socket fails (e.g. port already in use).
    pub fn bind(addr: &str, service: Arc<VotdService>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self { service, listener })
    }

    /// Returns the local socket address this server is listening on.
    ///
    /// Particularly useful in tests when bound to port `0` for dynamic ephemeral port assignment.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Reads an incoming HTTP/1.1 request from a client `TcpStream`, executes the service handler,
    /// and writes the formatted HTTP/1.1 response back through the network connection.
    ///
    /// Closes the connection upon completion (`Connection: close`).
    pub fn handle_client(service: &VotdService, mut stream: TcpStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line)? == 0 {
            return Ok(());
        }

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(());
        }

        let method = parts[0];
        let uri = parts[1];

        // Read headers until the empty line
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
                break;
            }
        }

        let response = service.handle_request(method, uri);

        let status_text = match response.status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            _ => "Internal Server Error",
        };

        let raw_response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.status,
            status_text,
            response.body.len(),
            response.body
        );

        stream.write_all(raw_response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    /// Enters the server accept loop, spawning a background worker thread for each incoming connection.
    pub fn run(&self) -> std::io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let service = Arc::clone(&self.service);
                    std::thread::spawn(move || {
                        let _ = Self::handle_client(&service, stream);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}
