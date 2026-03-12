// Copyright (c) 2026 Sebastian Ibanez

//! Custom communication protocol for mem-store-rs.
//! Written on top of TCP.

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

    /*
        PROTOCOL SPECIFICATION

        1. Message starts with 1 byte flag for type:
            [ as \x91 => Request
            ] as \x93 => Response

        2. Then 1 byte flag for response/request type:
            < as \x60 => Get request
            > as \x62 => Set request
            - as \x43 => Delete request

            $ as \x36 => Ok response
            & as \x38 => Error response

        3. 2 1 byte flags to specify which body fields are included:
            true/false => Key field
            true/false  => Value field

            Note: Not need for error responses.
            Error responses do not send the key or value fields,
            there is no ambiguity what the serialized field is.

        4. Body fields:
            Each body field is prepended with their size.
            Example:
            5hello

            Body field sizes:
            key => u16 (2 bytes)
            value => u32 (8 bytes)
            error => u16 (2 bytes)

            Example GET "hello" key:
            Request => [<5hello
            Response => ]$5world
    */

    fn serialize(&self) -> Result<Vec<u8>, Error> {
        let pkt_type_tags = self.packet_type.to_tag();
        let mut buf = Vec::new();
        buf.extend(pkt_type_tags);

        // Error response
        if let PacketType::ResponseError(error) = self.packet_type {
            buf.extend(self.packet_type.to_tag());
            let error = error.to_string();
            assert!(error.len() <= usize::from(u16::MAX));
            buf.extend(&(error.len() as u16).to_be_bytes());
            buf.extend(error.into_bytes());
            return Ok(buf);
        }

        let (key_include, value_include) = match (self.key.clone(), self.value.clone()) {
            (None, None) => (false, false),
            (None, Some(_)) => (false, true),
            (Some(_), None) => (true, false),
            (Some(_), Some(_)) => (true, true),
        };
        buf.push(key_include as u8);
        buf.push(value_include as u8);

        // Key field
        if let Some(key) = self.key.clone() {
            let key_bytes = key.as_bytes();
            assert!(key_bytes.len() <= usize::from(u16::MAX));
            buf.extend(&(key_bytes.len() as u16).to_be_bytes());
            buf.extend(key_bytes)
        }

        // Value field
        if let Some(value) = self.value.clone() {
            let value_str = value.to_string();
            let value_bytes = value_str.as_bytes();
            // Make sure value length fits in a u32.
            let len = u32::try_from(value_bytes.len()).map_err(|_| Error::ValueLengthTooLong)?;
            assert!(len <= u32::MAX);
            buf.extend(&(value_bytes.len() as u32).to_be_bytes());
            buf.extend(value_bytes)
        }
        Ok(buf)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Packet, Error> {
        todo!()
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
    pub fn to_tag(&self) -> [u8; 2] {
        match self {
            PacketType::RequestGet => ['[' as u8, '+' as u8],
            PacketType::RequestSet => ['[' as u8, '>' as u8],
            PacketType::RequestDelete => ['[' as u8, '-' as u8],
            PacketType::ResponseOk => [']' as u8, '$' as u8],
            PacketType::ResponseError(_) => [']' as u8, '&' as u8],
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
    let serial_pkt = packet.serialize()?;
    let len = (serial_pkt.len() as u32).to_be_bytes();
    stream
        .write_all(&len)
        .await
        .map_err(|_| Error::UnableToSend)?;
    stream
        .write_all(&serial_pkt)
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

    Packet::deserialize(&buf)
}
