use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<OrderedFloat<f64>, f64>, // Wrap f64 in OrderedFloat
    pub asks: BTreeMap<OrderedFloat<f64>, f64>, // Wrap f64 in OrderedFloat
}

#[derive(Debug)]
pub struct BookTickerUpdate {
    pub best_bid_price: f64,
    pub best_bid_qty: f64,
    pub best_ask_price: f64,
    pub best_ask_qty: f64,
}

#[derive(Debug)]
pub struct DepthUpdate {
    pub last_update_id: u64,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn update_book_ticker(&mut self, data: &BookTickerUpdate) {
        // Update best bid
        if data.best_bid_qty > 0.0 {
            self.bids
                .insert(OrderedFloat(data.best_bid_price), data.best_bid_qty);
        } else {
            self.bids.remove(&OrderedFloat(data.best_bid_price));
        }

        // Update best ask
        if data.best_ask_qty > 0.0 {
            self.asks
                .insert(OrderedFloat(data.best_ask_price), data.best_ask_qty);
        } else {
            self.asks.remove(&OrderedFloat(data.best_ask_price));
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
        let best_bid = self.bids.iter().rev().next(); // highest bid
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
}
