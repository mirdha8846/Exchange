use std::collections::{BTreeMap, VecDeque};
use dashmap::DashMap;
use shared::{EnrichedOrderRequest, MatchEvent, OrderKind, OrderType};

// Wrapper for f64 that implements Ord
//because Btree me keys ko sortable hona zaruri hai
//and rust me floating number ka comprasion thoda tricky hota h ye direct Ord trait nhi follow krte
//
// Ye ek tuple struct(Price)  hai. Matlab iska sirf ek hi field hai — ek f64 value and we access this value with .0

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Price(pub f64);

impl Eq for Price {}
//implementing Ord on Price ,so this can be comprable
impl Ord for Price {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}
// From trait ek conversion trait h,
//(we are using this for ki agr hme koi f64 mile to use Price strcut me kese convert kre)
impl From<f64> for Price {
    fn from(f: f64) -> Self {
        Price(f)
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub buy: BTreeMap<Price, VecDeque<EnrichedOrderRequest>>,   // price => orders//sorting must big-low
    pub sell: BTreeMap<Price, VecDeque<EnrichedOrderRequest>>,  // price => orders//low to big
}
//orderbook looks like this
/*
buy = {
  103.0 => VecDeque[
      EnrichedOrder { order_id: "O1", qty: 5, ... },
      EnrichedOrder { order_id: "O2", qty: 2, ... }
  ],

  102.0 => VecDeque[
      EnrichedOrder { order_id: "O3", qty: 8, ... }
  ],

  101.0 => VecDeque[
      EnrichedOrder { order_id: "O4", qty: 4, ... }
  ]
}
 */

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

fn match_limit_buy(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> { 
    let mut events = vec![];
    let mut remaining_qty = order.quantity;
    let mut remove_prices = vec![];
    let order_price = Price::from(order.price);

    for (price, queue) in self.sell.iter_mut() {
        if price.0 > order.price {
            break;
        }

        while let Some(mut sell_order) = queue.pop_front() {
            let trade_qty = remaining_qty.min(sell_order.quantity);
            remaining_qty -= trade_qty;
            sell_order.quantity -= trade_qty;
            
            // push into event
            events.push(MatchEvent {
                order_id: order.order_id.clone(),
                matched_with: sell_order.order_id.clone(),
                quantity: trade_qty,
                price: price.0,
                market: order.market.clone(),
                event_type: if remaining_qty == 0 {
                    "full_fill".to_string()
                } else {
                    "partial_fill".to_string()
                },
            });
           //sell order abhi baki h buying qty< selling qty to bache huve jo nhi sell huve vapis queue me
            if sell_order.quantity > 0 {
                queue.push_front(sell_order);
                break;
            }
            //sare buying order full-fill ho gye to loop se bhar aa javo ab kuch nhi krna hme
            if remaining_qty == 0 {
                break;
            }
        }
        // agr given price range ki queue empty ho gyi to order book se uska price bhi remove krdo, 
        //because hmne sab sell kr diya(so un sabhi prices ko array me daal do next step me remove krne ke liye)
        if queue.is_empty() {
            remove_prices.push(*price);
        }
        if remaining_qty == 0 {
            break;
        }
    }
    //sell orderbook se vo prices hta do jinki queue upr empty ho gyi thi
    for price in remove_prices {
        self.sell.remove(&price);
    }
    //and agr ab bhi sare order full-fill nhi huve unko buy orderbook me daal do
    if remaining_qty > 0 {
        let price_level = self.buy.entry(order_price).or_insert_with(VecDeque::new);
        let mut new_order = order.clone();
        new_order.quantity = remaining_qty;
        price_level.push_back(new_order);
    }

    events
}



    fn match_market_buy(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> {
   let mut events = vec![];
    let mut remaining_qty = order.quantity;
    let mut remove_prices = vec![];
    let order_price = Price::from(order.price);

    for (price, queue) in self.sell.iter_mut() {
       //this two line feature add in next version
       // if we want then we can add ki order actual(current sell price)
       // price se 1-2% se jyda manhega ho rha h to buy ko stop there

        while let Some(mut sell_order) = queue.pop_front() {
            let trade_qty = remaining_qty.min(sell_order.quantity);
            remaining_qty -= trade_qty;
            sell_order.quantity -= trade_qty;
            
            // push into event
            events.push(MatchEvent {
                order_id: order.order_id.clone(),
                matched_with: sell_order.order_id.clone(),
                quantity: trade_qty,
                price: price.0,
                market: order.market.clone(),
                event_type: if remaining_qty == 0 {
                    "full_fill".to_string()
                } else {
                    "partial_fill".to_string()
                },
            });
           //sell order abhi baki h buying qty< selling qty to bache huve jo nhi sell huve vapis queue me
            if sell_order.quantity > 0 {
                queue.push_front(sell_order);
                break;
            }
            //sare buying order full-fill ho gye to loop se bhar aa javo ab kuch nhi krna hme
            if remaining_qty == 0 {
                break;
            }
        }
        // agr given price range ki queue empty ho gyi to order book se uska price bhi remove krdo, 
        //because hmne sab sell kr diya(so un sabhi prices ko array me daal do next step me remove krne ke liye)
        if queue.is_empty() {
            remove_prices.push(*price);
        }
        if remaining_qty == 0 {
            break;
        }
    }
    //sell orderbook se vo prices hta do jinki queue upr empty ho gyi thi
    for price in remove_prices {
        self.sell.remove(&price);
    }
    
    //todo-this is wronge maket order kabhi orderbook me nhi jate
    if remaining_qty > 0 {
        // Optionally notify user that not all was filled
        events.push(MatchEvent {
            order_id: order.order_id.clone(),
            matched_with: "".to_string(),
            quantity: 0,
            price: 0.0,
            market: order.market.clone(),
            event_type: "partial_cancelled".to_string(), 
        });
    }

    events
    
    }


    fn match_limit_sell(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> {
        let mut events = vec![];
    let mut remaining_qty = order.quantity;
    let mut remove_prices = vec![];
    let order_price = Price::from(order.price);

     for (price,queue) in self.buy.iter_mut().rev()  {
             if order.price>price.0{
                break;
             }
         while let Some(mut buy_order)=queue.pop_front() {
            let trade_qty=remaining_qty.min(buy_order.quantity);
            remaining_qty-=trade_qty;
            buy_order.quantity-=trade_qty;

              // push into event
            events.push(MatchEvent {
                order_id: order.order_id.clone(),
                matched_with: buy_order.order_id.clone(),
                quantity: trade_qty,
                price: price.0,
                market: order.market.clone(),
                event_type: if remaining_qty == 0 {
                    "full_fill".to_string()
                } else {
                    "partial_fill".to_string()
                },
            });
             
            if buy_order.quantity>0{
                queue.push_front(buy_order);
            }
            if remaining_qty==0{
                break;
            }
         } 


        if queue.is_empty() {
            remove_prices.push(*price);
        }
        if remaining_qty == 0 {
            break;
        }
   }
    for price in remove_prices {
        self.buy.remove(&price);
    }
    //and agr ab bhi sare order full-fill nhi huve unko buy orderbook me daal do
    if remaining_qty > 0 {
        let price_level = self.sell.entry(order_price).or_insert_with(VecDeque::new);
        let mut new_order = order.clone();
        new_order.quantity = remaining_qty;
        price_level.push_back(new_order);
    }
 events
    }

    pub fn match_market_sell(&mut self, order: EnrichedOrderRequest) -> Vec<MatchEvent> {
    let mut events = vec![];
    let mut remaining_qty = order.quantity;
    let mut remove_prices = vec![];

    // Iterate over buy side from highest to lowest price
    for (price, queue) in self.buy.iter_mut().rev() {
        while let Some(mut buy_order) = queue.pop_front() {
            let trade_qty = remaining_qty.min(buy_order.quantity);
            remaining_qty -= trade_qty;
            buy_order.quantity -= trade_qty;

            events.push(MatchEvent {
                order_id: order.order_id.clone(),
                matched_with: buy_order.order_id.clone(),
                quantity: trade_qty,
                price: price.0,
                market: order.market.clone(),
                event_type: if remaining_qty == 0 {
                    "full_fill".to_string()
                } else {
                    "partial_fill".to_string()
                },
            });

            if buy_order.quantity > 0 {
                queue.push_front(buy_order);
                break;
            }

            if remaining_qty == 0 {
                break;
            }
        }

        if queue.is_empty() {
            remove_prices.push(*price);
        }

        if remaining_qty == 0 {
            break;
        }
    }

    for price in remove_prices {
        self.buy.remove(&price);
    }

    
    // Because it's a market order; it never enters the orderbook

    if remaining_qty > 0 {
        // Optionally notify user that not all was filled
        events.push(MatchEvent {
            order_id: order.order_id.clone(),
            matched_with: "".to_string(),
            quantity: 0,
            price: 0.0,
            market: order.market.clone(),
            event_type: "partial_cancelled".to_string(), // optional
        });
    }

    events
}

}

pub struct OrderBookMap {
    pub books: DashMap<String, OrderBook>,
}

impl OrderBookMap {
    pub fn new() -> Self {
        Self {
            books: DashMap::new(),
        }
    }

    pub fn get_or_create(&self, market: &str) -> dashmap::mapref::one::RefMut<String, OrderBook> {
        self.books.entry(market.to_string()).or_insert_with(OrderBook::new)
    }
}

