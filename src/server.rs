// Copyright (c) 2026 Sebastian Ibanez

use std::net::IpAddr;
use tokio::net::{TcpListener, TcpStream};

use crate::error::Error;

pub struct Server {
    addr: IpAddr,
    stream: Option<TcpStream>,
}

impl Server {
    pub async fn init(raw_addr: &'static str) -> Result<Self, Error> {
        if let Ok(addr) = raw_addr.parse::<IpAddr>() {
            return Ok(Self {
                addr: addr,
                stream: None,
            });
        }
        Err(Error::InvalidAddr)
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.addr.to_string())
            .await
            .map_err(|_| Error::UnableToBind)?;
        self.stream = match listener.accept().await {
            Ok((stream, _)) => Some(stream),
            Err(_) => return Err(Error::UnableToAccept),
        };
        Ok(())
    }
}
