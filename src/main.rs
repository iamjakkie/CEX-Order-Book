use websocket::ClientBuilder;

fn main() {
    let ws_url = "wss://ws.okx.com:8443/ws/v5/business";

    let client = ClientBuilder::new(ws_url)
    .unwrap()
    .connect_insecure()
    .unwrap();

    
}
