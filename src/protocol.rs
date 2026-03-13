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
            packet_type: PacketType::ResponseError(String::from(error.to_string())),
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

        1. Then 1 byte flag for response/request type:
            < as \x60 => Get request
            > as \x62 => Set request
            - as \x43 => Delete request

            $ as \x36 => Ok response
            & as \x38 => Error response

        2. 2 1 byte flags to specify which body fields are included:
            true/false => Key field
            true/false  => Value field

            Note: Not needed for error responses.
            Error responses do not send the key or value fields,
            there is no ambiguity about what the body field is
            (an error message prepended with it's u16 length).

        3. Body fields:
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
        let mut buf = Vec::new();
        buf.push(self.packet_type.to_tag());

        // Error response
        if let PacketType::ResponseError(error) = self.packet_type.clone() {
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
        let mut bytes_iter = bytes.iter();
        let pkt_type = match bytes_iter.next() {
            Some(0x60) => PacketType::RequestGet,
            Some(0x62) => PacketType::RequestSet,
            Some(0x43) => PacketType::RequestDelete,
            Some(0x36) => PacketType::ResponseOk,
            Some(0x38) => {
                let mut len_bytes = Vec::new();
                for _ in 0..2 {
                    if let Some(byte) = bytes_iter.next() {
                        len_bytes.push(*byte);
                    } else {
                        eprintln!("error: packet ended while reading error message length");
                        return Err(Error::UnableToDeserialize);
                    }
                }

                let len = u16::from_be_bytes(
                    len_bytes[0..2]
                        .try_into()
                        .map_err(|_| Error::UnableToDeserialize)?,
                );
                let mut message_bytes: Vec<char> = Vec::new();
                for _ in 0..len {
                    if let Some(byte) = bytes_iter.next() {
                        message_bytes.push(*byte as char);
                    } else {
                        eprintln!("error: packet ended while reading error message");
                        return Err(Error::UnableToDeserialize);
                    }
                }

                return Ok(Packet {
                    key: None,
                    value: None,
                    packet_type: PacketType::ResponseError(message_bytes.iter().collect()),
                });
            }
            _ => {
                eprintln!("error: unable to read packet type: invalid packet type flag");
                return Err(Error::UnableToDeserialize);
            }
        };

        let include_key = match bytes_iter.next() {
            Some(1) => true,
            Some(0) => false,
            _ => {
                eprintln!("error: unable to read 'include_key' flag");
                return Err(Error::UnableToDeserialize);
            }
        };

        let include_value = match bytes_iter.next() {
            Some(1) => true,
            Some(0) => false,
            _ => {
                eprintln!("error: unable to read 'include_value' flag");
                return Err(Error::UnableToDeserialize);
            }
        };

        let mut pkt = Packet {
            key: None,
            value: None,
            packet_type: pkt_type,
        };

        if include_key {
            let mut len_bytes = Vec::new();
            for _ in 0..2 {
                if let Some(byte) = bytes_iter.next() {
                    len_bytes.push(*byte);
                } else {
                    eprintln!("error: packet ended while reading key field length");
                    return Err(Error::UnableToDeserialize);
                }
            }

            let len = u16::from_be_bytes(
                len_bytes[0..2]
                    .try_into()
                    .map_err(|_| Error::UnableToDeserialize)?,
            );

            let mut key_bytes: Vec<char> = Vec::new();
            for _ in 0..len {
                if let Some(byte) = bytes_iter.next() {
                    key_bytes.push(*byte as char);
                } else {
                    eprintln!("error: packet ended while reading key field");
                    return Err(Error::UnableToDeserialize);
                }
            }

            pkt.key = Some(key_bytes.iter().collect());
        }

        if include_value {
            let mut len_bytes = Vec::new();
            for _ in 0..2 {
                if let Some(byte) = bytes_iter.next() {
                    len_bytes.push(*byte);
                } else {
                    eprintln!("error: packet ended while reading value field length");
                    return Err(Error::UnableToDeserialize);
                }
            }

            let len = u32::from_be_bytes(
                len_bytes[0..4]
                    .try_into()
                    .map_err(|_| Error::UnableToDeserialize)?,
            );

            let mut value_bytes: Vec<char> = Vec::new();
            for _ in 0..len {
                if let Some(byte) = bytes_iter.next() {
                    value_bytes.push(*byte as char);
                } else {
                    eprintln!("error: packet ended while reading value field");
                    return Err(Error::UnableToDeserialize);
                }
            }
            pkt.value = Some(Item::from_string(value_bytes.iter().collect()));
        }

        Ok(pkt)
    }
}

#[derive(Clone)]
pub enum PacketType {
    RequestGet,
    RequestSet,
    RequestDelete,
    ResponseOk,
    ResponseError(String),
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
