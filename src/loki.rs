
use std::time::SystemTime;

use loki_api::{logproto::{StreamAdapter, EntryAdapter, PushRequest}, prost_types::Timestamp, prost};

///
/// A buffer that can be used to encode and compress protobuf messages.
/// This is then used by communications with Loki.
struct Buffer {
    encoded: Vec<u8>,
    snappy: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
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

///
/// A very basic Loki client
pub struct Loki {
    address: String,
    client: reqwest::Client,
    buffer: Buffer
}

impl Loki {
    /// Create a new Loki client with a given address
    pub fn new(address: String) -> Loki {
        let client = reqwest::Client::new();
        Loki {
            address,
            client,
            buffer: Buffer::new()
        }
    }

    /// Send a message with labels to Loki
    /// The labels should be in format: {job="test"}
    pub async fn send_message(&mut self, line: String, labels: String) {
        let stream_adapter = StreamAdapter {
            labels,
            entries: vec![
                EntryAdapter {
                    timestamp: Some(Timestamp {
                        seconds: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
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
            println!("Error sending data to Loki: {}", e);
        } else {
            let response = response.unwrap();
            if response.status() != 204 {
                println!("Error sending data to Loki: {}", response.status());
                println!("Response: {:?}", response.text().await);
            }
        }
    }
}