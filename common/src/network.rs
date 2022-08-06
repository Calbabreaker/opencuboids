use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

use crate::Chunk;

pub type ErrorKind = bincode::ErrorKind;
pub type Result<T> = bincode::Result<T>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ChunkRange {
        start: glam::IVec3,
        end: glam::IVec3,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    ChunkData(Chunk),
    Test,
}

pub struct Protocol {
    reader: std::io::BufReader<TcpStream>,
    pub stream: TcpStream,
}

impl Protocol {
    pub fn with_stream(stream: TcpStream) -> Result<Self> {
        Ok(Self {
            reader: std::io::BufReader::new(stream.try_clone()?),
            stream,
        })
    }

    /// Tries to connect to address n number of tries waiting 1 second in between
    pub fn connect(address: SocketAddr, tries: u8) -> Result<Self> {
        match TcpStream::connect(address) {
            Err(err) => {
                if tries <= 0 {
                    Err(err)?
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Protocol::connect(address, tries - 1)
                }
            }
            Ok(stream) => Ok(Self::with_stream(stream)?),
        }
    }

    pub fn send(&mut self, data: &impl Serialize) -> Result<()> {
        bincode::serialize_into(&mut self.stream, data)?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn read<T: DeserializeOwned>(&mut self) -> Result<T> {
        bincode::deserialize_from(&mut self.reader)
    }
}
