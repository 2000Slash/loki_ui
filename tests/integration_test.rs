use log::error;
use loki_ui::loki::Loki;

#[tokio::test]
async fn main() {
    let mut loki = Loki::new(String::from("http://localhost:3100"));
    loki.send_message(
        String::from("Test log"),
        String::from("{job=\"a\", instance=\"localhost\"}"),
        None,
    )
    .await;
    loki.send_message(
        String::from("Another one"),
        String::from("{job=\"a\", instance=\"localhost\"}"),
        None,
    )
    .await;
    loki.send_message(
        String::from("A different one"),
        String::from("{job=\"a\", instance=\"someotherhost\"}"),
        None,
    )
    .await;
    /*loki.send_message(String::from("Hello world test message"), String::from("{name=\"nils\", job=\"a\"}"), None).await;*/

    let results = loki
        .query_range("count_over_time({job=\"a\"} [1h])", None, None, None)
        .await
        .unwrap();
    //let results = loki.query_range("{job=\"a\"}", None, None, None).await.unwrap();

    for (counter, result) in results.into_iter().enumerate() {
        error!("Result: {counter}");
        error!("Labels: {:?}", result.labels);
        error!("Values:");
        for value in result.values {
            error!("  {value:?}");
        }
    }
}
