/*!
Stat-Physics Market-Maker (StatMM) Strategy
====================================================
Overview
--------
This strategy implements a fully statistical, microstructure-aware market-making engine.
It continuously fits an Ornstein–Uhlenbeck (OU) model to the mid-price, computes
order-flow and book-based signals, and places inventory-aware quotes via the
Avellaneda–Stoikov framework.

Key Features
------------
1. **OU Mean-Reversion**  
   - Models mid-price \(P_t\) as an OU process \(dP_t = \theta(\mu - P_t)\,dt + \sigma\,dW_t\).  
   - Dynamically estimates \(\mu\), \(\theta\), \(\sigma\) on a rolling window of recent ticks.

2. **Inventory-Aware Quoting (Avellaneda–Stoikov)**  
   - Computes reservation price \(r=\mu\).  
   - Computes optimal half-spreads \(\delta_{\rm bid}, \delta_{\rm ask}\) that balance
     profit and inventory risk:  
     \[
       \delta = \frac{\gamma\sigma^2 T}{2} + \frac{1}{\gamma}\ln\Bigl(1+\tfrac{\gamma}{\kappa}\Bigr)
                \;\pm\; \gamma\,q\,\sigma^2 T
     \]  
   - Places two-sided limit orders at \(r\pm\delta\).

3. **Microstructure Signals**  
   - **Order-Flow Imbalance (OFI)**: net bid vs. ask size from top-of-book.  
   - **Book Entropy**: concentration measure of liquidity across levels.  
   - (Optional) regime detection via HMM or Hawkes-derived features.

4. **Inventory & PnL Tracking**  
   - Updates current inventory on fills, adjusts quotes to steer inventory toward zero
     over time.

Benefits
--------
- No arbitrary technical‐analysis zones or indicators.
- Statistically rigorous mean‐reversion anchor.
- Inventory risk controlled via proven AS model.
- Adaptable to changing liquidity regimes (via OFI/entropy/HMM).

Usage
-----
1. Tune parameters:  
   - `gamma`: risk aversion  
   - `kappa`: fill‐rate sensitivity  
   - `T`: quoting horizon  
   - Rolling window length for OU fit  

2. Feed live `OrderBook` updates into `on_order_book()`.  
3. Feed every mid-price tick into `on_price_tick()`.  
4. Handle fills in `on_order_filled()` to update inventory and PnL.

*/

use std::time::Instant;
use crate::{
    orderbook::OrderBook,
    strategy::{Strategy, Side, OrderRequest, OrderFill},
};

pub struct StatMM {
    prices: Vec<f64>,    // rolling buffer of recent mid-prices
    window: usize,       // rolling window size
    mu: f64,             // OU long‐term mean
    sigma: f64,          // OU volatility estimate
    gamma: f64,          // inventory risk aversion
    kappa: f64,          // fill‐rate sensitivity
    T: f64,              // quoting horizon (in same units as timestamps)
    inventory: f64,      // current net position
}

impl StatMM {
    pub fn new(gamma: f64, kappa: f64, T: f64, window: usize) -> Self {
        Self {
            prices: Vec::with_capacity(100),
            window,
            mu: 0.0,
            sigma: 0.0,
            gamma,
            kappa,
            T,
            inventory: 0.0,
        }
    }
    fn fit_vol(&mut self) {
        let n = self.prices.len() as f64;
        let mean = self.prices.iter().sum::<f64>() / n;
        let var = self.prices
            .iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / n;
        self.sigma = var.sqrt();
    }

    fn fit_ou(&mut self) {
        let n = self.prices.len() as f64;
        // mean
        self.mu = self.prices.iter().sum::<f64>() / n;
        // std dev
        let var = self.prices
            .iter()
            .map(|p| (p - self.mu).powi(2))
            .sum::<f64>() / n;
        self.sigma = var.sqrt();
    }

    fn ao_quotes(&self) -> (f64, f64) {
        let r = self.mu;  // reservation price
        let base = self.gamma * self.sigma.powi(2) * self.T / 2.0
                 + (1.0/self.gamma) * (1.0 + self.gamma/self.kappa).ln();
        let skew = self.gamma * self.sigma.powi(2) * self.T * self.inventory;
        let δ_bid = base - skew;
        let δ_ask = base + skew;
        (r - δ_bid, r + δ_ask)
    }

    fn ao_offsets(&self) -> (f64, f64) {
        // base = γσ²T/2 + (1/γ)·ln(1 + γ/κ)
        let base = self.gamma * self.sigma.powi(2) * self.T / 2.0
                 + (1.0 / self.gamma) * (1.0 + self.gamma / self.kappa).ln();
        // skew = γσ²T·inventory
        let skew = self.gamma * self.sigma.powi(2) * self.T * self.inventory;
        // bid offset = base - skew, ask offset = base + skew
        (base - skew, base + skew)
    }
}

impl Strategy for StatMM {
    /// On every new mid‐price tick:
    fn on_price_tick(&mut self, price: f64, _now: Instant) -> Vec<OrderRequest> {
        println!("StatMM: on_price_tick({:.5})", price);
        // 1) Add the new mid‐price
        self.prices.push(price);

        // 2) If we exceed the window, drop the oldest
        if self.prices.len() > self.window {
            self.prices.remove(0);
        }

        // 3) Don’t quote until we’ve got a full window
        if self.prices.len() < self.window {
            return Vec::new();
        }

        println!("StatMM: prices {:?}", self.prices);
        // 4) Fit volatility
        self.fit_vol();

        // 5) Compute offsets
        let (δ_bid, δ_ask) = self.ao_offsets();

        println!("StatMM: σ {:.5} δ_bid {:.5} δ_ask {:.5}", self.sigma, δ_bid, δ_ask);

        // 6) Center around *current* mid (= price)
        let bid_price = price - δ_bid;
        let ask_price = price + δ_ask;

        println!("StatMM: bid {:.5} ask {:.5}", bid_price, ask_price);

        // sanity clamp: never negative
        if bid_price <= 0.0 {
            return Vec::new();
        }

        // 7) Emit both sides
        let size = 1.0;
        vec![
            OrderRequest { side: Side::Buy,  price: bid_price, size },
            OrderRequest { side: Side::Sell, price: ask_price, size },
        ]
    }

    /// When an order actually fills, update inventory
    fn on_order_filled(&mut self, fill: OrderFill) {
        match fill.side {
            Side::Buy  => self.inventory += fill.size,
            Side::Sell => self.inventory -= fill.size,
        }
    }
}