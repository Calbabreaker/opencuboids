use std::{
    io::{Read, Write},
    net::TcpStream,
};

use bevy_utils::Duration;

const CONNECT_TRIES: u32 = 5;

pub fn connect(ip: &String, port: u16) {
    let address = format!("{}:{}", ip, port);
    if let Err(err) = try_connect(&address, CONNECT_TRIES) {
        log::error!("Failed to connect to {} - {}", address, err);
    }
}

fn try_connect(address: &String, tries: u32) -> std::io::Result<()> {
    match TcpStream::connect(address) {
        Err(err) => {
            if tries <= 0 {
                Err(err)?
            } else {
                std::thread::sleep(Duration::from_secs(1));
                try_connect(address, tries - 1)?
            }
        }
        Ok(stream) => {
            log::info!("Sucessfully connected to {}", address);
            handle_stream(stream)?;
        }
    }

    Ok(())
}

fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let msg = "Hello, world!";
    stream.write(msg.as_bytes())?;
    log::info!("Sending: {}", msg);

    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    log::info!("Got reply: {}", String::from_utf8_lossy(&buffer));
    Ok(())
}
