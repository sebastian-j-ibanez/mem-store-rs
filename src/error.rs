// Copyright (c) 2026 Sebastian Ibanez

use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    // Store
    StoreSetError,
    StoreDeleteError,
    KeyNotFoundError,
    // Networking
    InvalidAddr,
    UnableToBind,
    UnableToAccept,
    InvalidStream,
    ConnectionTimedOut,
    UnableToSend,
    UnableToReceive,
    // Packet
    InvalidPacketFields,
    PacketBuildError,
    UnableToSerialize,
    UnableToDeserialize,
    MissingPacketFields,
    UnexpectedPacketType,
    ValueLengthTooLong,
    InvalidPacketTypeFlag,
    InvalidPacketLengthFlag,
    InvalidIncludeFlag,
    InvalidKeyField,
    InvalidValueField,
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::StoreSetError => "unable to set key-value pair",
            Error::StoreDeleteError => "unable to delete key-value pair ",
            Error::KeyNotFoundError => "key not found",
            Error::InvalidAddr => "invalid IP address",
            Error::UnableToBind => "unable to bind to address",
            Error::UnableToAccept => "unable to accept incoming connection",
            Error::ConnectionTimedOut => "connection timed out",
            Error::UnableToSend => "unable to send packet through stream",
            Error::UnableToReceive => "unable to receive packet from stream",
            Error::PacketBuildError => "could not build packet",
            Error::InvalidStream => "invalid stream",
            Error::UnableToSerialize => "unable to serialize packet",
            Error::UnableToDeserialize => "unable to deserialize packet",
            Error::InvalidPacketFields => "invalid packet fields",
            Error::MissingPacketFields => "one or more packet fields are missing data",
            Error::UnexpectedPacketType => "unexpected packet type",
            Error::ValueLengthTooLong => "value exceeds max length of 4.2GB",
            Error::InvalidPacketTypeFlag => "invalid packet type flag",
            Error::InvalidPacketLengthFlag => "invalid packet length flag",
            Error::InvalidIncludeFlag => "invalid include flag",
            Error::InvalidKeyField => "invalid key field",
            Error::InvalidValueField => "invalid value field",
            Error::Custom(msg) => msg,
        };

        write!(f, "{message}")
    }
}

impl Error {
    pub fn from_string(message: impl Into<String>) -> Self {
        Error::Custom(message.into())
    }

    pub fn log(&self) {
        println!("error: {}", self.to_string())
    }
}
