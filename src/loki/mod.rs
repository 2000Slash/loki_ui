
use std::{ops::Add, collections::HashMap};

use chrono::{DateTime, Local, Duration, NaiveDateTime};
use loki_api::{logproto::{StreamAdapter, EntryAdapter, PushRequest}, prost_types::Timestamp, prost};
/// The json types used in rest requests
pub mod types;

use serde_json::{Value, Map};
use types::LokiLabels;

///
/// A buffer that can be used to encode and compress protobuf messages.
/// This is then used by communications with Loki.
struct Buffer {
    encoded: Vec<u8>,
    snappy: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            encoded: Vec::new(),
            snappy: Vec::new(),
        }
    }

    pub fn encode<'a, T: prost::Message>(&'a mut self, message: &T) -> &'a [u8] {
        self.encoded.clear();
        message
            .encode(&mut self.encoded)
            .expect("protobuf encoding is infallible");
        self.compress_encoded()
    }

    fn compress_encoded(&mut self) -> &[u8] {
        self.snappy
            .resize(snap::raw::max_compress_len(self.encoded.len()), 0);
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/clients/pkg/promtail/client/batch.go#L101
        //
        // In the Go code, `snappy.Encode` is used, which corresponds to the
        // snappy block format, and not the snappy stream format. hence
        // `snap::raw` instead of `snap::write` is needed.
        let snappy_len = snap::raw::Encoder::new()
            .compress(&self.encoded, &mut self.snappy)
            .expect("snappy encoding is infallible");
        &self.snappy[..snappy_len]
    }
}

#[derive(Debug)]
pub struct LokiResult {
    pub labels: HashMap<String, String>,
    pub values: Vec<LokiValue>
}

impl LokiResult {
    fn from_json(mut labels: Map<String, Value>, values: Vec<LokiValue>) -> Self {
        let mut labels_new: HashMap<_, _> = HashMap::new();
        let keys = labels.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let (k, v) = labels.remove_entry (&key).unwrap();
            labels_new.insert(k, v.as_str().unwrap().to_owned());
        }
        Self {
            labels: labels_new,
            values
        }
    }
}

#[derive(Debug)]
pub struct LokiValue {
    pub timestamp: DateTime<Local>,
    pub log_line: String
}

impl LokiValue {
    fn from_nano(timestamp: String, log_line: String) -> Option<Self> {
        let timestamp = timestamp.parse::<i64>().expect("Unable to parse timestamp");
        let secs = timestamp / 1_000_000_000;
        let ns = timestamp - (secs * 1_000_000_000);
    
        let dt = NaiveDateTime::from_timestamp_opt(secs, ns as u32);
        //let timestamp = Local.from_local_datetime(&dt.expect("Unable to parse timestamp")).unwrap();
        let timestamp = DateTime::<Local>::from_naive_utc_and_offset(dt.unwrap(), *Local::now().offset());
        Some(Self {
            timestamp,
            log_line
        })
    }

    fn from_sec(timestamp: i64, log_line: String) -> Option<Self> {
        let dt = NaiveDateTime::from_timestamp_opt(timestamp, 0);
        let timestamp = DateTime::<Local>::from_naive_utc_and_offset(dt.unwrap(), *Local::now().offset());
        Some(Self {
            timestamp,
            log_line
        })
    }
}

///
/// A very basic Loki client
pub struct Loki {
    address: String,
    client: reqwest::Client,
    buffer: Buffer
}

impl Loki {
    /// Create a new Loki client with a given address
    #[must_use] pub fn new(address: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            address,
            client,
            buffer: Buffer::new()
        }
    }

    pub async fn query_range(&mut self, query: &str, limit: Option<i64>, start: Option<DateTime<Local>>, end: Option<DateTime<Local>>) -> Option<Vec<LokiResult>> {
        let start = start.unwrap_or(Local::now().add(Duration::hours(-6)));
        let end = end.unwrap_or(Local::now());
        let limit = limit.unwrap_or(100);

        let response = self.client.get(format!("{}/loki/api/v1/query_range", self.address))
            .query(&[("start", start.timestamp()), ("end", end.timestamp()), ("limit", limit)])
            .query(&[("query", query)])
            .send()
            .await;

        if let Err(e) = response {
            println!("Error receiving label values: {e}");
            return None;
        }

        let response = response.unwrap();
        if response.status() != 200 {
            println!("Error sending data to Loki: {}", response.status());
            println!("Response: {:?}", response.text().await);
            return None;
        }
        let text = &response.text().await.unwrap();
        let text: Value = serde_json::from_str(text).unwrap();

        // There are two different types of results in loki
        // matrix and streams. We can read /data/resultType to find out
        let result_type = text.pointer("/data/resultType").unwrap();
        let mut results = Vec::new();
        if result_type == "streams" {
            let streams = text.pointer("/data/result").unwrap().as_array().unwrap();
            for stream in streams {
                let labels: Map<String, Value> = stream["stream"].as_object().unwrap().clone();

                let values = stream["values"].as_array().unwrap();
                let mut values_vec = Vec::new();
                for value in values {
                    let timestamp = value[0].as_str().unwrap().to_owned();
                    let log_line = value[1].as_str().unwrap().to_owned();
                    let result = LokiValue::from_nano(timestamp, log_line);
                    if let Some(result) = result {
                        values_vec.push(result);
                    }
                }

                results.push(LokiResult::from_json(labels, values_vec));
            }
        } else if result_type == "matrix" {
            let streams = text.pointer("/data/result").unwrap().as_array().unwrap();
            for stream in streams {
                let labels: Map<String, Value> = stream["metric"].as_object().unwrap().clone();

                let values = stream["values"].as_array().unwrap();
                let mut values_vec = Vec::new();
                for value in values {
                    let timestamp = value[0].as_i64().unwrap().to_owned();
                    let log_line = value[1].as_str().unwrap().to_owned();
                    let result = LokiValue::from_sec(timestamp, log_line);
                    if let Some(result) = result {
                        values_vec.push(result);
                    }
                }

                results.push(LokiResult::from_json(labels, values_vec));
            }
        } else {
            println!("Unknown result type: {result_type}");
        }
        Some(results)
    }

    /// Retrieve the values for a given label from Loki
    pub async fn label_values(&mut self, label: &str, start: Option<DateTime<Local>>, end: Option<DateTime<Local>>, query: Option<&str>) -> Option<Vec<String>> {
        let start = start.unwrap_or(Local::now().add(Duration::hours(-6)));
        let end = end.unwrap_or(Local::now());

        let response = self.client.get(format!("{}/loki/api/v1/label/{}/values", self.address, label))
            .query(&[("start", start.timestamp()), ("end", end.timestamp())])
            .query(&[("query", query)])
            .send()
            .await;

        if let Err(e) = response {
            println!("Error receiving label values: {e}");
            return None;
        }

        let response = response.unwrap();
        if response.status() != 200 {
            println!("Error sending data to Loki: {}", response.status());
            println!("Response: {:?}", response.text().await);
            return None;
        }

        let text = response.json::<LokiLabels>().await;
        if let Err(e) = text {
            println!("Error parsing labels: {e}");
            return None;
        }

        let labels_response = text.unwrap();
        Some(labels_response.data)
    }

    /// Retrieve the labels from Loki
    pub async fn labels(&mut self, start: Option<DateTime<Local>>, end: Option<DateTime<Local>>) -> Option<Vec<String>> {
        let start = start.unwrap_or(Local::now().add(Duration::hours(-6)));
        let end = end.unwrap_or(Local::now());
        
        let response = self.client.get(format!("{}/loki/api/v1/labels", self.address))
            .query(&[("start", start.timestamp()), ("end", end.timestamp())])
            .send()
            .await;

        if let Err(e) = response {
            println!("Error receiving labels: {e}");
            return None;
        }

        let response = response.unwrap();
        if response.status() != 200 {
            println!("Error sending data to Loki: {}", response.status());
            println!("Response: {:?}", response.text().await);
            return None;
        }

        let text = response.json::<LokiLabels>().await;
        if let Err(e) = text {
            println!("Error parsing labels: {e}");
            return None;
        }

        let labels_response = text.unwrap();
        Some(labels_response.data)
    }

    /// Send a message with labels to Loki
    /// The labels should be in format: {job="test"}
    pub async fn send_message(&mut self, line: String, labels: String, time: Option<DateTime<Local>>) {
        let time = time.unwrap_or(Local::now());
        let stream_adapter = StreamAdapter {
            labels,
            entries: vec![
                EntryAdapter {
                    timestamp: Some(Timestamp {
                        seconds: time.timestamp(),
                        nanos: 0
                    }),
                    line,
                },
            ],
            hash: 0
        };
        self.push(vec![stream_adapter]).await;
    }

    async fn push(&mut self, streams: Vec<StreamAdapter>) {
        let body = &mut self.buffer.encode(&PushRequest {streams}).to_owned();
        let response = self.client.post(format!("{}/loki/api/v1/push", self.address))
            .body(body.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/x-snappy")
            .send()
            .await;

        if let Err(e) = response {
            println!("Error sending data to Loki: {e}");
        } else {
            let response = response.unwrap();
            if response.status() != 204 {
                println!("Error sending data to Loki: {}", response.status());
                println!("Response: {:?}", response.text().await);
            }
        }
    }
}