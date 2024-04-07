use serde::{Deserialize};
//use serde_json::Result;
use bytes::Bytes;
//use std::{error::Error, fmt};

use prometheus::{CounterVec};
use prometheus::{register_counter_vec};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref GATEWAY_SERDE_ERROR: CounterVec = register_counter_vec!(
        "ruuvi_gateway_serde_error_count",
        "Number of messages which could not be parsed.",
        &["reason"]
    ).unwrap();

    static ref SOURCE_MAC_RE : Regex = Regex::new(".+(([0-9A-Fa-f]{2}:){5}([0-9A-Fa-f]{2}))$").unwrap();
}


#[derive(Deserialize, Debug, Clone)]
pub struct RuuviGatewayMessage {
    pub rssi: i16,
    pub ts : Box<str>,
    pub data : Box<str>,

    #[serde(skip_deserializing)]
    pub mac: String,
}

#[derive(Debug, Clone)]
pub enum GatewayMessageResult {
    Received(RuuviGatewayMessage),
    None(),
}


pub fn parse_gateway_message(bytes : &Bytes, topic : String) -> GatewayMessageResult {
    let str = match std::str::from_utf8(bytes) {
        Ok(str) => str,
        Err(e) => {
            println!("couldn't parse utf8: {:?}", e);
            GATEWAY_SERDE_ERROR.with_label_values(&[&"utf8"]).inc();
            return GatewayMessageResult::None()
        },
    };
    let message: RuuviGatewayMessage = match serde_json::from_str(&str) {
        Ok::<RuuviGatewayMessage, serde_json::Error>(mut message) => {
            message.mac = parse_source_mac(&topic).to_string();
            message
        },
        Err(e) => {
            println!("json read error: {:?}", e);
            GATEWAY_SERDE_ERROR.with_label_values(&[&"json"]).inc();
            return GatewayMessageResult::None()
        },
    };
    println!("Received RuuviGatewayMessage: {:?}", message);

    return GatewayMessageResult::Received(message)
}

fn parse_source_mac(topic : &str) -> &str {
    match SOURCE_MAC_RE.captures(topic).and_then(|cap| {
        cap.get(1).map(|source_mac| source_mac.as_str())
    }) {
        Some(str) => str,
        None => topic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_source_mac_parsing() {

        assert_eq!("11:22:33:44:55:66", parse_source_mac("ruuvi/00:00:00:00:00:00/11:22:33:44:55:66"));
        assert_eq!("11:22:33:44:55:66", parse_source_mac("ruuvi/11:22:33:44:55:66"));
        assert_eq!("11:22:33:44:55:66", parse_source_mac("ruuvi/asdf/asdf/asdf/11:22:33:44:55:66"));
    }
}
