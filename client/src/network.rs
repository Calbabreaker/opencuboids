use bevy_ecs::prelude::*;
use std::net::SocketAddr;

use bevy_utils::Duration;
use crossbeam_channel::{Receiver, Sender};
use opencuboids_common::network;

use crate::world::ChunkManager;

pub struct StreamChannel {
    pub sender: Sender<network::Request>,
    pub receiver: Receiver<network::Response>,
}

pub fn connect(address: SocketAddr) -> StreamChannel {
    log::info!("Connecting to {}", address);
    let (request_tx, request_rx) = crossbeam_channel::unbounded();
    let (response_tx, response_rx) = crossbeam_channel::unbounded();

    std::thread::spawn(move || match network::Protocol::connect(address, 5) {
        Ok(client) => {
            if let Err(err) = handle_client(client, response_tx, request_rx) {
                log::error!("Error with connection, disconnecting - {}", err);
            }
        }
        Err(err) => log::error!("Failed to connect to {} - {}", address, err),
    });

    StreamChannel {
        sender: request_tx,
        receiver: response_rx,
    }
}

fn handle_client(
    mut protocol: network::Protocol,
    sender: Sender<network::Response>,
    receiver: Receiver<network::Request>,
) -> network::Result<()> {
    protocol
        .stream
        .set_read_timeout(Some(Duration::from_millis(100)))?;

    loop {
        for request in receiver.try_iter() {
            protocol.send(&request)?;
        }

        match protocol.read::<network::Response>() {
            Ok(response) => sender.send(response).unwrap(),
            Err(err) => match *err {
                network::ErrorKind::Io(ref e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => (),
                    _ => Err(err)?,
                },
                _ => Err(err)?,
            },
        }
    }
}

pub fn handle_responses(channel: ResMut<StreamChannel>, mut chunk_manager: ResMut<ChunkManager>) {
    for response in channel.receiver.try_iter() {
        match response {
            network::Response::ChunkData(chunks) => {
                chunk_manager.handle_chunk_response(chunks);
            }
            _ => (),
        }
    }
}
