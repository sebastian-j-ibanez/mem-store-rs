// Copyright (c) 2026 Sebastian Ibanez

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use crate::{
    error::Error,
    protocol::{Packet, PacketType, recv_packet, send_packet},
    store::Store,
};

#[derive(Debug)]
pub struct Server {
    addr: SocketAddr,
    stream: Option<TcpStream>,
    store: Store,
}

impl Server {
    pub async fn init(raw_addr: &'static str) -> Result<Self, Error> {
        if let Ok(addr) = raw_addr.parse::<SocketAddr>() {
            return Ok(Self {
                addr: addr,
                stream: None,
                store: Store::new(),
            });
        }
        Err(Error::InvalidAddr)
    }

    pub async fn listen(&mut self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|_| Error::UnableToBind)?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    self.stream = Some(stream);
                    loop {
                        match self.handle_req().await {
                            Ok(_) => continue,
                            Err(_) => break,
                        }
                    }
                }
                Err(_) => return Err(Error::UnableToAccept),
            };
        }
    }

    pub fn stream(&mut self) -> Result<&mut TcpStream, Error> {
        self.stream.as_mut().ok_or(Error::InvalidStream)
    }

    pub async fn handle_req(&mut self) -> Result<(), Error> {
        let request = recv_packet(self.stream()?).await?;

        match request.packet_type {
            PacketType::RequestGet => self.handle_get(request).await?,
            PacketType::RequestSet => self.handle_set(request).await?,
            PacketType::RequestDelete => todo!(),
            _ => todo!(),
        }

        Ok(())
    }

    pub async fn handle_get(&mut self, request: Packet) -> Result<(), Error> {
        let key = match request.key {
            Some(key) => key,
            None => {
                let response = Packet::error_response(Error::InvalidPacketFields);
                send_packet(self.stream()?, &response).await?;
                return Err(Error::InvalidPacketFields);
            }
        };

        let response = match self.store.get(key) {
            Some(value) => Packet {
                key: None,
                value: Some(value.clone()),
                packet_type: PacketType::ResponseOk,
            },
            None => Packet::ok_response(),
        };
        send_packet(self.stream()?, &response).await
    }

    pub async fn handle_set(&mut self, request: Packet) -> Result<(), Error> {
        let (key, value) = match (request.key, request.value) {
            (Some(key), Some(value)) => (key, value),
            _ => {
                let response = Packet::error_response(Error::InvalidPacketFields);
                send_packet(self.stream()?, &response).await?;
                return Err(Error::InvalidPacketFields);
            }
        };

        let response = match self.store.add(key, value) {
            Ok(_) => Packet::ok_response(),
            Err(_) => Packet::error_response(Error::StoreSetError),
        };
        send_packet(self.stream()?, &response).await
    }
}
