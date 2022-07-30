use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use opencuboids_common::DEFAULT_PORT;

pub fn start(port: Option<u16>) {
    let port = port.unwrap_or(DEFAULT_PORT);
    if let Err(err) = bind(("0.0.0.0", port)) {
        log::error!("Failed to start server on port {} - {}", port, err);
    }
}

fn bind<A: ToSocketAddrs>(address: A) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    log::info!("Server running on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        let stream = stream?;
        let addr = stream.peer_addr()?;
        log::info!("Client connected at {}", addr);
        std::thread::spawn(move || {
            if handle_client(stream).is_err() {
                log::info!("Client disconnected unexpectedly at {}", addr);
            } else {
                log::info!("Client disconnected at {}", addr);
            }
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let size = stream.read(&mut buffer)?;
        if size < 1 {
            break;
        }

        log::info!("Received message: {}", String::from_utf8_lossy(&buffer));
        stream.write(&buffer[0..size])?;
        buffer = [0; 1024];
    }

    Ok(())
}
