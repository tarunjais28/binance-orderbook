use colored::*;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::{collections::BTreeMap, error::Error, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Mutex,
    time::{sleep, Duration},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

mod enums;
mod menu;
mod process;
mod structs;

#[cfg(test)]
mod tests;

use {enums::*, menu::*, process::*, structs::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Create a channel for passing messages from the WebSocket client to the processor
    let (tx, rx) = futures::channel::mpsc::unbounded();

    let symbol = "BNBUSDT";

    // Shared OrderBook state
    let orderbook = Arc::new(Mutex::new(OrderBook::new(symbol.to_string())));

    // Spawn the WebSocket client
    tokio::spawn(async move {
        if let Err(e) = binance_websocket_client(symbol, tx).await {
            eprintln!("Error in WebSocket client: {}", e);
        }
    });

    // Shared UnboundedReceiver wrapped in Arc<Mutex>
    let rx = Arc::new(Mutex::new(rx));

    // Launch menu interface for user interaction
    menu_interface(orderbook, rx).await?;

    Ok(())
}
