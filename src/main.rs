// Copyright (c) 2026 Sebastian Ibanez

pub mod api;
pub mod error;
pub mod protocol;
pub mod store;

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{api::new_router, store::Store};

#[tokio::main]
async fn main() {
    let shared_store = Arc::new(RwLock::new(Store::new()));
    let router = new_router().with_state(shared_store);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
