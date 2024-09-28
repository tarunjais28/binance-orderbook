use super::*;

/// Define a custom error type for handling different kinds of errors in the order book
#[derive(Debug)] // Enable debug formatting for the enum
pub enum OrderBookError {
    // Error caused by issues in I/O operations (e.g., reading/writing from/to stdin, files, etc.)
    IoError(std::io::Error),

    // Error in parsing JSON data (deserialization failure)
    JsonParseError(SerdeError),

    // Error when an operation encounters a symbol mismatch in the order book
    DifferentSymbol(String),

    // Error when the `lastUpdateId` is outdated, indicating the received update is not valid
    UpdateIdOutdated(String),

    // Error when parsing a value (e.g., price or volume) fails
    ParseError(String),

    // Error when a WebSocket connection fails or an issue occurs during communication
    ConnectionError(tungstenite::Error),

    // Error when sending a message over the channel fails
    SendError(TrySendError<BinanceMessage>),

    // Error when json data is incorrect
    IncorrectJsonData,
}

/// Implement the `Display` trait for the `OrderBookError` enum
/// This allows us to convert the errors into human-readable strings for easy debugging and logging
impl fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Convert the `IoError` into a string with "I/O Error" prefix
            OrderBookError::IoError(e) => write!(f, "I/O Error: {}", e),

            // Convert the `JsonParseError` into a string with "JSON Parse Error" prefix
            OrderBookError::JsonParseError(e) => write!(f, "JSON Parse Error: {}", e),

            // Custom message when the symbols in the order book are different
            OrderBookError::DifferentSymbol(e) => write!(f, "Symbol is different: {}", e),

            // Custom message when the `lastUpdateId` is outdated
            OrderBookError::UpdateIdOutdated(e) => write!(f, "lastUpdateId is outdated: {}", e),

            // Generic parse error with a custom message
            OrderBookError::ParseError(e) => write!(f, "Parse error: {}", e),

            // Error when there's a WebSocket connection issue
            OrderBookError::ConnectionError(e) => write!(f, "Connection error: {}", e),

            // Error when a message can't be sent over a channel
            OrderBookError::SendError(e) => write!(f, "Send error: {}", e),

            // Error when a json data is incorrect
            OrderBookError::IncorrectJsonData => write!(f, "Json data is incorrect!"),
        }
    }
}

/// Implement `From` trait for automatic conversion from `std::io::Error` to `OrderBookError::IoError`
/// This allows using the `?` operator in functions that return `Result<(), OrderBookError>`
impl From<std::io::Error> for OrderBookError {
    fn from(error: std::io::Error) -> Self {
        // Convert the `std::io::Error` into `OrderBookError::IoError`
        OrderBookError::IoError(error)
    }
}

/// Implement `From` for converting `serde_json::Error` into `OrderBookError::JsonParseError`
impl From<SerdeError> for OrderBookError {
    fn from(error: SerdeError) -> Self {
        // Convert `SerdeError` into `OrderBookError::JsonParseError`
        OrderBookError::JsonParseError(error)
    }
}

/// Implement `From` for converting WebSocket connection errors into `OrderBookError::ConnectionError`
impl From<tungstenite::Error> for OrderBookError {
    fn from(error: tungstenite::Error) -> Self {
        // Convert `tungstenite::Error` into `OrderBookError::ConnectionError`
        OrderBookError::ConnectionError(error)
    }
}

/// Implement `From` for converting channel send errors into `OrderBookError::SendError`
/// This allows converting `TrySendError<BinanceMessage>` into our custom error
impl From<TrySendError<BinanceMessage>> for OrderBookError {
    fn from(error: TrySendError<BinanceMessage>) -> Self {
        // Convert `TrySendError` into `OrderBookError::SendError`
        OrderBookError::SendError(error)
    }
}
