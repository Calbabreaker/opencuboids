mod world_gen;

use std::net::{SocketAddr, TcpListener, TcpStream};

use opencuboids_common::{loop_3d_vec, network, Chunk};

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
    // loop {
    //     let request = protocol.read::<network::Request>()?;
    //     log::info!("Received message: {:#?}", request);

    //     match request {
    //         Request::ChunkRange { start, end } => {
    let start = glam::ivec3(0, 0, 0);
    let end = glam::ivec3(3, 3, 3);
    let mut chunks = Vec::new();
    loop_3d_vec!(start, end, |chunk_pos| {
        let mut chunk = Chunk::new(chunk_pos);
        world_gen::gen_blocks(&mut chunk, chunk_pos);
        chunks.push(chunk);
    });

    let res = &Response::ChunkData(chunks);
    protocol.send(res)
    // }
    // }
    // }
}
