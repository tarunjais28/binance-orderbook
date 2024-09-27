use super::*;

pub async fn binance_websocket_client(
    symbol: &str,
    tx: UnboundedSender<BinanceMessage>,
) -> Result<(), OrderBookError> {
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
    orderbook: &Arc<Mutex<OrderBook>>,
    rx: &Arc<Mutex<UnboundedReceiver<BinanceMessage>>>,
) -> Result<(), OrderBookError> {
    let mut orderbook = orderbook.lock().await;
    let mut rx_locked = rx.lock().await;

    if let Some(message) = rx_locked.next().await {
        match message {
            BinanceMessage::BookTicker(update) => {
                println!("{}", format!("Book Ticker Update: {:#?}", update).blue());

                // Ensure same symbol
                orderbook.is_symbol_same(&update.symbol)?;

                // Ensure the update is sequential based on `lastUpdateId`
                orderbook.is_update_sequential(update.last_update_id)?;

                // Updating orderbook
                let book_ticker_update = BookTickerUpdate::from_reader(update)?;
                orderbook.update_book_ticker(&book_ticker_update);
            }
            BinanceMessage::DepthUpdate(update) => {
                println!("{}", format!("Depth Update: {:#?}", update).yellow());

                // Ensure the update is sequential based on `lastUpdateId`
                orderbook.is_update_sequential(update.last_update_id)?;

                // Updating orderbook
                let depth_update = DepthUpdate::from_reader(update);
                orderbook.update_depth(&depth_update);
            }
        }

        // Print the current best bid/ask
        display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
    }

    Ok(())
}
