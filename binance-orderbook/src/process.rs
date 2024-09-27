use super::*;

pub async fn binance_websocket_client(
    symbol: &str,
    tx: UnboundedSender<BinanceMessage>,
) -> Result<(), Box<dyn Error>> {
    // WebSocket URL for the book ticker and depth stream
    let ws_url = format!(
        "wss://stream.binance.com:9443/ws/{}@bookTicker/{}@depth20@100ms",
        symbol.to_lowercase(),
        symbol.to_lowercase()
    )
    .into_client_request()?;

    // Connect to Binance WebSocket
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_, mut read) = ws_stream.split();

    println!(
        "{}",
        format!("Connected to Binance stream for symbol: {}", symbol)
            .green()
            .bold()
    );

    // Read messages from the WebSocket
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Try to deserialize the JSON message as either BookTickerUpdate or DepthUpdate
                if let Ok(book_ticker) = serde_json::from_str::<BookTickerUpdateReader>(&text) {
                    tx.unbounded_send(BinanceMessage::BookTicker(book_ticker))?;
                } else if let Ok(depth_update) = serde_json::from_str::<DepthUpdateReader>(&text) {
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

pub async fn process_binance_messages(
    symbol: &str,
    mut rx: UnboundedReceiver<BinanceMessage>,
) -> Result<(), Box<dyn Error>> {
    let mut orderbook = OrderBook::new(symbol.to_string());
    let mut last_update_id = 0;

    while let Some(message) = rx.next().await {
        match message {
            BinanceMessage::BookTicker(update) => {
                println!("{}", format!("Book Ticker Update: {:#?}", update).blue());

                // Ensure same symbol
                if !orderbook.symbol.eq(&update.symbol) {
                    eprintln!(
                        "Symbol is different! expected: {}, found: {}",
                        orderbook.symbol, update.symbol
                    );
                }

                // Ensure the update is sequential based on `lastUpdateId`
                if update.last_update_id < last_update_id {
                    println!("Skipping outdated update: {}", update.last_update_id);
                    continue;
                } else {
                    // Update the lastUpdateId to match the latest one processed
                    last_update_id = update.last_update_id;

                    // Update is valid
                    let book_ticker_update = BookTickerUpdate::from_reader(update)?;
                    orderbook.update_book_ticker(&book_ticker_update);
                }
            }
            BinanceMessage::DepthUpdate(update) => {
                println!("{}", format!("Depth Update: {:#?}", update).yellow());

                // Ensure the update is sequential based on `lastUpdateId`
                if update.last_update_id < last_update_id {
                    println!("Skipping outdated update: {}", update.last_update_id);
                    continue;
                } else {
                    // Update the lastUpdateId to match the latest one processed
                    last_update_id = update.last_update_id;

                    // Update is valid and sequential
                    let depth_update = DepthUpdate::from_reader(update);
                    orderbook.update_depth(&depth_update);
                }
            }
        }

        // Print the current best bid/ask
        if let Some((best_bid, best_ask)) = orderbook.get_best_bid_ask() {
            println!(
                "{}",
                format!("Best Bid: {:?}, Best Ask: {:?}\n\n", best_bid, best_ask).purple()
            );
        }

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
