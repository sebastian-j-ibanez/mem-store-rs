// Copyright (c) 2026 Sebastian Ibanez

use std::fmt;

pub enum Error {
    StoreInsertError,
    StoreUpdateError,
    StoreDeleteError,
    KeyNotFoundError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::StoreInsertError => "unable to insert key-value pair into store",
            Error::StoreUpdateError => "unable to update key-value pair",
            Error::StoreDeleteError => "unable to delete key-value pair ",
            Error::KeyNotFoundError => "key not found",
        };

        write!(f, "{message}")
    }
}

impl Error {
    pub fn log(&self) {
        println!("error: {}", self.to_string())
    }
}
