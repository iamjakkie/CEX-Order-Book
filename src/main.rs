mod models;
mod orderbook;
mod sources;
mod strategy;
mod strategies;

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use models::{Instrument, OkxResponse, WsBookPush};
use orderbook::OrderBook;
use serde_json::json;
use strategies::statmm::StatMM;
use strategy::Strategy;
use tokio::{runtime::Runtime, time};
use tokio_tungstenite::connect_async;
use websocket::ClientBuilder;
use futures_util::{SinkExt, StreamExt};


fn fetch_instruments() -> Vec<Instrument> {
    let mut instruments = vec![];
    let client = reqwest::blocking::Client::new();
    let mut req = client.get("https://www.okx.com/api/v5/public/instruments");

    for inst_type in ["SPOT", "SWAP"].iter() {
        let req_with_query = req.try_clone().expect("Failed to clone request").query(&[("instType", inst_type)]);

        let resp = req_with_query.send().unwrap();
        if resp.status().is_success() {
            let parsed: OkxResponse<Instrument> = resp.json().unwrap();
            instruments.extend(parsed.data);
        } else {
            eprintln!("❌ Failed to fetch instruments: {}", resp.status());
        }
    }
    instruments
}

// #[tokio::main]
// async fn main() {

//     let instruments = fetch_instruments();

//     let ai16z_matches = instruments.iter()
//         .filter(|inst| inst.instId.as_ref().map_or(false, |id| id.contains("AI16Z")))
//         .collect::<Vec<_>>();

//     let ai16z = ai16z_matches.get(0).expect("No AI16Z instrument found");


//     sources::okx::subscribe_to_books(
//         &ai16z.instId.as_ref().unwrap(),
//         "books",
//     ).await;

//     loop {

//     }

// }

fn calc_latency(ts_str: &str) -> Option<Duration> {
    if let Ok(ts_millis) = ts_str.parse::<u128>() {
        let book_time = UNIX_EPOCH + Duration::from_millis(ts_millis as u64);
        let now = SystemTime::now();

        now.duration_since(book_time).ok()
    } else {
        None
    }
}
#[tokio::main]
async fn main() {
    // ─── 1) Instantiate your strategy ────────────────────────────────────────
    // gamma=0.1, kappa=1.0, T=1.0 are hyperparameters for the Avellaneda-Stoikov model
    let mut strat = StatMM::new(0.1, 100.0, 1.0, 50);

    // ─── 2) Prepare an OrderBook to accumulate snapshots & updates ──────────
    let mut book = OrderBook::new();

    // ─── 3) Open a WebSocket to OKX ────────────────────────────────────────
    let url = "wss://ws.okx.com:8443/ws/v5/public";
    let (mut ws_stream, _) = connect_async(url).await.expect("WS connect failed");

    // ─── 4) Send the subscribe message ─────────────────────────────────────
    let subscribe = serde_json::json!({
        "op": "subscribe",
        "args": [{ "channel": "books", "instId": "AI16Z-USDT-SWAP" }]
    })
    .to_string();
    ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(subscribe.into())).await.unwrap();

    // ─── 5) Timer for on_timer hooks (e.g. periodic PnL checks) ───────────
    let mut ticker = time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            // ─── 6a) Incoming WS messages ───────────────────────────────────
            msg = ws_stream.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => { eprintln!("WS error: {}", e); continue; },
                    None => break, // connection closed
                };

                if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                    // 6a.i) Parse into your push struct
                    if let Ok(parsed) = serde_json::from_str::<WsBookPush>(&txt) {
                        // 6a.ii) Apply snapshot or update
                        for data in parsed.data {
                            match parsed.action.as_deref() {
                                Some("snapshot") => book.apply_snapshot(&data),
                                Some("update")   => book.apply_update(&data),
                                _ => {}
                            }
                        }

                        // 6a.iii) Compute mid‐price = (best_bid + best_ask)/2
                        if let (Some((bid, _)), Some((ask, _))) = (book.best_bid(), book.best_ask()) {
                            let mid = (bid + ask) / 2.0;
                            let now = Instant::now();


                            // 6a.iv) Strategy: on_price_tick
                            for req in strat.on_price_tick(mid, now) {
                                println!("▶️  OrderRequest from on_price_tick: {:?}", req);
                                // → here you'd actually send the order to OKX
                            }

                            // 6a.v) Strategy: on_order_book (if you need raw‐book signals)
                            for req in strat.on_order_book(&book, &book) {
                                println!("▶️  OrderRequest from on_order_book: {:?}", req);
                            }
                        }
                    } else {
                        eprintln!("⚠️ Couldn't parse WS message: {}", txt);
                    }
                }
            }

            // ─── 6b) Timer event for on_timer ────────────────────────────────
            _ = ticker.tick() => {
                let now = Instant::now();
                for req in strat.on_timer(now) {
                    println!("⏲️  OrderRequest from on_timer: {:?}", req);
                }
            }
        }
    }
}
// fn main() {
//     let mut strat = StatMM::new(0.1, 1.0, 1.0);
    
//     let mut book = OrderBook::new();
//     let rt = Runtime::new().unwrap();
//     rt.block_on(async {
//         let url = "wss://ws.okx.com:8443/ws/v5/public";
//         let (mut ws_stream, _) = connect_async(url).await.expect("connect failed");

//         let msg = json!({
//             "op": "subscribe",
//             "args": [{
//                 "channel": "books",
//                 "instId": "AI16Z-USDT-SWAP"
//             }]
//         })
//         .to_string();

//         ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await.unwrap();

//         while let Some(msg) = ws_stream.next().await {
//             let msg = msg.unwrap();
//             if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
//                 if let Ok(parsed) = serde_json::from_str::<WsBookPush>(&txt) {
//                     // println!("📥 Received: {:?}", parsed);
//                     for data in parsed.data {
//                         match parsed.action.as_deref() {
//                             Some("snapshot") => book.apply_snapshot(&data),
//                             Some("update") => book.apply_update(&data),
//                             _ => {}
//                         }
//                         if let (Some((bid, _)), Some((ask, _))) = (book.best_bid(), book.best_ask()) {
//                             let mid = (bid + ask) / 2.0;
//                             let now = Instant::now();

//                             // 6a.iv) Strategy: on_price_tick
//                             for req in strat.on_price_tick(mid, now) {
//                                 println!("▶️  OrderRequest from on_price_tick: {:?}", req);
//                                 // → here you'd actually send the order to OKX
//                             }

//                         }
                
//                         // 🎯 START ACTING HERE (next step)
//                     }
//                 } else {
//                     println!("⚠️ Couldn't parse: {}", txt);
//                 }
//             }
//         }
//     });
// }