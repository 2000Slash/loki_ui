



use loki_ui::loki::Loki;



#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();
    let mut loki = Loki::new(String::from("http://localhost:3100"));
    loki.send_message(String::from("Hello world test message"), String::from("{job=\"a\"}"), None).await;
    loki.send_message(String::from("Hello world test message"), String::from("{name=\"nils\", job=\"b\"}"), None).await;

    let labels = loki.labels(None, None).await;
    for label in labels.unwrap() {
        println!("{}", label);
    }

    let label_values = loki.label_values("job", None, None, Some("{name=\"nils\"}")).await;
    for label in label_values.unwrap() {
        println!("{}", label);
    }
}