use std::collections::{BTreeMap, VecDeque};
use dashmap::DashMap;
use shared::{EnrichedOrderRequest, MatchEvent, OrderKind, OrderType};

#[derive(Debug)]
pub struct OrderBook {
    pub buy: BTreeMap<f64, VecDeque<EnrichedOrderRequest>>,   // price => orders//sorting must big-low
    pub sell: BTreeMap<f64, VecDeque<EnrichedOrderRequest>>,  // price => orders//low to big
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            buy: BTreeMap::new(),
            sell: BTreeMap::new(),
        }
    }

 pub fn match_order(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> {
        // let mut events = vec![];
     match order.order_type {
    OrderType::Buy => match order.kind {
        OrderKind::Limit => self.match_limit_buy(order),
        OrderKind::Market => self.match_market_buy(order),
    },
    OrderType::Sell => match order.kind {
        OrderKind::Limit => self.match_limit_sell(order),
        OrderKind::Market => self.match_market_sell(order),
    },
}
    }

fn match_limit_buy(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> { ... }
fn match_market_buy(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> { ... }
fn match_limit_sell(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> { ... }
fn match_market_sell(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> { ... }


}

