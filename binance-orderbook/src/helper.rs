use super::*;

/// Function to display the best bid and ask prices from the orderbook
pub fn display_best_bid_ask<F, T>(orderbook: &T, extract_fn: F)
where
    // `extract_fn` is a closure that takes a reference to the orderbook (`&T`)
    // and returns an `Option` with a tuple of ((best_bid_price, best_bid_volume), (best_ask_price, best_ask_volume))
    F: Fn(&T) -> Option<((f64, f64), (f64, f64))>,
{
    // Call the provided extraction function `extract_fn` to get the best bid and ask
    let msg = if let Some((best_bid, best_ask)) = extract_fn(orderbook) {
        // If bid/ask values are found, format the message with the bid and ask prices
        format!("Best Bid: {:?}, Best Ask: {:?}\n\n", best_bid, best_ask)
    } else {
        // If no bid/ask values are found, return a message indicating the orderbook is empty
        "Orderbook is empty.".to_string()
    };

    // Print the message with a purple color using the `.purple()` styling method
    println!("{}", msg.purple())
}

/// Function to parse a string into a `f64` and handle parsing errors
pub fn parse_f64(value: &str, name: &str) -> Result<f64, OrderBookError> {
    // Attempt to parse the input string `value` into a `f64` (floating-point number)
    match value.parse::<f64>() {
        // If parsing succeeds, return the parsed value
        Ok(val) => Ok(val),
        // If parsing fails, return an error wrapped in a custom `OrderBookError`
        Err(e) => {
            return Err(OrderBookError::ParseError(format!(
                // Error message includes the name of the field and the specific parsing error
                "Error parsing {}: {}",
                name, e
            )));
        }
    }
}
