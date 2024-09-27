use colored::*;
use futures::{
    channel::mpsc::{unbounded, TrySendError, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde_json::Error as SerdeError;
use std::{collections::BTreeMap, fmt, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Mutex,
    time::{sleep, Duration},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, client::IntoClientRequest, Message},
};

mod enums;
mod error;
mod helper;
mod menu;
mod process;
mod structs;

#[cfg(test)]
mod tests;

use {enums::*, error::*, helper::*, menu::*, process::*, structs::*};

#[tokio::main]
async fn main() -> Result<(), OrderBookError> {
    // Initialize logging
    env_logger::init();

    // Create a channel for passing messages from the WebSocket client to the processor
    let (tx, rx) = unbounded();

    let stdin = std::io::stdin();
    let mut input = String::new();

    println!("Enter coin pair symbol (bnbusdt / ethusdt / btcusdt / bnbbtc..etc):");
    stdin
        .read_line(&mut input)
        .map_err(|e| OrderBookError::IoError(e))?;
    let symbol = input.trim().to_uppercase();

    // Shared OrderBook state
    let orderbook = Arc::new(Mutex::new(OrderBook::new(symbol.to_string())));

    // Spawn the WebSocket client
    tokio::spawn(async move {
        if let Err(e) = binance_websocket_client(&symbol, tx).await {
            eprintln!("Error in WebSocket client: {}", e);
        }
    });

    // Shared UnboundedReceiver wrapped in Arc<Mutex>
    let rx = Arc::new(Mutex::new(rx));

    // Launch menu interface for user interaction
    menu_interface(orderbook, rx).await?;

    Ok(())
}
