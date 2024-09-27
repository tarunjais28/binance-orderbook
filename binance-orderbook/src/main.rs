use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use std::error::Error;
use tokio::sync::mpsc;

mod process;
mod structs;

#[cfg(test)]
mod tests;

use {process::*, structs::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Create a channel for passing messages from the WebSocket client to the processor
    let (tx, rx) = futures::channel::mpsc::unbounded();

    // Spawn the WebSocket client
    tokio::spawn(async move {
        if let Err(e) = binance_websocket_client("BNBUSDT", tx).await {
            eprintln!("Error in WebSocket client: {}", e);
        }
    });

    // Process the incoming WebSocket messages
    process_binance_messages(rx).await;

    Ok(())
}
