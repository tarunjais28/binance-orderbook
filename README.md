# Binance Order Book WebSocket Client

This project implements a WebSocket client that connects to Binance's book ticker and depth streams. It processes real-time updates of order book data, such as bids and asks, and provides an interactive command-line interface (CLI) for querying the data.

## Features

- **WebSocket Integration**: 
  Connects to Binance's WebSocket API to receive real-time order book updates.
  
- **Order Book Management**: 
  Manages and updates the order book state based on WebSocket data. Handles both book ticker updates and depth updates.

- **Command-Line Interface**: 
  An interactive menu allows users to query the best bid/ask prices, volume at specific price levels, and more.

- **Asynchronous Concurrency**: 
  Built with `tokio` for non-blocking asynchronous operations, supporting WebSocket communication and user input simultaneously.

## Project Structure

### Order Book

The `OrderBook` struct handles the order book data for a specific trading pair. It keeps track of:

- **Bids**: Stored as a `BTreeMap` to maintain price levels in an ordered fashion (highest to lowest).
- **Asks**: Stored as a `BTreeMap` to maintain price levels in an ordered fashion (lowest to highest).

#### Key Methods

- `update_book_ticker`: Updates the best bid and ask prices in the order book based on book ticker data from Binance.
- `update_depth`: Processes depth updates, modifying the bids and asks according to the incoming levels from Binance.
- `get_best_bid_ask`: Returns the current best bid and ask prices.
- `get_volume_at_price`: Retrieves the volume for a specific price, either from the bids or asks.

### WebSocket Client

The WebSocket client connects to Binance's WebSocket API and listens for real-time updates:

- **Book Ticker Stream**: Receives updates about the best bid and ask prices for the selected trading pair.
- **Depth Stream**: Receives updates for the top 20 levels of bids and asks.

WebSocket messages are deserialized into either `BookTickerUpdateReader` or `DepthUpdateReader` structs, which are then used to update the order book.

### CLI Menu

The system provides a user-friendly interactive command-line interface with several commands:

- **BestBidAsk**: Displays the current best bid and ask prices.
- **VolumeAtPrice**: Displays the volume at a specified price level.
- **JsonProcessing**: Allows manual processing of JSON messages for testing.
- **WebSocketProcessing**: Manages WebSocket communication.
- **Exit**: Exits the application.

### Error Handling

A custom error type, `OrderBookError`, is used to handle various error cases, such as:

- **I/O Errors**: Errors from reading/writing to the standard input/output.
- **JSON Parsing Errors**: Issues with deserializing WebSocket messages.
- **WebSocket Connection Errors**: Problems with the WebSocket connection.
- **Symbol Mismatch Errors**: Errors when the symbol in the update message doesn't match the current trading pair.
- **Update Sequence Errors**: Issues when update messages are received out of order.

## Usage

### Prerequisites

- **Rust**: This project is written in Rust, so the Rust need to be installed. Rust can be installed from [rust-lang.org](https://www.rust-lang.org/).

### Running the Project

1. Build and run the project:

    ```rust
    cd binance-orderbook
    cargo run --release
    ```

2. Upon running, user will be prompted to enter a trading pair symbol (e.g., BTCUSDT, ETHUSDT, etc.).

3. After connecting to Binance's WebSocket stream, use the interactive CLI to query the best bid/ask, volume at a specific price, json data processing, websocket processing or exit the program.

### Example Commands

- **Best Bid Ask**: Displays the current best bid and ask prices from the order book.

- **Volume At Price**: Shows the total volume at a given price level.

- **Json Data Processing**: Allows user to manually update the orderbook via json files (in compact form).

#### For Book Ticker Update use below format

```json
{"u":400900217,"s":"BNBUSDT","b":"25.35190000","B":"31.21000000","a":"25.36520000","A":"40.66000000"}
```

#### For Depth Update use below format

```json
{"lastUpdateId":160,"bids":[["0.0024","10"]],"asks":[["0.0026","100"]]}
```

- **Web Socket Processing**: Process Book Ticker Update and Depth Update based on the web socket address: `wss://stream.binance.com:9443/ws/<symbol>@bookTicker/<symbol>@depth20@100ms`. On different run different updates (either Book Ticker or Depth) will be applied to OrderBook.

- **Exit**: Terminates the WebSocket connection and exits the program.

## Key Dependencies

- **Tokio**: For async runtime and concurrency.

- **Tokio-Tungstenite**: For WebSocket communication.

- **Serde**: For JSON deserialization of WebSocket messages.

- **OrderedFloat**: For ensuring proper ordering of floating-point numbers in the order book.

- **Colored**: For colored terminal output in the CLI.
