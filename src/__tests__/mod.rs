#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::engine::models::{order::{Order, OrderId, OrderPrice, OrderQuantity, OrderSide}, orderbook::Orderbook};


    fn new_empty_orderbook() -> Orderbook {
        Orderbook::new()
    }

    fn new_market_order(id: OrderId, side: OrderSide, quantity: OrderQuantity) -> Order {
        Order::new_market(id, 0, 0, 0, side, quantity)
    }

    fn new_limit_order(id: OrderId, side: OrderSide, limit_price: OrderPrice, quantity: OrderQuantity) -> Order {
        Order::new_limit(id, 0, 0, 0, side, limit_price, quantity)
    }

    #[test]
    // Add bid market message to an empty lob. Order expires
    fn order_should_expires_for_add_bid_market_to_empty_orderbook() {
        let mut orderbook = new_empty_orderbook();

        let match_result = orderbook.put_order(new_market_order(0, OrderSide::Bid, Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());
        assert!(orderbook.get_asks_depth().is_empty());
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer market message to an empty lob. Order expires
    fn order_should_expires_for_ask_market_to_empty_orderbook() {
        let mut orderbook = new_empty_orderbook();

        let match_result = orderbook.put_order(new_market_order(0, OrderSide::Ask, Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());
        assert!(orderbook.get_asks_depth().is_empty());
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add bid market message. Order is filled
    fn order_should_filled_for_bid_market() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(80), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Ask, Decimal::from(50), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_market_order(3, OrderSide::Bid, Decimal::from(1000))).unwrap();

        assert_eq!(match_result.trades.len(), 3);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(50));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(80));
        assert_eq!(match_result.trades[2].get_price(), Decimal::from(100));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(200));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(500));
        assert_eq!(match_result.trades[2].get_quantity(), Decimal::from(300));

        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(700)]]);
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer market message. Order is filled
    fn order_should_filled_for_ask_market() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(1000))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(80), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Bid, Decimal::from(50), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_market_order(3, OrderSide::Ask, Decimal::from(1000))).unwrap();

        assert_eq!(match_result.trades.len(), 1);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(100));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(1000));

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(80), Decimal::from(500)], [Decimal::from(50), Decimal::from(200)]]);
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add bid market message. Order is partially filled. Order is expired
    fn order_should_partially_filled_and_expires_for_bid_market() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(80), Decimal::from(500))).unwrap();

        let match_result = orderbook.put_order(new_market_order(2, OrderSide::Bid, Decimal::from(1200))).unwrap();

        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(80));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(100));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(500));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(500));

        assert!(orderbook.get_bids_depth().is_empty());
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add offer market message. Order is partially filled. Order is expired
    fn order_should_partially_filled_and_expires_for_ask_market() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(80), Decimal::from(500))).unwrap();

        let match_result = orderbook.put_order(new_market_order(2, OrderSide::Ask, Decimal::from(1200))).unwrap();

        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(100));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(80));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(500));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(500));

        assert!(orderbook.get_bids_depth().is_empty());
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add bid limit message to an empty lob
    fn order_should_placed_for_bid_limit() {
        let mut orderbook = new_empty_orderbook();

        let match_result = orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(100), Decimal::from(1000)]]);
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add bid limit message with different bid price
    fn order_should_placed_for_bid_limit_with_different_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(200), Decimal::from(500))).unwrap();
    
        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(200), Decimal::from(500)], [Decimal::from(100), Decimal::from(1000)]]);
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add bid limit message with existing bid price
    fn order_should_placed_for_bid_limit_with_existing_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(100), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(100), Decimal::from(2000)]]);
        assert!(orderbook.get_asks_depth().is_empty());
    }

    #[test]
    // Add bid limit message with existing offer price. No Match
    fn order_should_not_matched_for_bid_limit_with_existing_ask_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(50), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(50), Decimal::from(1000)]]);
        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(1000)]]);
    }

    #[test]
    // Add bid limit message with existing offer price. Match
    fn order_should_matched_for_bid_limit_with_existsing_ask_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(80), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Ask, Decimal::from(50), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(3, OrderSide::Bid, Decimal::from(80), Decimal::from(1000))).unwrap();
    
        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(50));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(80));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(200));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(500));

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(80), Decimal::from(300)]]);
        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(1000)]]);
    }

    #[test]
    // Bid LO matches Offer LO. Time-priority determines the matching orders
    fn order_should_matched_for_bid_limit_with_time_priority() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(50), Decimal::from(300))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Ask, Decimal::from(50), Decimal::from(300))).unwrap();
        orderbook.put_order(new_limit_order(3, OrderSide::Ask, Decimal::from(20), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(4, OrderSide::Bid, Decimal::from(50), Decimal::from(500))).unwrap();

        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(20));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(50));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(200));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(300));

        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(50), Decimal::from(300)], [Decimal::from(100), Decimal::from(1000)]]);
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer limit message to an empty lob
    fn order_should_placed_for_ask_limit() {
        let mut orderbook = new_empty_orderbook();

        let match_result = orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(1000)]]);
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer limit message with different offer price
    fn order_should_placed_for_ask_limit_with_different_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(200), Decimal::from(500))).unwrap();
    
        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(1000)], [Decimal::from(200), Decimal::from(500)]]);
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer limit message with existing offer price
    fn order_should_placed_for_ask_limit_with_existing_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(2000)]]);
        assert!(orderbook.get_bids_depth().is_empty());
    }

    #[test]
    // Add offer limit message with existing bid price. No Match
    fn order_should_not_matched_for_ask_limit_with_existing_ask_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(50), Decimal::from(1000))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(1, OrderSide::Ask, Decimal::from(100), Decimal::from(1000))).unwrap();

        assert!(match_result.trades.is_empty());

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(50), Decimal::from(1000)]]);
        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(100), Decimal::from(1000)]]);
    }

    #[test]
    // Add offer limit message with existing bid price. Match
    fn order_should_matched_for_ask_limit_with_existsing_ask_price() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(500))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(80), Decimal::from(200))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Bid, Decimal::from(50), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(3, OrderSide::Ask, Decimal::from(80), Decimal::from(1000))).unwrap();
    
        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(100));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(80));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(500));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(200));

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(50), Decimal::from(200)]]);
        assert_eq!(orderbook.get_asks_depth(), vec![[Decimal::from(80), Decimal::from(300)]]);
    }

    #[test]
    // Offer LO matches Bid LO. Time-priority determines the matching orders
    fn order_should_matched_for_ask_limit_with_time_priority() {
        let mut orderbook = new_empty_orderbook();

        orderbook.put_order(new_limit_order(0, OrderSide::Bid, Decimal::from(100), Decimal::from(700))).unwrap();
        orderbook.put_order(new_limit_order(1, OrderSide::Bid, Decimal::from(50), Decimal::from(300))).unwrap();
        orderbook.put_order(new_limit_order(2, OrderSide::Bid, Decimal::from(50), Decimal::from(300))).unwrap();
        orderbook.put_order(new_limit_order(3, OrderSide::Bid, Decimal::from(20), Decimal::from(200))).unwrap();

        let match_result = orderbook.put_order(new_limit_order(4, OrderSide::Ask, Decimal::from(50), Decimal::from(1000))).unwrap();

        assert_eq!(match_result.trades.len(), 2);

        assert_eq!(match_result.trades[0].get_price(), Decimal::from(100));
        assert_eq!(match_result.trades[1].get_price(), Decimal::from(50));

        assert_eq!(match_result.trades[0].get_quantity(), Decimal::from(700));
        assert_eq!(match_result.trades[1].get_quantity(), Decimal::from(300));

        assert_eq!(orderbook.get_bids_depth(), vec![[Decimal::from(50), Decimal::from(300)], [Decimal::from(20), Decimal::from(200)]]);
        assert!(orderbook.get_asks_depth().is_empty());
    }
}