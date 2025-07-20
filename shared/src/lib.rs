use serde::{Deserialize,Serialize};

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq,)]
pub struct EnrichedOrderRequest{
    pub user_id:String,
    pub order_id:String,
    pub kind: OrderKind,      // limit or market
    pub order_type: OrderType, // buy or sell
    pub price: f64,
    pub quantity: u64,
    pub market: MarketType,
}

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum OrderKind {
    Limit,
    Market
}
#[derive(Serialize,Deserialize,Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum OrderType {
    Buy,
   Sell
}
#[derive(Serialize,Deserialize,Debug,Clone,PartialEq,Eq,Hash)]
pub enum MarketType {
    TATA_INR,
    JIO_INR
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchEvent {
    pub order_id: String,
    pub user_id:String,
    pub matched_with: String,
    pub quantity: u64,
    pub price: f64,
    pub order_kind:OrderKind ,//market or limit
    pub market: MarketType,
    pub event_type: EventType, //  FullFill,PartialFill, MarketPartialFill
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum EventType {
    FullFill,
    PartialFill,
    MarketPartialFill
}

#[derive(Serialize,Deserialize)]
pub enum IncomingMarketType {
    SubscribeOrderbook(MarketType)
}