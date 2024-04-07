
/*
pub fn decode_ble_packet(data : Vec<u8>) {

    let data_length = data[3];
    let data_type = data[4]; // 0xFF for manufacturer specific data
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
    fn test_packet_decoding_1() {
        let s = decode_hex("02010611FF9904035D1929C6670029FFEA041B0B6B");

        decode_ble_packet(s);
    }

}
*/