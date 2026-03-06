// Copyright (c) 2026 Sebastian Ibanez

use postcard::{from_bytes, to_allocvec};
use std::net::IpAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    error::Error,
    protocol::{Method, Packet, PacketType},
    store::Store,
};

pub struct Server {
    addr: IpAddr,
    stream: Option<TcpStream>,
    store: Store,
}

impl Server {
    pub async fn init(raw_addr: &'static str) -> Result<Self, Error> {
        if let Ok(addr) = raw_addr.parse::<IpAddr>() {
            return Ok(Self {
                addr: addr,
                stream: None,
                store: Store::new(),
            });
        }
        Err(Error::InvalidAddr)
    }

    pub async fn listen(&mut self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.addr.to_string())
            .await
            .map_err(|_| Error::UnableToBind)?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    self.stream = Some(stream);
                    self.handle_req().await?;
                }
                Err(_) => return Err(Error::UnableToAccept),
            };
        }
    }

    async fn send_packet(&mut self, packet: &Packet) -> Result<(), Error> {
        let stream = self.stream.as_mut().ok_or(Error::InvalidStream)?;
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

    async fn recv_packet(&mut self) -> Result<Packet, Error> {
        let stream = self.stream.as_mut().ok_or(Error::InvalidStream)?;
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

    pub async fn handle_req(&mut self) -> Result<(), Error> {
        let request = self.recv_packet().await?;

        match request.packet_type {
            PacketType::Request(Method::Get) => self.handle_get(request).await?,
            PacketType::Request(Method::Add) => todo!(),
            PacketType::Request(Method::Update) => todo!(),
            PacketType::Request(Method::Delete) => todo!(),
            PacketType::Response => todo!(),
        }

        Ok(())
    }

    pub async fn handle_get(&mut self, request: Packet) -> Result<(), Error> {
        let key = match request.key {
            Some(key) => key,
            None => {
                let response = Packet {
                    key: None,
                    value: None,
                    packet_type: PacketType::Response,
                };
                self.send_packet(&response).await?;
                return Err(Error::InvalidPacketFields);
            }
        };

        let response = match self.store.get(key) {
            Some(value) => Packet {
                key: None,
                value: Some(value.clone()),
                packet_type: PacketType::Response,
            },
            None => Packet {
                key: None,
                value: None,
                packet_type: PacketType::Response,
            },
        };
        self.send_packet(&response).await
    }
}
