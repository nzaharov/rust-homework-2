use solution::Packet;

fn main() {
    let (packet, _remainder) = Packet::from_source(b"rbcd", 3);
    println!("{:?}", packet.serialize());
    println!("{:?}", packet);
    println!("{:?}", packet.payload());
}
