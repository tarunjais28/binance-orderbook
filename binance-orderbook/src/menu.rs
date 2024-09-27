use super::*;

// Function to display the menu
async fn display_menu() {
    println!("{}", "\n------- Orderbook Menu -------".green().bold());
    println!("{}", "1. View Best Bid/Ask".green());
    println!("{}", "2. Get Volume at Price".green());
    println!("{}", "3. Start Json Processing".green());
    println!("{}", "4. Start WebSocket Processing".green());
    println!("{}", "5. Exit".green());
    println!("{}", "------------------------------".green().bold());
}

// Function to process user input for menu selection
async fn get_user_input() -> Result<MenuCommand, Box<dyn Error>> {
    let mut input = String::new();
    let mut stdin = BufReader::new(tokio::io::stdin());

    stdin.read_line(&mut input).await?;
    match input.trim() {
        "1" => Ok(MenuCommand::BestBidAsk),
        "2" => {
            println!("Enter price level to get volume:");
            let mut price_input = String::new();
            stdin.read_line(&mut price_input).await?;
            if let Ok(price) = price_input.trim().parse::<f64>() {
                Ok(MenuCommand::VolumeAtPrice(price))
            } else {
                println!("Invalid input for price.");
                Ok(MenuCommand::BestBidAsk) // fallback to default
            }
        }
        "3" => {
            println!("Enter json data in compact form:");
            let mut json_input = String::new();
            stdin.read_line(&mut json_input).await?;

            Ok(MenuCommand::JsonProcessing(json_input))
        }
        "4" => Ok(MenuCommand::WebSocketProcessing),
        "5" => Ok(MenuCommand::Exit),
        _ => {
            println!("Invalid option selected.");
            Ok(MenuCommand::BestBidAsk)
        }
    }
}

// Main function to handle the user menu and interact with the orderbook
pub async fn menu_interface(
    orderbook: Arc<Mutex<OrderBook>>,
    rx: Arc<Mutex<UnboundedReceiver<BinanceMessage>>>,
) -> Result<(), Box<dyn Error>> {
    // Main loop for user menu interaction
    loop {
        display_menu().await;
        match get_user_input().await? {
            MenuCommand::BestBidAsk => {
                let orderbook = orderbook.lock().await;
                display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
            }
            MenuCommand::VolumeAtPrice(price) => {
                let orderbook = orderbook.lock().await;
                let volume = orderbook.get_volume_at_price(price);
                println!("Volume at price {}: {}", price, volume);
            }
            MenuCommand::JsonProcessing(json_input) => {
                let mut orderbook = orderbook.lock().await;
                if let Ok(update) = serde_json::from_str::<BookTickerUpdateReader>(&json_input) {
                    // Ensure same symbol
                    orderbook.is_symbol_same(&update.symbol)?;

                    // Ensure the update is sequential based on `lastUpdateId`
                    orderbook.is_update_sequential(update.last_update_id)?;

                    // Updating orderbook
                    let book_ticker_update = BookTickerUpdate::from_reader(update)?;
                    orderbook.update_book_ticker(&book_ticker_update);
                } else if let Ok(update) = serde_json::from_str::<DepthUpdateReader>(&json_input) {
                    // Ensure the update is sequential based on `lastUpdateId`
                    orderbook.is_update_sequential(update.last_update_id)?;

                    // Updating orderbook
                    let depth_update = DepthUpdate::from_reader(update);
                    orderbook.update_depth(&depth_update);
                } else {
                    eprintln!("Invalid json data!")
                };
            }
            MenuCommand::WebSocketProcessing => {
                let orderbook_clone = Arc::clone(&orderbook);
                let rx_clone = Arc::clone(&rx);
                tokio::spawn(async move {
                    if let Err(e) = process_binance_messages(&orderbook_clone, &rx_clone).await {
                        eprintln!("Error in processing request: {}", e);
                    }
                });
            }
            MenuCommand::Exit => {
                println!("Exiting...");
                break;
            }
        }
        // Small delay before showing the menu again
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
