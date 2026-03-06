// Copyright (c) 2026 Sebastian Ibanez

pub mod error;
pub mod protocol;
pub mod server;
pub mod store;

use crate::server::Server;

#[tokio::main]
async fn main() {
    let mut server: Server = Server::init("0.0.0.0:3000").await.unwrap();
    server.listen().await.unwrap();
}
