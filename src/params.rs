pub const VERSION: u8 = 1;
pub const ACK_TIMEOUT: u64 = 2_000;
pub const ACK_RANDOM_FACTOR: f64 = 1.5;
pub const MAX_RETRANSMIT: u32 = 4;
pub const DEFAULT_LEISURE: u64 = 5_000;
pub const PROBING_RATE: f64 = 1.0;
// pub const MAX_TRANSMIT_SPAN: u64 =
//     (ACK_TIMEOUT as f64 * (2u64.pow(MAX_RETRANSMIT) - 1) as f64 * ACK_RANDOM_FACTOR) as u64;
// pub const MAX_TRANSMIT_WAIT: u64 =
//     (ACK_TIMEOUT as f64 * ((2u64.pow(MAX_RETRANSMIT + 1)) - 1) as f64 * ACK_RANDOM_FACTOR) as u64;
pub const MAX_LATENCY: u64 = 100_000;
pub const PROCESSING_DELAY: u64 = ACK_TIMEOUT;
pub const MAX_RTT: u64 = (2 * MAX_LATENCY) + PROCESSING_DELAY;
// pub const EXCHANGE_LIFETIME: u64 = MAX_TRANSMIT_SPAN + (2 * MAX_LATENCY) + PROCESSING_DELAY;
// pub const NON_LIFETIME: u64 = MAX_TRANSMIT_SPAN + MAX_LATENCY;

pub const HEADER_SIZE: usize = 4;
pub const PAYLOAD_MARKER: u8 = 0xFF;
