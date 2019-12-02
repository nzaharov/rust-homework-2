use solution::*;

#[test]
fn test_to_packet_data() {
    let (packet, _remainder) = Packet::from_source(b"rbcd", 4);
    let serialized = packet.serialize();

    let packet_data = String::from("rbcd").to_packet_data(4);

    assert_eq!(packet_data, serialized);
}

#[test]
fn test_successful_round_trip() {
    successful_round_trip("asdfrtyuioasd;235'zx");
    successful_round_trip("адяявея12124жз'd;ч");
    successful_round_trip(" ᾠ ᾡ ᾢ ᾣ ᾤ ᾥ ᾦ ᾧ ᾨ ᾩ ᾪ ᾫ ᾬ ᾭ ᾮ ᾯ ᾰ ᾱ ᾲ ᾳ ᾴ ᾵ ᾶ");
}

fn successful_round_trip(message: &str) {
    let initial_data = String::from(message);
    let packet_data = initial_data.to_packet_data(4);
    let restored_data = String::from_packet_data(&packet_data).unwrap();
    assert_eq!(initial_data, restored_data);
}

#[test]
fn test_incoming_zero() {
    let packet = [1, 1, 0, 0, 0, 0, 0];
    let content = String::from_packet_data(&packet).unwrap();

    println!("result {:?}", content);
    assert_eq!(content, "\u{0}");
}

#[test]
fn test_tampered_data() {
    tamper_data(3, PacketError::InvalidChecksum);
    tamper_data(0, PacketError::UnknownProtocolVersion);
    tamper_data(1, PacketError::InvalidPacket);
}

fn tamper_data(index: usize, expected_error: PacketError) {
    let initial_data = String::from("messageсда");
    let mut packet_data = initial_data.to_packet_data(4);

    packet_data[index] = 100;

    let result = match String::from_packet_data(&packet_data) {
        Ok(_) => None,
        Err(error) => Some(error),
    };

    assert_eq!(result.unwrap(), expected_error);
}

#[test]
fn test_downscaled_size() {
    let initial_data = String::from("messageсда");
    let (packet, _) = Packet::from_source(initial_data.as_bytes(), 255);
    let serialized = packet.serialize();

    assert_eq!(serialized[1] as usize, initial_data.len());
}
