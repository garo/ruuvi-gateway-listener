use std::{num::ParseIntError};

pub struct RuuviData {

    format: u8,
    temperature: f32,
    pressure: u32,
    humidity: f32,
    acceleration_x: f32,
    acceleration_y: f32,
    acceleration_z: f32,
    tx_power: i16,
    voltage: f32,
    movement: u8,
    measurement_sequence: u32,
    mac: [u8; 6],
    // mac
}

impl std::default::Default for RuuviData {
    fn default() -> Self {
        Self {
            format: 0,
            temperature: 0.0,
            pressure: 0,
            humidity: 0.0,
            acceleration_x: 0.0,
            acceleration_y: 0.0,
            acceleration_z: 0.0,
            tx_power: 0,
            voltage: 0.0,
            movement: 0,
            measurement_sequence: 0,
            mac: [0; 6],
        }
    }
}

impl RuuviData {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn ruuvi_decode_v5(buf : &Vec<u8>) -> RuuviData {

    let mut data : RuuviData = RuuviData::new();

    data.format = buf[0] as u8;
    data.temperature = (((buf[1] as u16) << 8) + buf[2] as u16) as i16 as f32 * 0.005;
    data.humidity = (((buf[3] as u16) << 8) + buf[4] as u16) as f32 * 0.0025;
    data.pressure = (((buf[5] as u32) << 8) + buf[6] as u32) + 50000;
    data.acceleration_x = ((((buf[7] as u16) << 8) + buf[8] as u16) as i16) as f32 / 1000.0;
    data.acceleration_y = ((((buf[9] as u16) << 8) + buf[10] as u16) as i16) as f32 / 1000.0;
    data.acceleration_z = ((((buf[11] as u16) << 8) + buf[12] as u16) as i16) as f32 / 1000.0;

    let power_info = ((buf[13] as u16) << 8) + buf[14] as u16;
    data.tx_power = ((power_info & 0b11111) as i16 * 2 - 40) as i16;
    data.voltage = ((power_info >> 5) + 1600) as f32 / 1000.0;
    data.movement = buf[15] as u8;
    data.measurement_sequence = ((buf[16] as u32) << 8) + (buf[17] as u32);

    for x in 0..6 {
        data.mac[x] = buf[18 + x];
    }

    return data;
}

pub fn ruuvi_decode_v3(buf : &Vec<u8>) -> RuuviData {

    let mut data : RuuviData = RuuviData::new();

    data.format = buf[0] as u8;
    data.humidity = (buf[1] as u8) as f32 * 0.5;

    // Temperature base: (MSB is sign, next 7 bits are decimal value)
    // Temperature fraction in 1/100
    let temperature_base = buf[2] as u8 & 0x7F;
    let temperature_fraction = (buf[3] as u8 as f32) / 100.0;
    let mut temperature = temperature_base as f32 + temperature_fraction;
    if (buf[2] >> 7) & 1 == 1 {
        temperature = -temperature;
    }
    data.temperature = temperature;
    
    data.pressure = (((buf[4] as u32) << 8) + buf[5] as u32) + 50000;
    data.acceleration_x = ((((buf[6] as u16) << 8) + buf[7] as u16) as i16) as f32 / 1000.0;
    data.acceleration_y = ((((buf[8] as u16) << 8) + buf[9] as u16) as i16) as f32 / 1000.0;
    data.acceleration_z = ((((buf[10] as u16) << 8) + buf[11] as u16) as i16) as f32 / 1000.0;

    data.voltage = (((buf[12] as u16) << 8) + buf[13] as u16) as f32 / 1000.0;

    return data;
}


#[cfg(test)]
mod tests {
    use super::*;

    // Takes a string such as "AABB" and returns Vec with AA and BB
    fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr, $e:expr) => ({
            let (a, b) = (&$a, &$b);
            assert!((*a - *b).abs() < $e,
                    "{} is not approximately equal to {}", *a, *b);
        })
    }

    #[test]
    fn test_ruuvi_decode_v5() {
        let s = decode_hex("0512FC5394C37C0004FFFC040CAC364200CDCBB8334C884F");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-5-rawv2
            Data format: 5
            Temperature: 24.3 C
            Pressure: 100044
            Humidity: 53.49 RH-%
            Acceleration X: 0.004 G
            Acceleration Y: -0.004 G
            Acceleration Z: 1.036 G
            TX Power: 4 dBm
            Voltage: 2.977 V
            Movement counter: 66
            Measurement Sequence: 205
            MAC: CB B8 33 4C 88 4F
        */

        let data = ruuvi_decode_v5(&s.unwrap());

        assert_eq!(data.format, 5);
        assert_approx_eq!(data.temperature, 24.3, 1e-5);
        assert_approx_eq!(data.humidity, 53.49, 1e-5);
        assert_eq!(data.pressure, 100044);
        assert_approx_eq!(data.acceleration_x, 0.004, 1e-9);
        assert_approx_eq!(data.acceleration_y, -0.004, 1e-9);
        assert_approx_eq!(data.acceleration_z, 1.036, 1e-9);
        assert_eq!(data.tx_power, 4);
        assert_eq!(data.voltage, 2.977);
        assert_eq!(data.movement, 66);
        assert_eq!(data.measurement_sequence, 205);

        assert_eq!(data.mac[0], 0xCB);
        assert_eq!(data.mac[1], 0xB8);
        assert_eq!(data.mac[2], 0x33);
        assert_eq!(data.mac[3], 0x4C);
        assert_eq!(data.mac[4], 0x88);
        assert_eq!(data.mac[5], 0x4F);

    }

    #[test]
    fn test_ruuvi_decode_v5_maximum_values() {
        let s = decode_hex("057FFFFFFEFFFE7FFF7FFF7FFFFFDEFEFFFECBB8334C884F");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-5-rawv2
        */

        let data = ruuvi_decode_v5(&s.unwrap());

        assert_eq!(data.format, 5);
        assert_approx_eq!(data.temperature, 163.835, 1e-4);
        assert_eq!(data.pressure, 115534);
        assert_approx_eq!(data.humidity, 163.8350, 1e-4);
        assert_approx_eq!(data.acceleration_x, 32.767, 1e-4);
        assert_approx_eq!(data.acceleration_y, 32.767, 1e-4);
        assert_approx_eq!(data.acceleration_z, 32.767, 1e-4);
        assert_eq!(data.tx_power, 20);
        assert_eq!(data.voltage, 3.646);
        assert_eq!(data.movement, 254);
        assert_eq!(data.measurement_sequence, 65534);

        assert_eq!(data.mac[0], 0xCB);
        assert_eq!(data.mac[1], 0xB8);
        assert_eq!(data.mac[2], 0x33);
        assert_eq!(data.mac[3], 0x4C);
        assert_eq!(data.mac[4], 0x88);
        assert_eq!(data.mac[5], 0x4F);

    }

    #[test]
    fn test_ruuvi_decode_v5_minimum_values() {
        let s = decode_hex("058001000000008001800180010000000000CBB8334C884F");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-5-rawv2
        */

        let data = ruuvi_decode_v5(&s.unwrap());

        assert_eq!(data.format, 5);
        assert_approx_eq!(data.temperature, -163.835, 1e-4);
        assert_eq!(data.pressure, 50000);
        assert_approx_eq!(data.humidity, 0.0, 1e-4);
        assert_approx_eq!(data.acceleration_x, -32.767, 1e-4);
        assert_approx_eq!(data.acceleration_y, -32.767, 1e-4);
        assert_approx_eq!(data.acceleration_z, -32.767, 1e-4);
        assert_eq!(data.tx_power, -40);
        assert_eq!(data.voltage, 1.6);
        assert_eq!(data.movement, 0);
        assert_eq!(data.measurement_sequence, 0);

        assert_eq!(data.mac[0], 0xCB);
        assert_eq!(data.mac[1], 0xB8);
        assert_eq!(data.mac[2], 0x33);
        assert_eq!(data.mac[3], 0x4C);
        assert_eq!(data.mac[4], 0x88);
        assert_eq!(data.mac[5], 0x4F);

    }

    #[test]
    fn test_ruuvi_decode_v3() {
        let s = decode_hex("03291A1ECE1EFC18F94202CA0B53");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-3-rawv1
        */

        let data = ruuvi_decode_v3(&s.unwrap());

        assert_eq!(data.format, 3);
        assert_approx_eq!(data.humidity, 20.5, 1e-4);
        assert_approx_eq!(data.temperature, 26.3, 1e-4);
        assert_eq!(data.pressure, 102766);
        assert_approx_eq!(data.acceleration_x, -1.0, 1e-4);
        assert_approx_eq!(data.acceleration_y, -1.726, 1e-4);
        assert_approx_eq!(data.acceleration_z, 0.714, 1e-4);
        assert_eq!(data.voltage, 2.899);

    }

    #[test]
    fn test_ruuvi_decode_v3_minimum_values() {
        let s = decode_hex("0300FF6300008001800180010000");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-3-rawv1
        */

        let data = ruuvi_decode_v3(&s.unwrap());

        assert_eq!(data.format, 3);
        assert_approx_eq!(data.humidity, 0.0, 1e-4);
        assert_approx_eq!(data.temperature, -127.99, 1e-4);
        assert_eq!(data.pressure, 50000);
        assert_approx_eq!(data.acceleration_x, -32.767, 1e-4);
        assert_approx_eq!(data.acceleration_y, -32.767, 1e-4);
        assert_approx_eq!(data.acceleration_z, -32.767, 1e-4);
        assert_eq!(data.voltage, 0.0);
    }

    #[test]
    fn test_ruuvi_decode_v3_maximum_values() {
        let s = decode_hex("03FF7F63FFFF7FFF7FFF7FFFFFFF");
        /* Taken from https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-3-rawv1
        */

        let data = ruuvi_decode_v3(&s.unwrap());

        assert_eq!(data.format, 3);
        assert_approx_eq!(data.humidity, 127.5, 1e-4);
        assert_approx_eq!(data.temperature, 127.99, 1e-4);
        assert_eq!(data.pressure, 115535);
        assert_approx_eq!(data.acceleration_x, 32.767, 1e-4);
        assert_approx_eq!(data.acceleration_y, 32.767, 1e-4);
        assert_approx_eq!(data.acceleration_z, 32.767, 1e-4);
        assert_eq!(data.voltage, 65.535);
    }

}
