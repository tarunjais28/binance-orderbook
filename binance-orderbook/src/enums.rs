use super::*;

#[derive(Debug)]
pub enum BinanceMessage {
    BookTicker(BookTickerUpdateReader),
    DepthUpdate(DepthUpdateReader),
}

// Enum for menu commands
pub enum MenuCommand {
    BestBidAsk,
    VolumeAtPrice(f64),
    UpdateBid(f64, f64),
    WebSocketProcessing,
    Exit,
}
