use clap::App;
use clap::Arg;
use log::info;

use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;
use rdkafka::util::get_rdkafka_version;

use rust_with_kafka_tls::log_utils::setup_logger;
use rust_with_kafka_tls::publish_messages::publish_messages;

// cargo build --example run-producer && export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/examples/run-producer -b COMMA_DELIMITED_BROKER_LIST -t testing

#[tokio::main]
async fn main() {
    let comma_delimited_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    let matches = App::new("producer example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple command line producer")
        .arg(
            Arg::with_name("brokers")
                .short("b")
                .long("brokers")
                .help("Broker list in kafka format")
                .takes_value(true)
                .default_value(&comma_delimited_brokers),
        )
        .arg(
            Arg::with_name("log-conf")
                .long("log-conf")
                .help("Configure the logging format (example: 'rdkafka=trace')")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("topic")
                .short("t")
                .long("topic")
                .help("Destination topic")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    setup_logger(true, matches.value_of("log-conf"));

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let topic = matches.value_of("topic").unwrap();
    let brokers = matches.value_of("brokers").unwrap();

    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
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
        .create()
        .expect("Producer creation error");

    info!("publishing messag to broker={brokers} topic={topic}");
    publish_messages(&producer, &topic).await;
}
