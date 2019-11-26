use solution::Packet;

fn main() {
    let (packet, remainder) = Packet::from_source(b"rbcd", 3);
    println!("{:?}", packet);
    println!("{:?}", packet.payload());
}
