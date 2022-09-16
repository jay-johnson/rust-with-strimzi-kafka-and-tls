use log::info;
use std::time::Duration;

use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;

/// publish_messages
///
/// Publish messages to a kafka ``topic_name`` with an initialized
/// [`rdkafka::producer::FutureProducer`](rdkafka::producer::FutureProducer)
/// using client tls assets based off environment variables.
///
/// ## Optional - Set TLS Asset Paths
///
/// You can either use the included tls assets within the github repo's
/// [rust-with-strimzi-kafka-and-tls/kubernetes/tls](https://github.com/jay-johnson/
/// rust-with-strimzi-kafka-and-tls/tree/main/kubernetes/tls)
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
/// * `producer` - initialized
/// [`rdkafka::producer::FutureProducer`](rdkafka::producer::FutureProducer)
/// that will publish messages to the kafka ``topic_name``
/// * `topic_name` - publish messages this kafka topic
///
/// # Examples
///
/// ```bash
/// # export KAFKA_TLS_CLIENT_CA=./kubernetes/tls/ca.pem
/// # export KAFKA_TLS_CLIENT_KEY=./kubernetes/tls/client.pem
/// # export KAFKA_TLS_CLIENT_CERT=./kubernetes/tls/client-key.pem
/// # export KAFKA_BROKERS=fqdn1:port,fqdn2:port,fqdn3:port
/// cargo build --example run-producer
/// export RUST_BACKTRACE=1
/// export RUST_LOG=info
/// ./target/debug/examples/run-producer -b $KAFKA_BROKERS -t testing
/// ```
///
pub async fn publish_messages(producer: &FutureProducer, topic_name: &str) {
    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .payload(&format!("Message {}", i))
                        .key(&format!("Key {}", i))
                        .headers(
                            OwnedHeaders::new()
                                .add("header_key", "header_value"),
                        ),
                    Duration::from_secs(0),
                )
                .await;

            // This will be executed when the result is received.
            info!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        info!("Future completed. Result: {:?}", future.await);
    }
}
