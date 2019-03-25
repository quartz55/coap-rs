use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone)]
pub struct MidGen(HashMap<IpAddr, u16>);
impl Default for MidGen {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
impl MidGen {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn next(&mut self, source: SocketAddr) -> u16 {
        *self
            .0
            .entry(source.ip())
            .and_modify(|e| *e += 1)
            .or_insert(0)
    }
}