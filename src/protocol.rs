// Copyright (c) 2026 Sebastian Ibanez

//! Custom communication protocol for mem-store-rs.
//! Written on top of TCP.

use serde::{self, Deserialize};

use crate::{error::Error, store::Item};

/// Request methods that can be made in the protocol.
#[derive(Deserialize)]
pub enum Method {
    Get,
    Add,
    Update,
    Delete,
}

/// mem-store-rs protocol packet structure.
#[derive(Deserialize)]
pub struct Packet {
    key: String,
    value: Option<Item>,
    method: Method,
}

impl Packet {}

pub struct PacketBuilder {
    key: Option<String>,
    value: Option<Option<Item>>,
    method: Option<Method>,
}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder {
            key: None,
            value: None,
            method: None,
        }
    }

    pub fn key(self, k: String) -> Self {
        Self {
            key: Some(k),
            ..self
        }
    }

    pub fn value(self, v: Item) -> Self {
        Self {
            value: Some(Some(v)),
            ..self
        }
    }

    pub fn method(self, m: Method) -> Self {
        Self {
            method: Some(m),
            ..self
        }
    }

    pub fn build(self) -> Result<Packet, Error> {
        Ok(Packet {
            key: self.key.ok_or(Error::PacketBuildError)?,
            value: self.value.ok_or(Error::PacketBuildError)?,
            method: self.method.ok_or(Error::PacketBuildError)?,
        })
    }
}
