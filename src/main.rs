use solution::Packet;

fn main() {
    let packet = Packet::from_source(b"rbcd", 3);
    println!("{:?}", packet);
}
