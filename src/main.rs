
use loki_ui::loki::Loki;



#[tokio::main]
async fn main() {
    let mut loki = Loki::new(String::from("http://localhost:3100"));
    loki.send_message(String::from("Hello world test message"), String::from("{job=\"test\"}")).await;
}