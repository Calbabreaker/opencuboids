mod world_gen;

use std::net::{SocketAddr, TcpListener, TcpStream};

use opencuboids_common::{iter_3d_vec, network, Chunk};

pub fn start(address: SocketAddr) {
    if let Err(err) = bind(address) {
        log::error!("Failed to start server on {} - {}", address, err);
    }
}

fn bind(address: SocketAddr) -> std::io::Result<()> {
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

fn handle_client(stream: TcpStream) -> network::Result<()> {
    use network::{Request, Response};

    let mut protocol = network::Protocol::with_stream(stream)?;
    loop {
        let request = protocol.read::<network::Request>()?;
        log::info!("Received message: {:#?}", request);

        match request {
            Request::ChunkRange { start, end } => {
                for chunk_pos in iter_3d_vec(start, end) {
                    let mut chunk = Chunk::new(chunk_pos);
                    world_gen::gen_blocks(&mut chunk, chunk_pos);
                    let response = Response::ChunkData(chunk);
                    protocol.send(&response)?
                }
            }
        }
    }
}
