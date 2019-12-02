// use solution::Packet;
// use solution::PacketSerializer;
// use solution::Packetable;
use solution::*;

fn main() {
    // let (packet, _remainder) = Packet::from_source(b"rbcd", 3);
    // let serialized = packet.serialize();
    // println!("{:?}", serialized);

    // let deserialized = Packet::deserialize(&serialized);
    // println!("{:?}", deserialized);

    // let arr = vec![1, 3, 114, 98, 99, 0, 0, 1, 55];
    // let deserialized_broken = Packet::deserialize(&arr);
    // println!("{:?}", deserialized_broken);

    // let string = String::from("asdf");
    // let serializer: Vec<Packet> = string.to_packets(4).collect();
    // println!("{:?}", serializer);
    let packet = [1, 0, 0, 0, 0, 0];
    let content = String::from_packet_data(&packet).unwrap();

    println!("result {:?}", content);
    //assert_eq!(content, "");
}
