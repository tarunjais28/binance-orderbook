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
    // Separate WebSocket URLs for book ticker and depth streams
    let book_ticker_url = format!(
        "wss://stream.binance.com:9443/ws/{}@bookTicker",
        symbol.to_lowercase()
    ).into_client_request()?;
    let depth_url = format!(
        "wss://stream.binance.com:9443/ws/{}@depth20@100ms",
        symbol.to_lowercase()
    ).into_client_request()?;

    // Connect to the Book Ticker WebSocket
    let (book_ticker_ws_stream, _) = connect_async(book_ticker_url).await?;

    let (_, mut book_ticker_read) = book_ticker_ws_stream.split(); // Split into write and read
    // let read_future = book_ticker_read.for_each(|message| async {
    //     println!("receiving...");
    //      let data = message.unwrap().into_data();
    //      tokio::io::stdout().write(&data).await.unwrap();
    //      println!("received...");
    // });
    
    // read_future.await;

    // Connect to the Depth WebSocket
    let (depth_ws_stream, _) = connect_async(depth_url).await?;
    let (_, mut depth_read) = depth_ws_stream.split(); // Split into write and read

    println!(
        "Connected to Binance WebSocket streams for symbol: {}",
        symbol
    );

    // Clone the `tx` sender for the book ticker task
    let tx_clone_for_book_ticker = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = book_ticker_read.next().await {
            // Reading from `SplitStream`
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(book_ticker) = serde_json::from_str::<BookTickerUpdate>(&text) {
                        tx_clone_for_book_ticker
                            .unbounded_send(BinanceMessage::BookTicker(book_ticker))
                            .expect("Failed to send Book Ticker message");
                    }
                }
                _ => {}
            }
        }
    });

    // Clone the `tx` sender for the depth task
    let tx_clone_for_depth = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = depth_read.next().await {
            // Reading from `SplitStream`
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(depth_update) = serde_json::from_str::<DepthUpdate>(&text) {
                        tx_clone_for_depth
                            .unbounded_send(BinanceMessage::DepthUpdate(depth_update))
                            .expect("Failed to send Depth message");
                    }
                }
                _ => {}
            }
        }
    });

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
