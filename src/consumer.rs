//! # Rust Kafka Publisher and Subscriber Demo with Strimzi Kafka and Client mTLS for encryption in transit
//!
//! ## Sources
//!
//! This crate was built from these awesome repos:
//!
//! - Rust Consumer and Producer examples from [rdkafka](https://github.com/fede1024/rust-rdkafka) with examples: https://github.com/fede1024/rust-rdkafka/tree/master/examples
//! - Using your own CA and TLS Assets with Strimzi: https://github.com/scholzj/strimzi-custom-ca-test
//!
//! ## Optional - Custom TLS Assets
//!
//! By default the ``./kubernetes/deploy.sh`` script will use the included tls assets in the repo: [./kubernetes/tls](https://github.com/jay-johnson/
//! rust-with-strimzi-kafka-and-tls/tree/main/kubernetes/tls). Before going into production with these, please change these to your own to prevent security issues.
//!
//! If you want to use your own tls assets you can set these environment variables:
//!
//! - ``CA_FILE`` - path to your Certificate Authority (CA) file
//! - ``CA_KEY_FILE`` - path to your CA key file
//! - ``TLS_CHAIN_FILE`` - path to your tls server chain file (ordered by: cert then CA)
//! - ``TLS_KEY_FILE`` - path to your tls server key file
//!
//! ```bash
//! ./kubernetes/deploy.sh
//! ```
//!
//! ## Verify Client mTLS
//!
//! Clients must provide the tls key, cert and CAfile for establishing a valid mutual tls connection.
//!
//! For local testing you will need to add these entries to your ``/etc/hosts`` or set up a real nameserver for dns:
//!
//! - ``cluster-0-broker-0.redten.io``
//! - ``cluster-0-broker-1.redten.io``
//! - ``cluster-0-broker-2.redten.io``
//!
//! As an example on the local loopback device:
//!
//! ```bash
//! # /etc/hosts
//! 127.0.0.1      cluster-0-broker-0.redten.io cluster-0-broker-1.redten.io cluster-0-broker-2.redten.io
//! ```
//!
//! For users on minikube you can use ``minikube ip -p CLUSTERNAME`` to get the ip address:
//!
//! ```bash
//! # /etc/hosts
//! 192.168.49.2   cluster-0-broker-0.redten.io cluster-0-broker-1.redten.io cluster-0-broker-2.redten.io
//! ```
//!
//! ```bash
//! echo "ssl test" | openssl s_client -connect \
//!     cluster-0-broker-0.redten.io:32151 \
//!     -key ./kubernetes/tls/client-key.pem \
//!     -cert ./kubernetes/tls/client.pem \
//!     -CAfile ./kubernetes/tls/ca.pem \
//!     -verify_return_error \
//!     && echo "strimzi kafka cluster is working with self-signed tls assets!"
//! ```
//!
//! ## Create Kafka Topic for Rust Messaging
//!
//! ```bash
//! cat <<EOL | kubectl apply -n dev -f -
//! apiVersion: kafka.strimzi.io/v1beta2
//! kind: KafkaTopic
//! metadata:
//!   name: testing
//!   labels:
//!     strimzi.io/cluster: "dev"
//! spec:
//!   partitions: 3
//!   replicas: 3
//! EOL
//! ```
//!
//! ## Rust Messaging
//!
//! ### Set TLS Paths
//!
//! You can either copy the TLS assets into the ``./tls`` directory or export the environment variables:
//!
//! - ``KAFKA_TLS_CLIENT_CA`` - path to the Certificate Authority file
//! - ``KAFKA_TLS_CLIENT_KEY`` - path to the server key file
//! - ``KAFKA_TLS_CLIENT_CERT`` - path to the server certificate file
//!
//! ### Set Broker Addresses
//!
//! Export this environment variable to the correct broker fqdns and ports:
//!
//! - ``KAFKA_BROKERS`` - comma delimited list of kafka brokers (format: ``cluster-0-broker-0.redten.io:32151,cluster-0-broker-1.redten.io:32152,cluster-0-broker-2.redten.io:32153``)
//!
//! ### Start Consumer
//!
//! ```bash
//! # export KAFKA_BROKERS=cluster-0-broker-0.redten.io:32151,cluster-0-broker-1.redten.io:32152,cluster-0-broker-2.redten.io:32153
//! cargo build --bin run-consumer
//! export RUST_BACKTRACE=1
//! export RUST_LOG=info
//! ./target/debug/run-consumer --brokers $KAFKA_BROKERS -g rust-consumer-testing --topics testing
//! ```
//!
//! ### Start Producer
//!
//! ```bash
//! # export KAFKA_BROKERS=cluster-0-broker-0.redten.io:32151,cluster-0-broker-1.redten.io:32152,cluster-0-broker-2.redten.io:32153
//! cargo build --bin run-producer
//! export RUST_BACKTRACE=1
//! export RUST_LOG=info
//! ./target/debug/run-producer --brokers $KAFKA_BROKERS --topic testing
//! ```
//!
use clap::{App, Arg};
use log::{info, warn};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

use crate::example_utils::setup_logger;

mod example_utils;

// cargo build --bin run-consumer && export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/run-consumer --brokers COMMA_DELIMITED_BROKER_LIST -g rust-consumer --topics testing

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        info!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

/// consume_and_print
///
/// Create a rust kafka consumer client with ``rust-rdkafka`` and include paths to
/// client tls assets based off environment variables. This consumer will connect
/// to the ``brokers`` addresses (format: ``fqdn1:port,fqdn2:port,fqdn3:port``)
/// with the ``group_id`` and subscribed to messages stored in the kafka ``topics`` list
///
/// # Optional - Set TLS Asset Paths
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
/// * `brokers` - comma-delimited list of kafka brokers format: ``fqdn1:port,fqdn2:port,fqdn3:port``
/// * `group_id` - custom group identifier for the consumer client
/// * `topics` - subscribe to these topics for consuming messages from the kafka ``brokers``
///
/// # Examples
///
/// ```bash
/// # export KAFKA_TLS_CLIENT_CA=./kubernetes/tls/ca.pem
/// # export KAFKA_TLS_CLIENT_KEY=./kubernetes/tls/client.pem
/// # export KAFKA_TLS_CLIENT_CERT=./kubernetes/tls/client-key.pem
/// # export KAFKA_BROKERS=fqdn1:port,fqdn2:port,fqdn3:port
/// cargo build --bin run-consumer
/// export RUST_BACKTRACE=1
/// export RUST_LOG=info
/// ./target/debug/run-consumer --brokers $KAFKA_BROKERS -g rust-consumer-testing --topics testing
/// ```
///
async fn consume_and_print(brokers: &str, group_id: &str, topics: &[&str]) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "SSL")
        .set(
            "ssl.ca.location",
            std::env::var("KAFKA_TLS_CLIENT_CA")
                .unwrap_or_else(|_| "./kubernetes/tls/ca.pem".to_string()),
        )
        .set(
            "ssl.key.location",
            std::env::var("KAFKA_TLS_CLIENT_KEY").unwrap_or_else(|_| {
                "./kubernetes/tls/client-key.pem".to_string()
            }),
        )
        .set(
            "ssl.certificate.location",
            std::env::var("KAFKA_TLS_CLIENT_CERT")
                .unwrap_or_else(|_| "./kubernetes/tls/client.pem".to_string()),
        )
        .set("enable.ssl.certificate.verification", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(topics)
        .expect("Can't subscribe to specified topics");

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
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        info!("  Header {:#?}: {:?}", header.0, header.1);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let comma_delimited_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    let matches = App::new("consumer example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple command line consumer")
        .arg(
            Arg::with_name("brokers")
                .short("b")
                .long("brokers")
                .help("Broker list in kafka format")
                .takes_value(true)
                .default_value(&comma_delimited_brokers),
        )
        .arg(
            Arg::with_name("group-id")
                .short("g")
                .long("group-id")
                .help("Consumer group id")
                .takes_value(true)
                .default_value("example_consumer_group_id"),
        )
        .arg(
            Arg::with_name("log-conf")
                .long("log-conf")
                .help("Configure the logging format (example: 'rdkafka=trace')")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("topics")
                .short("t")
                .long("topics")
                .help("Topic list")
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .get_matches();

    setup_logger(true, matches.value_of("log-conf"));

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let topics = matches.values_of("topics").unwrap().collect::<Vec<&str>>();
    let brokers = matches.value_of("brokers").unwrap();
    let group_id = matches.value_of("group-id").unwrap();

    consume_and_print(brokers, group_id, &topics).await
}
