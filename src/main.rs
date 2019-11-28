use solution::Packet;

fn main() {
    let (packet, _remainder) = Packet::from_source(b"rbcd", 3);
    let serialized = packet.serialize();
    println!("{:?}", serialized);

    let deserialized = Packet::deserialize(&serialized);
    println!("{:?}", deserialized);

    let arr = vec![1, 3, 114, 98, 99, 0, 0, 1, 55];
    let deserialized_broken = Packet::deserialize(&arr);
    println!("{:?}", deserialized_broken);
}
