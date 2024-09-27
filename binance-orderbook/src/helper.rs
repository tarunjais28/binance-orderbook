use super::*;

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
