use colored::*;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::{collections::BTreeMap, error::Error};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

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

    let symbol = "BNBUSDT";

    // Spawn the WebSocket client
    tokio::spawn(async move {
        if let Err(e) = binance_websocket_client(&symbol, tx).await {
            eprintln!("Error in WebSocket client: {}", e);
        }
    });

    // Process the incoming WebSocket messages
    process_binance_messages(&symbol, rx).await?;

    Ok(())
}
