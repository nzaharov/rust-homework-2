use std::convert::TryInto;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PacketError {
    InvalidPacket,
    InvalidChecksum,
    UnknownProtocolVersion,
    CorruptedMessage,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidPacket => write!(f, "Invalid packet"),
            Self::InvalidChecksum => write!(f, "Checksum invalid"),
            Self::UnknownProtocolVersion => write!(f, "Unknown protocol version"),
            Self::CorruptedMessage => write!(f, "Data is corrupted"),
        }
    }
}

impl std::error::Error for PacketError {}

#[derive(PartialEq, Debug)]
pub struct Packet<'a> {
    version: u8,
    size: u8,
    payload: &'a [u8],
    checksum: [u8; 4],
}

impl<'a> Packet<'a> {
    pub fn from_source(source: &'a [u8], size: u8) -> (Self, &[u8]) {
        if size == 0 {
            panic!();
        }

        let payload: &[u8];
        let remainder: &[u8];

        let source_length = source.len();
        let mut parsed_size = size as usize;

        if source_length > parsed_size {
            payload = &source[0..parsed_size];
            remainder = &source[parsed_size..];
        } else {
            payload = source;
            remainder = &[];
            parsed_size = source_length;
        }

        let checksum: [u8; 4] = Self::find_checksum(payload);

        (
            Packet {
                version: 1,
                size: parsed_size.try_into().unwrap(),
                payload,
                checksum,
            },
            remainder,
        )
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![self.version, self.size];

        bytes.extend_from_slice(self.payload);
        bytes.extend(self.checksum.iter().cloned());

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Packet, &[u8]), PacketError> {
        let reserved_bytes_count = 6_usize;

        let byte_count = bytes.len();
        if byte_count < reserved_bytes_count {
            return Err(PacketError::InvalidPacket);
        }

        let version = match bytes[0] {
            1 => 1,
            _ => return Err(PacketError::UnknownProtocolVersion),
        };

        let size = bytes[1] as usize;
        if size > (byte_count - reserved_bytes_count) {
            return Err(PacketError::InvalidPacket);
        }

        let payload = &bytes[2..(size + 2)];
        let checksum_to_check = &bytes[(size + 2)..(size + reserved_bytes_count)];
        let checksum = Self::find_checksum(payload);
        if checksum != checksum_to_check {
            return Err(PacketError::InvalidChecksum);
        }

        let remainder = &bytes[(size + reserved_bytes_count)..];

        Ok((
            Packet {
                version,
                size: size.try_into().unwrap(),
                payload,
                checksum,
            },
            remainder,
        ))
    }

    fn find_checksum(payload: &[u8]) -> [u8; 4] {
        let sum: u32 = payload.iter().map(|&byte| byte as u32).sum();
        sum.to_be_bytes()
    }
}

#[derive(Debug)]
pub struct PacketSerializer<'a> {
    packet_size: u8,
    remaining_bytes: &'a [u8],
}

impl<'a> Iterator for PacketSerializer<'a> {
    type Item = Packet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bytes.len() == 0 {
            return None;
        }
        let (packet, remainder) = Packet::from_source(self.remaining_bytes, self.packet_size);
        self.remaining_bytes = remainder;

        Some(packet)
    }
}

pub trait Packetable: Sized {
    fn to_packets(&self, packet_size: u8) -> PacketSerializer;
    fn to_packet_data(&self, packet_size: u8) -> Vec<u8>;
    fn from_packet_data(packet_data: &[u8]) -> Result<Self, PacketError>;
}

impl Packetable for String {
    fn to_packets(&self, packet_size: u8) -> PacketSerializer {
        let string_as_bytes = self.as_bytes();
        PacketSerializer {
            packet_size,
            remaining_bytes: string_as_bytes,
        }
    }

    fn to_packet_data(&self, packet_size: u8) -> Vec<u8> {
        let mut serialized_data = Vec::<u8>::new();
        let packet_serializer = self.to_packets(packet_size);

        for packet in packet_serializer {
            serialized_data.extend(packet.serialize());
        }

        serialized_data
    }

    fn from_packet_data(packet_data: &[u8]) -> Result<Self, PacketError> {
        let mut remaining_data: &[u8] = packet_data;
        let mut encoded_message = Vec::<u8>::new();

        while remaining_data.len() > 0 {
            let (packet, remainder) = Packet::deserialize(remaining_data)?;

            encoded_message.extend_from_slice(packet.payload());
            remaining_data = remainder;
        }

        String::from_utf8(encoded_message).map_err(|_| PacketError::CorruptedMessage)
    }
}
