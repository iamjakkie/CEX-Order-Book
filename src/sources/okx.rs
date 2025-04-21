use tokio_tungstenite::connect_async;
use websocket::{ClientBuilder, OwnedMessage};
use serde_json::json;
use tokio::runtime::Runtime;
use futures_util::{SinkExt, StreamExt};

use crate::models::BookMsg;

pub async fn subscribe_to_books(inst_id: &str, channel: &str) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let url = "wss://ws.okx.com:8443/ws/v5/public";
        let (mut ws_stream, _) = connect_async(url).await.expect("connect failed");

        let msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": "books",
                "instId": "BTC-USDT-SWAP"
            }]
        })
        .to_string();

        ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await.unwrap();

        while let Some(msg) = ws_stream.next().await {
            let msg = msg.unwrap();
            if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                println!("📨 {}", txt);
            }
        }
    });
    // let url = "wss://ws.okx.com:8443/ws/v5/public";
    // let mut client = ClientBuilder::new(url)
    //     .unwrap()
    //     .connect_insecure()
    //     .unwrap();

    // let sub_msg = serde_json::json!({
    //     "op": "subscribe",
    //     "args": [{
    //         "channel": channel,
    //         "instId": inst_id
    //     }]
    // })
    // .to_string();

    // client.send_message(&OwnedMessage::Text(sub_msg)).unwrap();
    // println!("📡 Subscribed to {} | {}", channel, inst_id);

    // loop {
    //     let msg = client.recv_message().unwrap();

    //     if let OwnedMessage::Text(txt) = msg {
    //         if txt.contains("event") {
    //             println!("📥 Sub event: {}", txt);
    //         } else {
    //             if let Ok(parsed) = serde_json::from_str::<BookMsg>(&txt) {
    //                 println!("🔄 {}: {} entries", parsed.arg.instId, parsed.data.len());
    //                 for book in parsed.data {
    //                     println!("  Asks: {:?}", &book.asks[..1.min(book.asks.len())]);
    //                     println!("  Bids: {:?}", &book.bids[..1.min(book.bids.len())]);
    //                 }
    //             } else {
    //                 println!("⚠️ Couldn't parse: {}", txt);
    //             }
    //         }
    //     }
    // }
}