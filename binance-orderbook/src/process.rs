use super::*;

/// Function to establish a WebSocket connection to Binance and process incoming messages
pub async fn binance_websocket_client(
    symbol: &str,                        // The trading symbol (e.g., BTCUSDT)
    tx: UnboundedSender<BinanceMessage>, // The channel to send processed Binance messages to the orderbook
) -> Result<(), OrderBookError> {
    // WebSocket URL for both book ticker and depth stream for the given symbol
    let ws_url = format!(
        "wss://stream.binance.com:9443/ws/{}@bookTicker/{}@depth20@100ms",
        symbol.to_lowercase(), // Convert symbol to lowercase for the URL
        symbol.to_lowercase()
    )
    .into_client_request()?; // Convert the formatted URL string into a client request

    // Connect to the Binance WebSocket asynchronously
    let (ws_stream, _) = connect_async(ws_url).await?;
    // Split the WebSocket stream into a writer (unused here) and a reader (used to receive messages)
    let (_, mut read) = ws_stream.split();

    // Print a confirmation message indicating that the WebSocket connection was successful
    println!(
        "{}",
        format!("Connected to Binance stream for symbol: {}", symbol)
            .green()
            .bold()
    );

    // Asynchronously read messages from the WebSocket
    while let Some(msg) = read.next().await {
        match msg {
            // Handle text messages (JSON format) from the WebSocket
            Ok(Message::Text(text)) => {
                // Try to parse the message as a `BookTickerUpdate`
                if let Ok(book_ticker) = serde_json::from_str::<BookTickerUpdateReader>(&text) {
                    // If parsing succeeds, send the BookTicker message through the channel
                    tx.unbounded_send(BinanceMessage::BookTicker(book_ticker))?;
                }
                // Try to parse the message as a `DepthUpdate`
                else if let Ok(depth_update) = serde_json::from_str::<DepthUpdateReader>(&text) {
                    // If parsing succeeds, send the DepthUpdate message through the channel
                    tx.unbounded_send(BinanceMessage::DepthUpdate(depth_update))?;
                }
            }
            // Handle WebSocket close message
            Ok(Message::Close(_)) => {
                // Print a message indicating that the WebSocket connection has been closed
                println!("WebSocket connection closed.");
                break;
            }
            // Handle any error that occurs while receiving a WebSocket message
            Err(e) => {
                // Print an error message
                eprintln!("Error receiving WebSocket message: {}", e);
                break;
            }
            // Ignore other types of messages (e.g., binary)
            _ => {}
        }
    }

    Ok(())
}

/// Function to process Binance WebSocket messages and update the orderbook accordingly
pub async fn process_binance_messages(
    orderbook: &Arc<Mutex<OrderBook>>, // A shared, thread-safe reference to the orderbook
    rx: &Arc<Mutex<UnboundedReceiver<BinanceMessage>>>, // A shared, thread-safe reference to the receiver channel for Binance messages
) -> Result<(), OrderBookError> {
    // Lock the orderbook and receiver to ensure thread-safe access
    let mut orderbook = orderbook.lock().await;
    let mut rx_locked = rx.lock().await;

    // Check if there are any messages received from the WebSocket
    if let Some(message) = rx_locked.next().await {
        // Match the type of Binance message (either BookTicker or DepthUpdate)
        match message {
            // Handle `BookTicker` update messages
            BinanceMessage::BookTicker(update) => {
                // Print the BookTicker update to the console (for debugging)
                println!("{}", format!("Book Ticker Update: {:#?}", update).blue());

                // Ensure the symbol in the update matches the symbol in the orderbook
                orderbook.is_symbol_same(&update.symbol)?;

                // Ensure the update is sequential based on `lastUpdateId`
                orderbook.is_update_sequential(update.last_update_id)?;

                // Convert the update to a `BookTickerUpdate` and apply it to the orderbook
                let book_ticker_update = BookTickerUpdate::from_reader(update)?;
                orderbook.update_book_ticker(&book_ticker_update);
            }
            // Handle `DepthUpdate` update messages
            BinanceMessage::DepthUpdate(update) => {
                // Print the DepthUpdate to the console (for debugging)
                println!("{}", format!("Depth Update: {:#?}", update).yellow());

                // Ensure the update is sequential based on `lastUpdateId`
                orderbook.is_update_sequential(update.last_update_id)?;

                // Convert the update to a `DepthUpdate` and apply it to the orderbook
                let depth_update = DepthUpdate::from_reader(update);
                orderbook.update_depth(&depth_update);
            }
        }

        // After processing the message, display the current best bid and ask prices
        display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
    }

    Ok(())
}
