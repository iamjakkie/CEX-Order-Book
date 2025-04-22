use std::collections::BTreeMap;
use ordered_float::OrderedFloat;

use crate::models::BookData;

type Price = OrderedFloat<f64>;
type Size = f64;

#[derive(Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<Price, Size>, // descending
    pub asks: BTreeMap<Price, Size>, // ascending
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn apply_snapshot(&mut self, data: &BookData) {
        self.bids.clear();
        self.asks.clear();

        for level in &data.bids {
            if let Some((p, s)) = Self::parse_price_size(level) {
                self.bids.insert(p, s);
            }
        }

        for level in &data.asks {
            if let Some((p, s)) = Self::parse_price_size(level) {
                self.asks.insert(p, s);
            }
        }
    }

    pub fn apply_update(&mut self, update: &BookData) {
        for [price_str, qty_str, ..] in &update.bids {
            let price = price_str.parse::<f64>().unwrap();
            let qty = qty_str.parse::<f64>().unwrap();
    
            if qty == 0.0 {
                self.bids.remove((&price).into());
            } else {
                self.bids.insert(ordered_float::OrderedFloat(price), qty);
            }
        }
    
        for [price_str, qty_str, ..] in &update.asks {
            let price = price_str.parse::<f64>().unwrap();
            let qty = qty_str.parse::<f64>().unwrap();
    
            if qty == 0.0 {
                self.asks.remove((&price).into());
            } else {
                self.asks.insert(ordered_float::OrderedFloat(price), qty);
            }
        }
    }

    fn parse_price_size(level: &[String; 4]) -> Option<(Price, Size)> {
        let price = level[0].parse::<f64>().ok()?;
        let size = level[1].parse::<f64>().ok()?;
        Some((OrderedFloat(price), size))
    }

    pub fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.iter().rev().next().map(|(p, s)| (p.into_inner(), *s))
    }

    pub fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.iter().next().map(|(p, s)| (p.into_inner(), *s))
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn imbalance(&self) -> Option<f64> {
        let bid_vol: f64 = self.bids.values().take(5).sum();
        let ask_vol: f64 = self.asks.values().take(5).sum();
        if bid_vol + ask_vol == 0.0 {
            None
        } else {
            Some(bid_vol / (bid_vol + ask_vol))
        }
    }
}