
#[derive(Debug)]
enum Phase {
    Consolidation,
    LiquidityRun,
    SMR,
    Accumulation,
    Completion,
}

pub struct MMXMStrategy {
    // PD arrays
    support: Option<f64>,
    resistance: Option<f64>,
    is_bullish: bool,

    // market state
    current_phase: Phase,
    volatility: f64,
    last_price: f64,
    last_tick: Instant,

    // snapshot storage for SMR detector
    last_bids: Vec<(f64, f64)>,
    last_asks: Vec<(f64, f64)>,

    // order sim
    open_orders: Vec<OrderRequest>,
    closed_pnl: Vec<f64>,

    // account
    account_balance: f64,

    // HTTP client for klines
    http: Client,
    symbol: String,
    htf_bar: String,
    htf_hours: usize,
}

#[derive(Deserialize)]
struct OkxKlineResponse {
    data: Vec<[String; 6]>, // [ts, open, high, low, close, vol]
}

impl MMXMStrategy {
    pub fn new(
        symbol: impl Into<String>,
        start_balance: f64,
        htf_bar: impl Into<String>,
        htf_hours: usize,
    ) -> Self {
        Self {
            support: None,
            resistance: None,
            is_bullish: true,
            current_phase: Phase::Consolidation,
            volatility: 0.0,
            last_price: 0.0,
            last_tick: Instant::now(),
            last_bids: Vec::new(),
            last_asks: Vec::new(),
            open_orders: Vec::new(),
            closed_pnl: Vec::new(),
            account_balance: start_balance,
            http: Client::new(),
            symbol: symbol.into(),
            htf_bar: htf_bar.into(),
            htf_hours,
        }
    }

    /// Fetch klines & compute support/resistance/SMA
    async fn update_pd_arrays(&mut self) {
        let limit = self.htf_hours * 60 / 
            match &*self.htf_bar {
                "1H" => 60,
                "4H" => 240,
                _    => 20
            };
        let url = format!(
            "https://www.okx.com/api/v5/market/history-candles?instId={}&bar={}&limit={}",
            self.symbol, self.htf_bar, limit
        );

        if let Ok(resp) = self.http.get(&url).send().await
            .and_then(|r| r.json::<OkxKlineResponse>().await)
        {
            let mut highs = Vec::new();
            let mut lows  = Vec::new();
            let mut closes = Vec::new();

            for entry in resp.data.iter().rev() {
                let high: f64 = entry[2].parse().unwrap_or(0.0);
                let low:  f64 = entry[3].parse().unwrap_or(0.0);
                let close:f64 = entry[4].parse().unwrap_or(0.0);
                highs.push(high);
                lows.push(low);
                closes.push(close);
            }

            // simple SMA
            let sma = closes.iter().sum::<f64>() / closes.len() as f64;

            self.support   = lows.iter().cloned().fold(None, |min, x| {
                Some(min.map_or(x, |m| m.min(x)))
            });
            self.resistance= highs.iter().cloned().fold(None, |max, x| {
                Some(max.map_or(x, |m| m.max(x)))
            });
            if let Some(s) = self.support {
                self.is_bullish = closes.last().copied().unwrap_or(0.0) > sma;
                println!(
                    "🗺 PD Arrays S={:.5}, R={:.5}, Bullish={} SMA={:.5}",
                    s, self.resistance.unwrap_or(0.0), self.is_bullish, sma
                );
            }
        }
    }

    /// 90th-percentile threshold
    fn quantile_90(qs: &mut [f64]) -> f64 {
        qs.sort_by(|a,b| a.partial_cmp(b).unwrap());
        let idx = ((qs.len() as f64) * 0.9).floor() as usize;
        qs[idx.min(qs.len()-1)]
    }

    fn detect_liquidity_run(&self, price: f64) -> bool {
        if let Some(s) = self.support {
            // MMBM only when bullish HTF
            if self.is_bullish && price < s {
                let mut bids_qty: Vec<f64> = self.last_bids.iter()
                    .map(|(_,qty)| *qty).collect();
                let thr = Self::quantile_90(&mut bids_qty);
                let sum_liq: f64 = self.last_bids.iter()
                    .filter(|(p,_)| *p <= s)
                    .map(|(_,q)| *q).sum();
                return sum_liq > thr;
            }
        }
        false
    }

    fn detect_smr(&self, price: f64) -> bool {
        if let Some(r) = self.resistance {
            // MMSM only when bearish HTF
            if !self.is_bullish && price > r {
                let mut asks_qty: Vec<f64> = self.last_asks.iter()
                    .map(|(_,qty)| *qty).collect();
                let thr = Self::quantile_90(&mut asks_qty);
                let sum_liq: f64 = self.last_asks.iter()
                    .filter(|(p,_)| *p >= r)
                    .map(|(_,q)| *q).sum();
                return sum_liq > thr;
            }
        }
        false
    }

    fn place_order(&mut self, side: Side, price: f64, size: f64) {
        self.open_orders.push(OrderRequest { side, price, size });
        println!("📤 PLACE {:?} @ {:.5} x {:.5}", side, price, size);
    }

    fn process_fills(&mut self, price: f64) {
        // ... your existing fill logic ...
    }
}

impl Strategy for MMXMStrategy {
    fn on_price_tick(&mut self, price: f64, now: Instant) -> Vec<OrderRequest> {
        self.on_price_tick_internal(price, now);
        // return any orders you want to place
        std::mem::take(&mut self.pending_orders)
    }

    fn on_order_book(&mut self, bids: &OrderBook, asks: &OrderBook) -> Vec<OrderRequest> {
        // if you need book depth to set zones
        self.update_zones(bids, asks);
        Vec::new()
    }

    fn on_order_filled(&mut self, fill: OrderFill) {
        self.process_fill(fill);
    }
}