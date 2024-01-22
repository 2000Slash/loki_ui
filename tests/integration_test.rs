use loki_ui::loki::Loki;

#[tokio::test]
async fn main() {
    color_eyre::install().unwrap();
    let mut loki = Loki::new(String::from("http://localhost:3100"));
    //loki.send_message(String::from("Was geht"), String::from("{job=\"a\"}"), None).await;
    /*loki.send_message(String::from("Hello world test message"), String::from("{name=\"nils\", job=\"a\"}"), None).await;*/

    let results = loki.query_range("count_over_time({job=\"a\"} [1h])", None, None, None).await.unwrap();
    //let results = loki.query_range("{job=\"a\"}", None, None, None).await.unwrap();
    
    for (counter, result) in results.into_iter().enumerate() {
        println!("Result: {counter}");
        println!("Labels: {:?}", result.labels);
        println!("Values:");
        for value in result.values {
            println!("  {value:?}");
        }
    }
}