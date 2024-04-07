use prometheus::{GaugeVec, CounterVec};
use prometheus::{register_counter_vec, register_gauge_vec};
use lazy_static::lazy_static;

use crate::ruuvi::parser::{RuuviData, RuuviSink};

lazy_static! {
    static ref IOT_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "iot_temperature",
        "Temperature in Celcius.",
        &["mac"]
    ).unwrap();

    static ref RUUVI_MEASUREMENTS: CounterVec = register_counter_vec!(
        "ruuvi_measurement_count",
        "Number of received ruuvi measurements.",
        &["mac"]
    ).unwrap();
}

pub struct RuuviPrometheusSink {

}

impl RuuviPrometheusSink {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::default::Default for RuuviPrometheusSink {
    fn default() -> Self {
        Self {

        }
    }
}

impl RuuviSink for RuuviPrometheusSink {
    fn sink(&mut self, source_mac : &str, measurement : RuuviData) {
        println!("source_mac: {:?}, measurement: {:?}", source_mac, measurement);
        RUUVI_MEASUREMENTS.with_label_values(&[&source_mac]).inc();
        IOT_TEMPERATURE.with_label_values(&[&source_mac]).set(measurement.temperature as f64);
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_prometheus_sink() {
        let mut measurement = RuuviData::new();
        measurement.temperature = 21.0;

        let mut sink = RuuviPrometheusSink::new();
        sink.sink("11:22:33:44:55:66", measurement);

        assert_eq!(21.0, IOT_TEMPERATURE.with_label_values(&["11:22:33:44:55:66"]).get());
    }
}
