// Copyright (c) 2026 Sebastian Ibanez

//! Custom communication protocol for mem-store-rs.
//! Written on top of TCP.

use postcard::{from_bytes, to_allocvec};
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::{error::Error, store::Item};

pub const PKT_SIZE: usize = size_of::<Packet>();

pub trait Serialize {
    fn serialize(&self) -> String;
}

/// mem-store-rs protocol packet structure.
#[derive(Default, Clone)]
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

    // fn serialize(&self) -> Vec<u8> {
    //     match self.packet_type {
    //         PacketType::RequestGet => self.serialize_key('+'),
    //         PacketType::RequestSet => self.serialize_key_and_value('>'),
    //         PacketType::RequestDelete => self.serialize_key('-'),
    //         PacketType::ResponseOk => self.serialize_key_and_value('$'),
    //         PacketType::ResponseError(error) => format!("&{}", error.to_string()),
    //     }
    // }

    fn serialize(&self, symbol: char) -> Vec<u8> {
        let mut buf = vec![symbol as u8];
        if let Some(key) = self.key.clone() {
            let key_bytes = key.as_bytes();
            buf.extend(&(key_bytes.len() as u16).to_be_bytes());
            buf.extend(key_bytes)
        } else {
            buf.extend(&0u16.to_be_bytes());
        }
        if let Some(value) = self.value.clone() {
            let key_bytes = Item::as_bytes(&value);
            buf.extend(&(key_bytes.len() as u16).to_be_bytes());
            buf.extend(key_bytes)
        } else {
            buf.extend(&0u16.to_be_bytes());
        }
        buf
    }

    pub fn deserialize(bytes: &[u8]) -> Packet {
        let serial_data = String::from_utf8_lossy(bytes);
        let packet_str = serial_data.as_ref().to_string();
        let pkt_type = match packet_str.get(0..1) {
            Some("+") => PacketType::RequestGet,
            Some(">") => PacketType::RequestSet,
            Some("-") => PacketType::RequestDelete,
            Some("$") => PacketType::ResponseOk,
            Some("&") => todo!(),
            _ => todo!(),
        };
        // TODO: somehow split packet_str at the key and value
        // maybe have a delimiter for:
        // - Some key
        // - No key
        // - Some value
        // - No value
        //
        // ^ Feels wrong. No way of knowing if value delimiter is part of key or not.
        // HOW DO PEOPLE SERIALIZE/DESERIALIZE PACKETS!?
    }
}

#[derive(Clone)]
pub enum PacketType {
    RequestGet,
    RequestSet,
    RequestDelete,
    ResponseOk,
    ResponseError(Error),
}

impl PacketType {
    pub fn to_tag(&self) -> u8 {
        match self {
            PacketType::RequestGet => '+' as u8,
            PacketType::RequestSet => '>' as u8,
            PacketType::RequestDelete => '-' as u8,
            PacketType::ResponseOk => '$' as u8,
            PacketType::ResponseError(_) => '&' as u8,
        }
    }
}

impl Default for PacketType {
    fn default() -> Self {
        PacketType::ResponseOk
    }
}

/// Send a Packet across a `&mut TcpStream`.
pub async fn send_packet(stream: &mut TcpStream, packet: &Packet) -> Result<(), Error> {
    let serial_pkt = packet.serialize();
    let payload = serial_pkt.as_bytes();
    // let payload = to_allocvec(packet).map_err(|_| Error::UnableToSerialize)?;
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
