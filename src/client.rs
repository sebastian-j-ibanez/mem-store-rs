// Copyright (c) 2026 Sebastian Ibanez

use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::{
    error::Error,
    protocol::{Packet, PacketType, recv_packet, send_packet},
    store::Item,
};

#[derive(Debug)]
pub struct Client {
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl Client {
    pub async fn init(raw_addr: &'static str) -> Result<Self, Error> {
        if let Ok(addr) = raw_addr.parse::<SocketAddr>() {
            return Ok(Self {
                addr: addr,
                stream: None,
            });
        }
        Err(Error::InvalidAddr)
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        self.stream = match TcpStream::connect(self.addr).await {
            Ok(stream) => Some(stream),
            Err(_) => return Err(Error::ConnectionTimedOut),
        };
        Ok(())
    }

    pub fn stream(&mut self) -> Result<&mut TcpStream, Error> {
        self.stream.as_mut().ok_or(Error::InvalidStream)
    }

    pub async fn get_value(&mut self, key: String) -> Result<Option<Item>, Error> {
        let request = Packet::get_request(key);
        send_packet(self.stream()?, &request).await?;
        let response = recv_packet(self.stream()?).await?;
        Ok(response.value)
    }

    pub async fn set_value(&mut self, key: String, value: Item) -> Result<(), Error> {
        let request = Packet::set_request(key, value);
        send_packet(self.stream()?, &request).await?;
        let response = recv_packet(self.stream()?).await?;
        match response.packet_type {
            PacketType::ResponseOk => Ok(()),
            PacketType::ResponseError(e) => return Err(e),
            _ => return Err(Error::UnexpectedPacketType),
        }
    }

    pub async fn delete_value(&mut self, key: String) -> Result<(), Error> {
        let stream = self.stream()?;
        let request = Packet::delete_request(key);
        send_packet(stream, &request).await?;
        let response = recv_packet(stream).await?;
        match response.packet_type {
            PacketType::ResponseOk => Ok(()),
            PacketType::ResponseError(e) => return Err(e),
            _ => return Err(Error::UnexpectedPacketType),
        }
    }
}
