// Copyright (c) 2026 Sebastian Ibanez

//! Custom communication protocol for mem-store-rs.
//! Written on top of TCP.

use derive_builder::Builder;
use serde::{self, Deserialize, Serialize};

use crate::store::Item;

pub const PKT_SIZE: usize = size_of::<Packet>();

/// mem-store-rs protocol packet structure.
#[derive(Serialize, Deserialize, Default, Builder, Clone)]
pub struct Packet {
    pub key: Option<String>,
    pub value: Option<Item>,
    pub packet_type: PacketType,
}

impl Packet {}

#[derive(Serialize, Deserialize, Clone)]
pub enum PacketType {
    Request(Method),
    Response,
}

impl Default for PacketType {
    fn default() -> Self {
        PacketType::Response
    }
}

/// Request methods that can be made in the protocol.
#[derive(Serialize, Deserialize, Clone)]
pub enum Method {
    Get,
    Add,
    Update,
    Delete,
}
