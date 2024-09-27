use super::OrderBook;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::WebSocketConfig, Message},
    WebSocketStream,
};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use std::error::Error;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct BookTickerUpdate {
    u: u64,    // order book updateId
    s: String, // symbol
    b: String, // best bid price
    B: String, // best bid qty
    a: String, // best ask price
    A: String, // best ask qty
}

#[derive(Debug, Deserialize)]
struct DepthUpdate {
    lastUpdateId: u64,      // Last update ID
    bids: Vec<[String; 2]>, // Price level to be updated
    asks: Vec<[String; 2]>, // Price level to be updated
}

#[derive(Debug)]
pub enum BinanceMessage {
    BookTicker(BookTickerUpdate),
    DepthUpdate(DepthUpdate),
}

pub async fn binance_websocket_client(
    symbol: &str,
    tx: UnboundedSender<BinanceMessage>,
) -> Result<(), Box<dyn Error>> {
    // WebSocket URL for the book ticker and depth stream
    let ws_url = format!(
        "wss://stream.binance.com:9443/ws/{}@bookTicker/{}@depth20@100ms",
        symbol.to_lowercase(),
        symbol.to_lowercase()
    ).into_client_request()?;

    // Connect to Binance WebSocket
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_, mut read) = ws_stream.split();

    println!("Connected to Binance stream for symbol: {}", symbol);

    // Read messages from the WebSocket
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Try to deserialize the JSON message as either BookTickerUpdate or DepthUpdate
                if let Ok(book_ticker) = serde_json::from_str::<BookTickerUpdate>(&text) {
                    tx.unbounded_send(BinanceMessage::BookTicker(book_ticker))?;
                } else if let Ok(depth_update) = serde_json::from_str::<DepthUpdate>(&text) {
                    tx.unbounded_send(BinanceMessage::DepthUpdate(depth_update))?;
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed.");
                break;
            }
            Err(e) => {
                eprintln!("Error receiving WebSocket message: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn process_binance_messages(mut rx: UnboundedReceiver<BinanceMessage>) {
    let mut orderbook = OrderBook::new("BNBUSDT".to_string());

    while let Some(message) = rx.next().await {
        match message {
            BinanceMessage::BookTicker(update) => {
                println!("Book Ticker Update: {:#?}", update);
                let book_ticker_update = crate::structs::BookTickerUpdate {
                    best_bid_price: update.b.parse().unwrap(),
                    best_bid_qty: update.B.parse().unwrap(),
                    best_ask_price: update.a.parse().unwrap(),
                    best_ask_qty: update.A.parse().unwrap(),
                };
                orderbook.update_book_ticker(&book_ticker_update);
            }
            BinanceMessage::DepthUpdate(update) => {
                println!("Depth Update: {:#?}", update);
                let depth_update = crate::structs::DepthUpdate {
                    last_update_id: update.lastUpdateId,
                    bids: update
                        .bids
                        .into_iter()
                        .map(|b| (b[0].parse().unwrap(), b[1].parse().unwrap()))
                        .collect(),
                    asks: update
                        .asks
                        .into_iter()
                        .map(|a| (a[0].parse().unwrap(), a[1].parse().unwrap()))
                        .collect(),
                };
                orderbook.update_depth(&depth_update);
            }
        }

        // Print the current best bid/ask
        if let Some((best_bid, best_ask)) = orderbook.get_best_bid_ask() {
            println!("Best Bid: {:?}, Best Ask: {:?}", best_bid, best_ask);
        }

        sleep(Duration::from_secs(20)).await;
        println!("\n\n\n\n")
    }
}
