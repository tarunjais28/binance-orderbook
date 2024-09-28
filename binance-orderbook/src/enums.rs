use super::*;

/// Enum representing different types of messages received from Binance
#[derive(Debug)]
pub enum BinanceMessage {
    // Represents a BookTicker message with a deserialized BookTickerUpdateReader
    BookTicker(BookTickerUpdateReader),

    // Represents a DepthUpdate message with a deserialized DepthUpdateReader
    DepthUpdate(DepthUpdateReader),
}

/// Enum representing different menu commands that the system can handle
pub enum MenuCommand {
    // Command to fetch and display the best bid and ask prices from the order book
    BestBidAsk,

    // Command to fetch the volume at a specific price level; the f64 parameter represents the price
    VolumeAtPrice(f64),

    // Command to process a given JSON string (the String parameter contains the JSON data)
    JsonProcessing(String),

    // Command to handle WebSocket message processing
    WebSocketProcessing,

    // Command to exit the menu or application
    Exit,
}
