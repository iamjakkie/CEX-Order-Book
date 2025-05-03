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