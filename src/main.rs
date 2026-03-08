// Copyright (c) 2026 Sebastian Ibanez

pub mod client;
pub mod error;
pub mod protocol;
pub mod server;
pub mod store;

use crate::{client::Client, error::Error, server::Server, store::Item};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server_handle = tokio::spawn(async { run_server().await });
    let client_handle = tokio::spawn(async { run_client().await });
    client_handle.await.unwrap()?;
    server_handle.abort();
    Ok(())
}

async fn run_server() -> Result<(), Error> {
    let mut server: Server = Server::init("0.0.0.0:3000").await?;
    server.listen().await
}

async fn run_client() -> Result<(), Error> {
    let mut client = Client::init("127.0.0.1:3000").await.unwrap();
    client.connect().await.unwrap();
    let key = String::from("my_list");
    let value = Item::from_string(String::from("a,b,c"));
    client.set_value(key.clone(), value.clone()).await.unwrap();
    match client.get_value(key).await.unwrap() {
        Some(response_value) => {
            println!(
                "expected: {}\ngot:      {}",
                value.to_string().replace('\n', "\n          "),
                response_value.to_string().replace('\n', "\n          ")
            );
            Ok(())
        }
        None => Err(Error::MissingPacketFields),
    }
}
