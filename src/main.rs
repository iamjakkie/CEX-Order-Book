mod models;
mod orderbook;
mod sources;
mod strategy;
mod strategies;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use models::{Instrument, OkxResponse, WsBookPush};
use orderbook::OrderBook;
use serde_json::json;
use tokio::runtime::Runtime;
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

fn main() {
    let mut book = OrderBook::new();
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let url = "wss://ws.okx.com:8443/ws/v5/public";
        let (mut ws_stream, _) = connect_async(url).await.expect("connect failed");

        let msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": "books",
                "instId": "AI16Z-USDT-SWAP"
            }]
        })
        .to_string();

        ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await.unwrap();

        while let Some(msg) = ws_stream.next().await {
            let msg = msg.unwrap();
            if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                if let Ok(parsed) = serde_json::from_str::<WsBookPush>(&txt) {
                    // println!("📥 Received: {:?}", parsed);
                    for data in parsed.data {
                        match parsed.action.as_deref() {
                            Some("snapshot") => book.apply_snapshot(&data),
                            Some("update") => book.apply_update(&data),
                            _ => {}
                        }
                        // println!("📊 Order Book: {:?}", book);
                        if let Some(mid) = book.mid_price() {
                            let spread_bps = 0.0020;
                            let quote_bid = mid * (1.0 - spread_bps / 2.0);
                            let quote_ask = mid * (1.0 + spread_bps / 2.0);
                        
                            println!(
                                "📢 QUOTE: BID {:.5} | ASK {:.5} | MID {:.5}",
                                quote_bid, quote_ask, mid
                            );
                        }
                
                        // 🎯 START ACTING HERE (next step)
                    }
                } else {
                    println!("⚠️ Couldn't parse: {}", txt);
                }
            }
        }
    });
}