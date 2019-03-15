use lazy_static::lazy_static;

pub const VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 4;
pub const PAYLOAD_MARKER: u8 = 0xFF;

pub const ACK_TIMEOUT: u64 = 2_000;
pub const ACK_RANDOM_FACTOR: f64 = 1.5;
pub const MAX_RETRANSMIT: u32 = 4;
pub const DEFAULT_LEISURE: u64 = 5_000;
pub const PROBING_RATE: f64 = 1.0;
pub const MAX_LATENCY: u64 = 100_000;
pub const PROCESSING_DELAY: u64 = ACK_TIMEOUT;
pub const MAX_RTT: u64 = (2 * MAX_LATENCY) + PROCESSING_DELAY;

pub fn max_transmit_span(ack_timeout: u64, max_retransmit: u32, ack_random_factor: f64) -> u64 {
    (ack_timeout as f64 * (2u64.pow(max_retransmit) - 1) as f64 * ack_random_factor) as u64
}

pub fn max_transmit_wait(ack_timeout: u64, max_retransmit: u32, ack_random_factor: f64) -> u64 {
    (ack_timeout as f64 * ((2u64.pow(max_retransmit + 1)) - 1) as f64 * ack_random_factor) as u64
}

pub fn exchange_lifetime(max_transmit_span: u64, max_latency: u64, proc_delay: u64) -> u64 {
    max_transmit_span + (2 * max_latency) + proc_delay
}

pub fn non_lifetime(max_transmit_span: u64, max_latency: u64) -> u64 {
    max_transmit_span + max_latency
}

lazy_static! {
    pub static ref MAX_TRANSMIT_SPAN: u64 =
        max_transmit_span(ACK_TIMEOUT, MAX_RETRANSMIT, ACK_RANDOM_FACTOR);
    pub static ref MAX_TRANSMIT_WAIT: u64 =
        max_transmit_wait(ACK_TIMEOUT, MAX_RETRANSMIT, ACK_RANDOM_FACTOR);
    pub static ref EXCHANGE_LIFETIME: u64 =
        exchange_lifetime(*MAX_TRANSMIT_SPAN, MAX_LATENCY, PROCESSING_DELAY);
    pub static ref NON_LIFETIME: u64 = non_lifetime(*MAX_TRANSMIT_SPAN, MAX_LATENCY);
}
