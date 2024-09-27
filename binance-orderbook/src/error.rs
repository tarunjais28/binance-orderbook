use super::*;

// Define an enum to handle custom errors
#[derive(Debug)]
pub enum OrderBookError {
    IoError(std::io::Error),
    JsonParseError(SerdeError),
    DifferentSymbol(String),
    UpdateIdOutdated(String),
    ParseError(String),
    ConnectionError(tungstenite::Error),
    SendError(TrySendError<BinanceMessage>),
}

// Implement `std::fmt::Display` for the custom error
impl fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderBookError::IoError(e) => write!(f, "I/O Error: {}", e),
            OrderBookError::JsonParseError(e) => write!(f, "JSON Parse Error: {}", e),
            OrderBookError::DifferentSymbol(e) => write!(f, "Symbol is different: {}", e),
            OrderBookError::UpdateIdOutdated(e) => write!(f, "lastUpdateId id outdated: {}", e),
            OrderBookError::ParseError(e) => write!(f, "Parse error: {}", e),
            OrderBookError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            OrderBookError::SendError(e) => write!(f, "Send error: {}", e),
        }
    }
}

// Implement `std::convert::From` to convert from lower-level errors to our custom error
impl From<std::io::Error> for OrderBookError {
    fn from(error: std::io::Error) -> Self {
        OrderBookError::IoError(error)
    }
}

impl From<SerdeError> for OrderBookError {
    fn from(error: SerdeError) -> Self {
        OrderBookError::JsonParseError(error)
    }
}

impl From<tungstenite::Error> for OrderBookError {
    fn from(error: tungstenite::Error) -> Self {
        OrderBookError::ConnectionError(error)
    }
}

impl From<TrySendError<BinanceMessage>> for OrderBookError {
    fn from(error: TrySendError<BinanceMessage>) -> Self {
        OrderBookError::SendError(error)
    }
}
