// Copyright (c) 2026 Sebastian Ibanez

//! Custom communication protocol for mem-store-rs.
//! Written on top of TCP.

use postcard::{from_bytes, to_allocvec};
use serde::{self, Deserialize, Serialize};
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::{error::Error, store::Item};

pub const PKT_SIZE: usize = size_of::<Packet>();

/// mem-store-rs protocol packet structure.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Packet {
    pub key: Option<String>,
    pub value: Option<Item>,
    pub packet_type: PacketType,
}

impl Packet {
    pub fn ok_response() -> Self {
        Self {
            key: None,
            value: None,
            packet_type: PacketType::ResponseOk,
        }
    }

    pub fn error_response(error: Error) -> Self {
        Self {
            key: None,
            value: None,
            packet_type: PacketType::ResponseError(error),
        }
    }

    pub fn get_request(key: String) -> Self {
        Self {
            key: Some(key),
            value: None,
            packet_type: PacketType::RequestGet,
        }
    }

    pub fn set_request(key: String, value: Item) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            packet_type: PacketType::RequestSet,
        }
    }

    pub fn delete_request(key: String) -> Self {
        Self {
            key: Some(key),
            value: None,
            packet_type: PacketType::RequestSet,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PacketType {
    RequestGet,
    RequestSet,
    RequestDelete,
    ResponseOk,
    ResponseError(Error),
}

impl Default for PacketType {
    fn default() -> Self {
        PacketType::ResponseOk
    }
}

/// Send a Packet across a `&mut TcpStream`.
pub async fn send_packet(stream: &mut TcpStream, packet: &Packet) -> Result<(), Error> {
    let payload = to_allocvec(packet).map_err(|_| Error::UnableToSerialize)?;
    let len = (payload.len() as u32).to_be_bytes();
    stream
        .write_all(&len)
        .await
        .map_err(|_| Error::UnableToSend)?;
    stream
        .write_all(&payload)
        .await
        .map_err(|_| Error::UnableToSend)?;
    Ok(())
}

/// Receive a Packet from an `&mut TcpStream`.
pub async fn recv_packet(stream: &mut TcpStream) -> Result<Packet, Error> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|_| Error::UnableToReceive)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|_| Error::UnableToReceive)?;
    from_bytes(&buf).map_err(|_| Error::UnableToDeserialize)
}
