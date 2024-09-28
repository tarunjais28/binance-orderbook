use super::*;

/// Struct representing the order book with bids, asks, symbol, and the last update ID
#[derive(Debug, Clone)]
pub struct OrderBook {
    // Trading pair symbol (e.g., BTCUSDT)
    symbol: String,

    // Last update ID for tracking sequential updates
    last_update_id: u64,

    // Map to store bids (price -> quantity), OrderedFloat ensures correct float comparison
    pub bids: BTreeMap<OrderedFloat<f64>, f64>,

    // Map to store asks (price -> quantity)
    pub asks: BTreeMap<OrderedFloat<f64>, f64>,
}

impl OrderBook {
    // Constructor function to create a new OrderBook
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,                // Initialize the symbol for the order book
            last_update_id: 0,     // Set the initial update ID to 0
            bids: BTreeMap::new(), // Initialize empty bids map
            asks: BTreeMap::new(), // Initialize empty asks map
        }
    }

    // Function to update the book ticker (best bid and ask)
    pub fn update_book_ticker(&mut self, data: &BookTickerUpdate) {
        // Update the last_update_id with the new data's update ID
        self.last_update_id = data.last_update_id;

        // Update bids: if the bid quantity is positive, add/update the bid; otherwise, remove it
        if data.bid_qty > 0.0 {
            self.bids.insert(OrderedFloat(data.bid_price), data.bid_qty);
        } else {
            self.bids.remove(&OrderedFloat(data.bid_price));
        }

        // Update asks: if the ask quantity is positive, add/update the ask; otherwise, remove it
        if data.ask_qty > 0.0 {
            self.asks.insert(OrderedFloat(data.ask_price), data.ask_qty);
        } else {
            self.asks.remove(&OrderedFloat(data.ask_price));
        }
    }

    // Function to update the depth of the order book (multiple bid/ask updates)
    pub fn update_depth(&mut self, data: &DepthUpdate) {
        // Update the last_update_id with the new data's update ID
        self.last_update_id = data.last_update_id;

        // Update bids: process all price levels from the update
        for (price, qty) in &data.bids {
            if *qty > 0.0 {
                self.bids.insert(OrderedFloat(*price), *qty);
            } else {
                self.bids.remove(&OrderedFloat(*price));
            }
        }

        // Update asks: process all price levels from the update
        for (price, qty) in &data.asks {
            if *qty > 0.0 {
                self.asks.insert(OrderedFloat(*price), *qty);
            } else {
                self.asks.remove(&OrderedFloat(*price));
            }
        }
    }

    // Function to get the best bid (highest) and best ask (lowest) from the order book
    pub fn get_best_bid_ask(&self) -> Option<((f64, f64), (f64, f64))> {
        // Get the highest bid (last entry in the map)
        let best_bid = self.bids.iter().next_back();
        // Get the lowest ask (first entry in the map)
        let best_ask = self.asks.iter().next();

        // If both best bid and best ask exist, return them; otherwise, return None
        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            Some(((bid.0.into_inner(), *bid.1), (ask.0.into_inner(), *ask.1)))
        } else {
            None
        }
    }

    // Function to get the volume at a specific price level in the order book
    pub fn get_volume_at_price(&self, price: f64) -> f64 {
        // Check if the price exists in bids or asks, and return the quantity
        if let Some(&qty) = self.bids.get(&OrderedFloat(price)) {
            qty
        } else if let Some(&qty) = self.asks.get(&OrderedFloat(price)) {
            qty
        } else {
            0.0 // If the price is not found, return 0
        }
    }

    // Function to check if the symbol matches the current order book's symbol
    pub fn is_symbol_same(&self, symbol: &str) -> Result<(), OrderBookError> {
        // If the symbols don't match, return a DifferentSymbol error
        if !self.symbol.eq(&symbol) {
            return Err(OrderBookError::DifferentSymbol(format!(
                "Symbol is different! expected: {}, found: {}",
                self.symbol, symbol
            )));
        }

        Ok(())
    }

    // Function to ensure that updates are sequential by comparing the last_update_id
    pub fn is_update_sequential(&self, last_update_id: u64) -> Result<(), OrderBookError> {
        // If the update ID is outdated, return an UpdateIdOutdated error
        if self.last_update_id >= last_update_id {
            return Err(OrderBookError::UpdateIdOutdated(format!(
                "Skipping outdated update: {}",
                last_update_id
            )));
        }

        Ok(())
    }
}

/// Struct to represent a Book Ticker update (single best bid/ask)
#[derive(Debug)]
pub struct BookTickerUpdate {
    // ID of the last order book update
    last_update_id: u64,

    // Best bid price
    bid_price: f64,

    // Best bid quantity
    bid_qty: f64,

    // Best ask price
    ask_price: f64,

    // Best ask quantity
    ask_qty: f64,
}

impl BookTickerUpdate {
    // Constructor function to create a new BookTickerUpdate
    pub fn new(
        last_update_id: u64,
        bid_price: f64,
        bid_qty: f64,
        ask_price: f64,
        ask_qty: f64,
    ) -> Self {
        Self {
            last_update_id,
            bid_price,
            bid_qty,
            ask_price,
            ask_qty,
        }
    }

    // Function to construct a BookTickerUpdate from a reader (deserialized data)
    pub fn from_reader(reader: BookTickerUpdateReader) -> Result<Self, OrderBookError> {
        // Parse the bid and ask prices/quantities from strings to f64
        let bid_price = parse_f64(&reader.bid_price, "bid_price")?;
        let bid_qty = parse_f64(&reader.bid_qty, "bid_qty")?;
        let ask_price = parse_f64(&reader.ask_price, "ask_price")?;
        let ask_qty = parse_f64(&reader.ask_qty, "ask_qty")?;

        // Return the constructed BookTickerUpdate
        Ok(Self {
            last_update_id: reader.last_update_id,
            bid_price,
            bid_qty,
            ask_price,
            ask_qty,
        })
    }
}

/// Struct to represent a Depth update (multiple bid/ask levels)
#[derive(Debug)]
pub struct DepthUpdate {
    // ID of the last order book update
    last_update_id: u64,

    // List of bid price levels and quantities
    bids: Vec<(f64, f64)>,

    // List of ask price levels and quantities
    asks: Vec<(f64, f64)>,
}

impl DepthUpdate {
    // Constructor function to create a new DepthUpdate
    pub fn new(last_update_id: u64, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> Self {
        Self {
            last_update_id,
            bids,
            asks,
        }
    }

    // Function to construct a DepthUpdate from a reader (deserialized data)
    pub fn from_reader(reader: DepthUpdateReader) -> Self {
        Self {
            last_update_id: reader.last_update_id,
            // Parse bids from strings to f64 tuples
            bids: reader
                .bids
                .into_iter()
                .map(|b| {
                    (
                        b[0].parse().unwrap_or_default(),
                        b[1].parse().unwrap_or_default(),
                    )
                })
                .collect(),
            // Parse asks from strings to f64 tuples
            asks: reader
                .asks
                .into_iter()
                .map(|a| {
                    (
                        a[0].parse().unwrap_or_default(),
                        a[1].parse().unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }
}

/// Struct representing a reader for BookTickerUpdate, used for deserialization from JSON
#[derive(Debug, Deserialize)]
pub struct BookTickerUpdateReader {
    // Last update ID of the order book
    #[serde(rename = "u")]
    pub last_update_id: u64,

    // Trading pair symbol
    #[serde(rename = "s")]
    pub symbol: String,

    // Best bid price (as string for deserialization)
    #[serde(rename = "b")]
    pub bid_price: String,

    // Best bid quantity (as string for deserialization)
    #[serde(rename = "B")]
    pub bid_qty: String,

    // Best ask price (as string for deserialization)
    #[serde(rename = "a")]
    pub ask_price: String,

    // Best ask quantity (as string for deserialization)
    #[serde(rename = "A")]
    pub ask_qty: String,
}

/// Struct representing a reader for DepthUpdate, used for deserialization from JSON
#[derive(Debug, Deserialize)]
pub struct DepthUpdateReader {
    // Last update ID of the order book
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,

    // Bids as arrays of [price, quantity] in strings
    pub bids: Vec<[String; 2]>,

    // Asks as arrays of [price, quantity] in strings
    pub asks: Vec<[String; 2]>,
}
