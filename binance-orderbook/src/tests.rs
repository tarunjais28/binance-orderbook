use ordered_float::OrderedFloat;

use super::*;

#[test]
fn test_update_book_ticker() {
    let mut orderbook = OrderBook::new("BNBUSDT".to_string());
    let book_ticker_update = BookTickerUpdate::new(25.3519, 31.21, 25.3652, 40.66);

    orderbook.update_book_ticker(&book_ticker_update);
    let best_bid_ask = orderbook.get_best_bid_ask().unwrap();
    assert_eq!(best_bid_ask.0, (25.3519, 31.21));
    assert_eq!(best_bid_ask.1, (25.3652, 40.66));
}

#[test]
fn test_update_depth() {
    let mut orderbook = OrderBook::new("BNBUSDT".to_string());
    let depth_update = DepthUpdate::new(vec![(0.0024, 10.0), (0.0025, 5.0)], vec![(0.0026, 100.0)]);

    orderbook.update_depth(&depth_update);
    assert_eq!(orderbook.get_volume_at_price(0.0024), 10.0);
    assert_eq!(orderbook.get_volume_at_price(0.0026), 100.0);
}

#[test]
fn test_get_volume_at_price() {
    let mut orderbook = OrderBook::new("BNBUSDT".to_string());
    orderbook.bids.insert(OrderedFloat(0.0024), 10.0);
    orderbook.asks.insert(OrderedFloat(0.0026), 100.0);

    assert_eq!(orderbook.get_volume_at_price(0.0024), 10.0);
    assert_eq!(orderbook.get_volume_at_price(0.0026), 100.0);
    assert_eq!(orderbook.get_volume_at_price(0.0030), 0.0);
}
