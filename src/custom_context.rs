use log::trace;
use rdkafka::client::ClientContext;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::ConsumerContext;
use rdkafka::consumer::Rebalance;
use rdkafka::error::KafkaResult;
use rdkafka::topic_partition_list::TopicPartitionList;

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        trace!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        trace!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        trace!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
pub type LoggingConsumer = StreamConsumer<CustomContext>;
