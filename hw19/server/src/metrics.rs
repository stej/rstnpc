use lazy_static::lazy_static;
use prometheus::{IntCounter, Gauge};

lazy_static! {
    pub static ref METRICS_MESSAGES_COUNT_COUNTER: IntCounter = IntCounter::new(
        "chatapp_total_messages_count",
        "Count of messages sent to server."
    ).unwrap();
    pub static ref METRICS_CONNECTED_USERS_GAUGE: Gauge = Gauge::new(
        "chatapp_connected_users_count",
        "Count of users currently connected to server."
    ).unwrap();
}

pub fn messages_up() {
    METRICS_MESSAGES_COUNT_COUNTER.inc();
}
pub fn users_up() {
    METRICS_CONNECTED_USERS_GAUGE.inc();
}
pub fn users_down() {
    METRICS_CONNECTED_USERS_GAUGE.dec();
}

pub fn init() {
    prometheus::default_registry()
        .register(Box::new(METRICS_CONNECTED_USERS_GAUGE.clone()))
        .unwrap();
    prometheus::default_registry()
        .register(Box::new(METRICS_MESSAGES_COUNT_COUNTER.clone()))
        .unwrap();
}