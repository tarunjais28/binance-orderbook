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

/// Main function with asynchronous runtime using Tokio
#[tokio::main]
async fn main() -> Result<(), OrderBookError> {
    // Initialize the logger for logging purposes
    env_logger::init();

    // Create an unbounded channel for sending and receiving messages asynchronously
    let (tx, rx) = unbounded();

    // Prepare to capture user input from stdin
    let stdin = std::io::stdin(); // Standard input
    let mut input = String::new(); // Buffer for user input

    // Prompt the user to enter a coin pair symbol (e.g., BTCUSDT, ETHUSDT)
    println!("Enter coin pair symbol (bnbusdt / ethusdt / btcusdt / bnbbtc..etc):");
    // Read the user input and handle potential IO errors
    stdin
        .read_line(&mut input)
        .map_err(|e| OrderBookError::IoError(e))?; // If there's an error reading input, convert it to `OrderBookError::IoError`

    // Trim whitespace from input and convert the coin symbol to uppercase
    let symbol = input.trim().to_uppercase();

    // Create a new `OrderBook` instance and wrap it in an `Arc<Mutex>` to allow shared access between async tasks
    let orderbook = Arc::new(Mutex::new(OrderBook::new(symbol.to_string())));

    // Spawn an asynchronous task to handle WebSocket communication for the specified coin pair
    tokio::spawn(async move {
        // Call the WebSocket client for Binance. If there's an error, it gets logged.
        if let Err(e) = binance_websocket_client(&symbol, tx).await {
            eprintln!("Error in WebSocket client: {}", e); // Log the error
        }
    });

    // Wrap the receiver in `Arc<Mutex>` for shared access
    let rx = Arc::new(Mutex::new(rx));

    // Launch the user menu interface for interacting with the orderbook and WebSocket
    menu_interface(orderbook, rx).await?;

    Ok(())
}
