// Copyright (c) 2026 Sebastian Ibanez

use std::fmt;

pub enum Error {
    // Store
    StoreInsertError,
    StoreUpdateError,
    StoreDeleteError,
    KeyNotFoundError,
    // Networking
    InvalidAddr,
    UnableToBind,
    UnableToAccept,
    InvalidStream,
    // Protocol
    PacketBuildError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::StoreInsertError => "unable to insert key-value pair into store",
            Error::StoreUpdateError => "unable to update key-value pair",
            Error::StoreDeleteError => "unable to delete key-value pair ",
            Error::KeyNotFoundError => "key not found",
            Error::InvalidAddr => "invalid IP address",
            Error::UnableToBind => "unable to bind to address",
            Error::UnableToAccept => "unable to accept incoming connection",
            Error::PacketBuildError => "could not build packet",
        };

        write!(f, "{message}")
    }
}

impl Error {
    pub fn log(&self) {
        println!("error: {}", self.to_string())
    }
}
