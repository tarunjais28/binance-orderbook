use super::*;

/// Function to display the menu
/// This function asynchronously prints a list of menu options for the user.
async fn display_menu() {
    // Display the menu header
    println!("{}", "\n------- Orderbook Menu -------".green().bold());

    // Display the option to view the best bid/ask
    println!("{}", "1. View Best Bid/Ask".green());

    // Display the option to get the volume at a specific price
    println!("{}", "2. Get Volume at Price".green());

    // Display the option to start JSON processing
    println!("{}", "3. Start Json Processing".green());

    // Display the option to start WebSocket processing
    println!("{}", "4. Start WebSocket Processing".green());

    // Display the option to exit the program
    println!("{}", "5. Exit".green());

    // Display the footer
    println!("{}", "------------------------------".green().bold());
}

/// Function to process user input for menu selection
/// This function asynchronously reads user input and maps it to a corresponding menu command.
async fn get_user_input() -> Result<MenuCommand, OrderBookError> {
    // Create a new string to hold the user input
    let mut input = String::new();
    // Create a buffered reader for reading input from stdin (standard input)
    let mut stdin = BufReader::new(tokio::io::stdin());

    // Read a line of input from the user asynchronously
    stdin.read_line(&mut input).await?;

    // Match the user's input with the available menu options
    match input.trim() {
        // If the input is "1", return the `BestBidAsk` command
        "1" => Ok(MenuCommand::BestBidAsk),
        // If the input is "2", ask for a price level and return the `VolumeAtPrice` command
        "2" => {
            println!("Enter price level to get volume:");
            let mut price_input = String::new();
            stdin.read_line(&mut price_input).await?;
            if let Ok(price) = price_input.trim().parse::<f64>() {
                // If the price input is valid, return the command with the specified price
                Ok(MenuCommand::VolumeAtPrice(price))
            } else {
                // If the input is invalid, notify the user and return the default `BestBidAsk` command
                println!("Invalid input for price.");
                Ok(MenuCommand::BestBidAsk) // fallback to default
            }
        }
        // If the input is "3", ask for JSON data and return the `JsonProcessing` command
        "3" => {
            println!("Enter json data in compact form:");
            let mut json_input = String::new();
            stdin.read_line(&mut json_input).await?;
            // Return the JSON processing command with the user's input
            Ok(MenuCommand::JsonProcessing(json_input))
        }
        // If the input is "4", return the `WebSocketProcessing` command
        "4" => Ok(MenuCommand::WebSocketProcessing),
        // If the input is "5", return the `Exit` command
        "5" => Ok(MenuCommand::Exit),
        // If the input is invalid, notify the user and return the default `BestBidAsk` command
        _ => {
            println!("Invalid option selected.");
            Ok(MenuCommand::BestBidAsk)
        }
    }
}

/// Main function to handle the user menu and interact with the orderbook
/// This function processes the user's commands and interacts with the orderbook asynchronously.
pub async fn menu_interface(
    orderbook: Arc<Mutex<OrderBook>>, // A shared, thread-safe reference to the orderbook
    rx: Arc<Mutex<UnboundedReceiver<BinanceMessage>>>, // A shared, thread-safe reference to the Binance message receiver
) -> Result<(), OrderBookError> {
    // Main loop for user menu interaction
    loop {
        // Display the menu and wait for the user's input
        display_menu().await;
        // Handle the user's menu selection
        match get_user_input().await? {
            // If the `BestBidAsk` command is selected, display the best bid/ask prices
            MenuCommand::BestBidAsk => {
                // Lock the orderbook to ensure thread-safe access
                let orderbook = orderbook.lock().await;
                // Call a function to display the best bid/ask prices
                display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
            }
            // If the `VolumeAtPrice` command is selected, display the volume at the specified price
            MenuCommand::VolumeAtPrice(price) => {
                // Lock the orderbook to ensure thread-safe access
                let orderbook = orderbook.lock().await;
                // Get the volume at the specified price and display it
                let volume = orderbook.get_volume_at_price(price);
                println!(
                    "{}",
                    format!("Volume at price {}: {}", price, volume).cyan()
                );
            }
            // If the `JsonProcessing` command is selected, process the provided JSON data
            MenuCommand::JsonProcessing(json_input) => {
                let mut orderbook = orderbook.lock().await;
                // Try to parse the input as a BookTickerUpdate message
                if let Ok(update) = serde_json::from_str::<BookTickerUpdateReader>(&json_input) {
                    // Ensure the symbol in the update matches the orderbook's symbol
                    if let Err(err) = orderbook.is_symbol_same(&update.symbol) {
                        eprintln!("{}", err.to_string().red());
                        continue;
                    }

                    // Ensure the update is sequential based on `lastUpdateId`
                    if let Err(err) = orderbook.is_update_sequential(update.last_update_id) {
                        eprintln!("{}", err.to_string().red());
                        continue;
                    };

                    // Update the orderbook with the new book ticker data
                    let book_ticker_update = match BookTickerUpdate::from_reader(update) {
                        Ok(u) => u,
                        Err(err) => {
                            eprintln!("{}", err.to_string().red());
                            continue;
                        }
                    };
                    orderbook.update_book_ticker(&book_ticker_update);

                    // Call a function to display the best bid/ask prices
                    display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
                }
                // Try to parse the input as a DepthUpdate message
                else if let Ok(update) = serde_json::from_str::<DepthUpdateReader>(&json_input) {
                    // Ensure the update is sequential based on `lastUpdateId`
                    if let Err(err) = orderbook.is_update_sequential(update.last_update_id) {
                        eprintln!("{}", err.to_string().red());
                        continue;
                    };

                    // Update the orderbook with the new depth data
                    let depth_update = DepthUpdate::from_reader(update);
                    orderbook.update_depth(&depth_update);

                    // Call a function to display the best bid/ask prices
                    display_best_bid_ask(&orderbook, |orderbook| orderbook.get_best_bid_ask());
                } else {
                    // If the input is invalid, print an error message
                    eprintln!("{}", OrderBookError::IncorrectJsonData.to_string().red());
                };
            }
            // If the `WebSocketProcessing` command is selected, start processing WebSocket messages
            MenuCommand::WebSocketProcessing => {
                // Clone the orderbook and receiver to use in the spawned task
                let orderbook_clone = Arc::clone(&orderbook);
                let rx_clone = Arc::clone(&rx);
                // Spawn an asynchronous task to process WebSocket messages
                tokio::spawn(async move {
                    if let Err(e) = process_binance_messages(&orderbook_clone, &rx_clone).await {
                        // If an error occurs, print it
                        eprintln!("{}", e.to_string().red());
                    }
                });
            }
            // If the `Exit` command is selected, break out of the loop and end the program
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
