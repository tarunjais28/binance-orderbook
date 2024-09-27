use super::*;

#[derive(Debug, Clone)]
pub struct OrderBook {
    symbol: String,
    last_order_id: u64,
    pub bids: BTreeMap<OrderedFloat<f64>, f64>, // Wrap f64 in OrderedFloat
    pub asks: BTreeMap<OrderedFloat<f64>, f64>, // Wrap f64 in OrderedFloat
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            last_order_id: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn update_book_ticker(&mut self, data: &BookTickerUpdate) {
        // Update best bid
        if data.bid_qty > 0.0 {
            self.bids.insert(OrderedFloat(data.bid_price), data.bid_qty);
        } else {
            self.bids.remove(&OrderedFloat(data.bid_price));
        }

        // Update best ask
        if data.ask_qty > 0.0 {
            self.asks.insert(OrderedFloat(data.ask_price), data.ask_qty);
        } else {
            self.asks.remove(&OrderedFloat(data.ask_price));
        }
    }

    pub fn update_depth(&mut self, data: &DepthUpdate) {
        // Update bids
        for (price, qty) in &data.bids {
            if *qty > 0.0 {
                self.bids.insert(OrderedFloat(*price), *qty);
            } else {
                self.bids.remove(&OrderedFloat(*price));
            }
        }

        // Update asks
        for (price, qty) in &data.asks {
            if *qty > 0.0 {
                self.asks.insert(OrderedFloat(*price), *qty);
            } else {
                self.asks.remove(&OrderedFloat(*price));
            }
        }
    }

    pub fn get_best_bid_ask(&self) -> Option<((f64, f64), (f64, f64))> {
        let best_bid = self.bids.iter().next_back(); // highest bid
        let best_ask = self.asks.iter().next(); // lowest ask

        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            Some(((bid.0.into_inner(), *bid.1), (ask.0.into_inner(), *ask.1)))
        } else {
            None
        }
    }

    pub fn get_volume_at_price(&self, price: f64) -> f64 {
        if let Some(&qty) = self.bids.get(&OrderedFloat(price)) {
            qty
        } else if let Some(&qty) = self.asks.get(&OrderedFloat(price)) {
            qty
        } else {
            0.0
        }
    }

    pub fn is_symbol_same(&self, symbol: &str) -> Result<(), Box<dyn Error>> {
        if !self.symbol.eq(&symbol) {
            eprintln!(
                "Symbol is different! expected: {}, found: {}",
                self.symbol, symbol
            );
        }

        Ok(())
    }

    pub fn is_update_sequential(&self, last_update_id: u64) -> Result<(), Box<dyn Error>> {
        if self.last_order_id >= last_update_id {
            eprintln!("Skipping outdated update: {}", last_update_id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BookTickerUpdate {
    bid_price: f64,
    bid_qty: f64,
    ask_price: f64,
    ask_qty: f64,
}

impl BookTickerUpdate {
    pub fn new(bid_price: f64, bid_qty: f64, ask_price: f64, ask_qty: f64) -> Self {
        Self {
            bid_price,
            bid_qty,
            ask_price,
            ask_qty,
        }
    }

    pub fn from_reader(reader: BookTickerUpdateReader) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            bid_price: reader.bid_price.parse()?,
            bid_qty: reader.bid_qty.parse()?,
            ask_price: reader.ask_price.parse()?,
            ask_qty: reader.ask_qty.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct DepthUpdate {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

impl DepthUpdate {
    pub fn new(bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> Self {
        Self { bids, asks }
    }

    pub fn from_reader(reader: DepthUpdateReader) -> Self {
        Self {
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

#[derive(Debug, Deserialize)]
pub struct BookTickerUpdateReader {
    #[serde(rename = "u")]
    pub last_update_id: u64, // order book updateId

    #[serde(rename = "s")]
    pub symbol: String, // symbol

    #[serde(rename = "b")]
    pub bid_price: String, // best bid price

    #[serde(rename = "B")]
    pub bid_qty: String, // best bid qty

    #[serde(rename = "a")]
    pub ask_price: String, // best ask price

    #[serde(rename = "A")]
    pub ask_qty: String, // best ask qty
}

#[derive(Debug, Deserialize)]
pub struct DepthUpdateReader {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64, // Last update ID
    pub bids: Vec<[String; 2]>, // Price level to be updated
    pub asks: Vec<[String; 2]>, // Price level to be updated
}

pub fn display_best_bid_ask<F, T>(orderbook: &T, extract_fn: F)
where
    F: Fn(&T) -> Option<((f64, f64), (f64, f64))>,
{
    let msg = if let Some((best_bid, best_ask)) = extract_fn(orderbook) {
        format!("Best Bid: {:?}, Best Ask: {:?}\n\n", best_bid, best_ask)
    } else {
        "Orderbook is empty.".to_string()
    };

    println!("{}", msg.purple())
}
