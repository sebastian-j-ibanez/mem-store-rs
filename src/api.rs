// Copyright (c) 2026 Sebastian Ibanez

use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use tokio::sync::RwLock;

use crate::{
    error::Error,
    store::{Item, Store},
};

pub type SharedStore = Arc<RwLock<Store>>;

pub fn new_router() -> Router<SharedStore> {
    Router::new()
        .route("/{key}", get(get_key_handler))
        .route("/{key}", delete(delete_key_handler))
        .route("/{key}/{value}", post(post_key_handler))
}

/// Handle `GET /{key}` requests.
async fn get_key_handler(
    Path(key): Path<String>,
    State(store_lock): State<SharedStore>,
) -> Result<String, StatusCode> {
    let store = store_lock.read().await;
    if let Some(value) = store.get(key) {
        return Ok(value.clone().to_string());
    }
    Err(StatusCode::NOT_FOUND)
}

/// Handle `POST /{key}/{value}` requests.
async fn post_key_handler(
    Path((key, value)): Path<(String, String)>,
    State(store_lock): State<SharedStore>,
) -> Result<(), StatusCode> {
    let mut store = store_lock.write().await;
    if let Err(e) = store.add(key, Item::from_string(value)) {
        println!("error: {}", e.to_string());
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(())
}

/// Handle `DELETE /{key}` requests.
async fn delete_key_handler(
    Path(key): Path<String>,
    State(store_lock): State<SharedStore>,
) -> Result<(), StatusCode> {
    let mut store = store_lock.write().await;
    match store.delete(key) {
        Ok(_) => Ok(()),
        Err(e @ Error::KeyNotFoundError) => {
            e.log();
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            e.log();
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
