use clap::App;
use clap::Arg;
use log::info;

use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::Consumer;
use rdkafka::util::get_rdkafka_version;

use rust_with_kafka_tls::consume_and_print::consume_and_print;
use rust_with_kafka_tls::custom_context::CustomContext;
use rust_with_kafka_tls::custom_context::LoggingConsumer;
use rust_with_kafka_tls::log_utils::setup_logger;

// cargo build --example run-consumer && export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/examples/run-consumer -b COMMA_DELIMITED_BROKER_LIST -g rust-consumer-testing -t testing

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

    info!(
        "building consumer brokers={brokers} group_id={group_id} topics={:?}",
        topics
    );

    consumer
        .subscribe(&topics)
        .expect("Can't subscribe to specified topics");

    consume_and_print(&consumer).await
}
