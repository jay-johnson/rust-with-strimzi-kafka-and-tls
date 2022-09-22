use log::info;
use log::warn;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::message::Headers;
use rdkafka::message::Message;

use crate::custom_context::LoggingConsumer;

/// consume_and_print
///
/// Consume messages from kafka with an initalized
/// [`rdkafka::consumer::Consumer`](rdkafka::consumer::Consumer)
/// using client tls assets based off environment variables.
///
/// # Optional - Set TLS Asset Paths
///
/// You can either use the included tls assets within the github repo
/// [rust-with-strimzi-kafka-and-tls/kubernetes/tls](https://github.com/jay-johnson/rust-with-strimzi-kafka-and-tls/tree/main/kubernetes/tls)
/// directory or export the environment variables:
///
/// - ``KAFKA_TLS_CLIENT_CA`` - path to the Certificate Authority file
/// - ``KAFKA_TLS_CLIENT_KEY`` - path to the server key file
/// - ``KAFKA_TLS_CLIENT_CERT`` - path to the server certificate file
///
/// # Set Broker Addresses
///
/// Export this environment variable to the correct broker fqdns and ports:
///
/// - ``KAFKA_BROKERS`` - comma delimited list of kafka brokers
///   (format: ``fqdn1:port,fqdn2:port,fqdn3:port``)
///
/// # Arguments
///
/// * `consumer` - initialized
/// [`rdkafka::consumer::Consumer`](rdkafka::consumer::Consumer)
/// that is already subscribed to a list of ``topics`` with a ``group_id``
///
/// # Examples
///
/// ```bash
/// # export KAFKA_TLS_CLIENT_CA=./kubernetes/tls/ca.pem
/// # export KAFKA_TLS_CLIENT_KEY=./kubernetes/tls/client.pem
/// # export KAFKA_TLS_CLIENT_CERT=./kubernetes/tls/client-key.pem
/// # export KAFKA_BROKERS=fqdn1:port,fqdn2:port,fqdn3:port
/// cargo build --example run-consumer
/// export RUST_BACKTRACE=1
/// export RUST_LOG=info
/// ./target/debug/examples/run-consumer -b $KAFKA_BROKERS -g rust-consumer-testing -t testing
/// ```
///
pub async fn consume_and_print(consumer: &LoggingConsumer) {
    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!(
                            "Error while deserializing message payload: {:?}",
                            e
                        );
                        ""
                    }
                };
                let mut header_str = String::from("");
                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        let new_str = format!(
                            "{i}:{:#?}=>{:?}",
                            header.0,
                            // https://doc.rust-lang.org/stable/std/str/fn.from_utf8.html
                            std::str::from_utf8(header.1).unwrap()
                        );
                        if header_str.is_empty() {
                            header_str = new_str;
                        } else {
                            header_str = header_str + ", " + &new_str;
                        }
                    }
                }
                let mut found_key = "";
                if m.key().is_some() {
                    found_key = std::str::from_utf8(m.key().unwrap()).unwrap();
                }
                info!(
                    "key='{}' payload='{}', \
                    topic={} partition={}, \
                    offset={} timestamp={:?} \
                    headers=[{header_str}]",
                    // https://doc.rust-lang.org/stable/std/str/fn.from_utf8.html
                    found_key,
                    payload,
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
