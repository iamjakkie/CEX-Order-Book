use std::time::Instant;

use crate::orderbook::OrderBook;

// Reusable order and fill types
#[derive(Debug, Clone, Copy)]
pub enum Side { Buy, Sell }

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub side: Side,
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderFill {
    pub id: uuid::Uuid,
    pub side: Side,
    pub entry_price: f64,
    pub exit_price: f64,
    pub size: f64,
    pub pnl: f64,
}

pub trait Strategy {
    /// Called on every mid‐price update
    fn on_price_tick(&mut self, price: f64, now: Instant) -> Vec<OrderRequest> { Vec::new() }
    /// Called whenever you get a new full or incremental order‐book snapshot
    fn on_order_book(&mut self, bids: &OrderBook, asks: &OrderBook) -> Vec<OrderRequest> { Vec::new() }
    /// Called on every timer tick (e.g. 1s, 5s) if you need periodic work
    fn on_timer(&mut self, now: Instant) -> Vec<OrderRequest> { Vec::new() }
    /// Called whenever an order is filled
    fn on_order_filled(&mut self, fill: OrderFill) {}
}