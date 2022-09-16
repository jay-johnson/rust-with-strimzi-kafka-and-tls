//! # Rust Kafka Publisher and Subscriber Demo with Strimzi Kafka and Client mTLS for encryption in transit
//!
//! ## Sources
//!
//! This crate was built from these awesome repos:
//!
//! - Rust Consumer and Producer examples from [rdkafka](https://github.com/fede1024/rust-rdkafka) with [examples](https://github.com/fede1024/rust-rdkafka/tree/master/examples)
//! - [Using your own CA and TLS Assets with Strimzi](https://github.com/scholzj/strimzi-custom-ca-test)
//!
//! ## Optional - Custom TLS Assets
//!
//! By default the ``./kubernetes/deploy.sh`` script will use the included tls assets in the repo: [./kubernetes/tls](https://github.com/jay-johnson/rust-with-strimzi-kafka-and-tls/tree/main/kubernetes/tls). Before going into production with these, please change these to your own to prevent security issues.
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
//! cargo build --example run-consumer
//! export RUST_BACKTRACE=1
//! export RUST_LOG=info
//! ./target/debug/examples/run-consumer -b $KAFKA_BROKERS -g rust-consumer-testing -t testing
//! ```
//!
//! ### Start Producer
//!
//! ```bash
//! # export KAFKA_BROKERS=cluster-0-broker-0.redten.io:32151,cluster-0-broker-1.redten.io:32152,cluster-0-broker-2.redten.io:32153
//! cargo build --example run-producer
//! export RUST_BACKTRACE=1
//! export RUST_LOG=info
//! ./target/debug/examples/run-producer -b $KAFKA_BROKERS -t testing
//! ```

pub mod consume_and_print;
pub mod custom_context;
pub mod log_utils;
pub mod publish_messages;
