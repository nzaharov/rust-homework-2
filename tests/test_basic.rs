// Бележка: името на проекта трябва да се казва "solution". Ако не се казва така, променете го
// на този ред:
use solution::*;

#[test]
fn test_basic_packets() {
    let source = b"hello";
    let (packet, remainder) = Packet::from_source(source, 100);

    assert_eq!(packet.payload().len(), source.len());
    assert_eq!(remainder, b"");
    assert!(packet.serialize().len() > 0);

    if let Err(_) = Packet::deserialize(&packet.serialize()) {
        assert!(false, "Couldn't deserialize serialized packet");
    }
}

#[test]
fn test_basic_iteration() {
    let source = String::from("hello");
    let packets = source.to_packets(100).collect::<Vec<Packet>>();
    assert!(packets.len() > 0);

    let data = source.to_packet_data(100);
    assert!(data.len() > 0);

    if let Err(_) = String::from_packet_data(&data) {
        assert!(false, "Couldn't deserialize serialized packet data");
    }
}

// #[test]
// fn test_none() {
//     let source = String::from("hello")
// }

#[test]
fn test_to_packet_data() {
    let (packet, _remainder) = Packet::from_source(b"rbcd", 4);
    let serialized = packet.serialize();

    let packet_data = String::from("rbcd").to_packet_data(4);

    assert_eq!(packet_data, serialized);
}

#[test]
fn test_round_trip() {
    let initial_data = String::from("asdfrtyuioasd;235'zx");
    let packet_data = initial_data.to_packet_data(4);
    let restored_data = String::from_packet_data(&packet_data).unwrap();

    assert_eq!(initial_data, restored_data);
}