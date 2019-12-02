use std::convert::TryInto;
use std::fmt;

/// Грешките, които ще очакваме да върнете. По-долу ще е описано кои от тези грешки очакваме да се
/// върнат в каква ситуация.
///
#[derive(Debug)]
pub enum PacketError {
    InvalidPacket,
    InvalidChecksum,
    UnknownProtocolVersion,
    CorruptedMessage,
}

/// Нужна е имплементация на Display за грешките, за да може да имплементират `std::error::Error`.
/// Свободни сте да напишете каквито искате съобщения, ще тестваме само типовете, не низовия им
/// вид.
///
/// Ако са във формат на хайку, няма да получите бонус точки, но може да получите чувство на
/// вътрешно удовлетворение.
///
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

/// Тази имплементация би трябвало да сработи директно благодарение на горните. При желание, можете
/// да си имплементирате ръчно някои от методите, само внимавайте.
///
impl std::error::Error for PacketError {}

impl From<std::str::Utf8Error> for PacketError {
    fn from(_err: std::str::Utf8Error) -> Self {
        PacketError::CorruptedMessage
    }
}

/// Един пакет, съдържащ част от съобщението. Изберете сами какви полета да използвате за
/// съхранение.
///
/// Може да е нужно да добавите lifetimes на дефиницията тук и/или на методите в impl блока.
///
#[derive(PartialEq, Debug)]
pub struct Packet<'a> {
    version: u8,
    size: u8,
    payload: &'a [u8],
    checksum: [u8; 4],
}

impl<'a> Packet<'a> {
    /// Конструира пакет от дадения slice от байтове. Приема параметър `size`, който е размера на
    /// payload-а на новия пакет. Връща двойка от пакет + оставащите байтове. Тоест, ако имате низа
    /// "abcd" и викнете метода върху байтовата му репрезентация с параметър `size` равен на 3, ще
    /// върнете двойката `(<пакет с payload "abc">, <байтовия низ "d">)`.
    ///
    /// Байтове от низ можете да извадите чрез `.as_bytes()`, можете и да си конструирате байтов
    /// литерал като b"abcd".
    ///
    /// Ако подадения `size` е по-голям от дължината на `source`, приемаме, че размера ще е точно
    /// дължината на `source` (и остатъка ще е празен slice).
    ///
    /// Ако параметъра `size` е 0, очакваме тази функция да panic-не (приемаме, че това извикване
    /// просто е невалидно, програмистка грешка).
    ///
    pub fn from_source(source: &'a [u8], size: u8) -> (Self, &[u8]) {
        if size == 0 {
            panic!();
        }

        let payload: &'a [u8];
        let remainder: &'a [u8];

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

    /// Връща само slice-а който пакета опакова. Тоест, ако сме конструирали пакета със
    /// `Packet::from_source(b"abc", 3)`, очакваме `.payload()` да ни върне `b"abc"`.
    ///
    /// Защо това просто не е публично property? За да не позволяваме мутация, а само конструиране
    /// и четене.
    ///
    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    /// Сериализира пакета, тоест превръща го в байтове, готови за трансфер. Версия, дължина,
    /// съобщение (payload), checksum. Вижте по-горе за детайлно обяснение.
    ///
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![self.version, self.size];

        bytes.extend_from_slice(self.payload);
        bytes.extend(self.checksum.iter().cloned());

        bytes
    }

    // Note: if size is less, it'll be the checksum not being valid
    /// Имайки slice от байтове, искаме да извадим един пакет от началото и да върнем остатъка,
    /// пакетиран в `Result`.
    ///
    /// Байтовете са репрезентация на пакет -- версия, размер, и т.н. както е описано по-горе.
    ///
    /// Ако липсват версия, размер, чексума, или размера е твърде малък, за да може да се изпарси
    /// валиден пакет от байтовете, връщаме грешка `PacketError::InvalidPacket`.
    ///
    /// Ако версията е различна от 1, връщаме `PacketError::UnknownProtocolVersion`.
    ///
    /// Ако checksum-а, който прочитаме от последните 4 байта на пакета е различен от изчисления
    /// checksum на payload-а (сумата от байтовете му), връщаме `PacketError::InvalidChecksum`.
    ///
    /// Забележете, че ако размера е по-голям от истинския размер на payload-а, се очаква
    /// `PacketError::InvalidPacket`. Ако размера е по-малък от истинския размер на payload-а,
    /// въпросния ще се изпарси, но чексумата ще е грешна, така че ще очакваме
    /// `PacketError::InvalidChecksum`. Малко тъпо! Но уви, протоколите имат подобни тъпи ръбове,
    /// особено като са написани за един уикенд. Авторите обещават по-добър протокол за версия 2.
    ///
    pub fn deserialize(bytes: &[u8]) -> Result<(Packet, &[u8]), PacketError> {
        let byte_count = bytes.len();
        if byte_count < 6 {
            return Err(PacketError::InvalidPacket);
        }

        let version = match bytes[0] {
            1 => 1,
            _ => return Err(PacketError::UnknownProtocolVersion),
        };

        let size = bytes[1] as usize;

        if size > (byte_count - 6_usize) {
            return Err(PacketError::InvalidPacket);
        }

        let payload = &bytes[2..(size + 2)];
        let checksum_to_check = &bytes[(size + 2)..(size + 6)];
        let checksum = Self::find_checksum(payload);

        if checksum != checksum_to_check {
            return Err(PacketError::InvalidChecksum);
        }

        let remainder = &bytes[(size + 6)..];

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

/// Структура, която ще служи за итериране по пакети. Ще я конструираме от някакво съобщение, и
/// итерацията ще връща всеки следващ пакет, докато съобщението не бъде напълно "изпратено".
/// Изберете каквито полета ви трябват.
///
/// Може да е нужно да добавите lifetimes на дефиницията тук и/или на методите в impl блока.
///
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

/// Този trait ще ни позволи да конвертираме един `String` (а ако искаме, и други неща) от и до
/// комплект от байтове за прехвърляне по мрежата.
///
/// Детайли за методите вижте по-долу в имплементацията на този trait за `String`.
///
pub trait Packetable: Sized {
    fn to_packets(&self, packet_size: u8) -> PacketSerializer;
    fn to_packet_data(&self, packet_size: u8) -> Vec<u8>;
    fn from_packet_data(packet_data: &[u8]) -> Result<Self, PacketError>;
}

impl Packetable for String {
    /// Този метод приема размер, който да използваме за размера на payload-а на всеки пакет. Връща
    /// итератор върху въпросните пакети. Низа трябва да се използва под формата на байтове.
    ///
    fn to_packets(&self, packet_size: u8) -> PacketSerializer {
        let string_as_bytes = self.as_bytes();

        PacketSerializer {
            packet_size,
            remaining_bytes: string_as_bytes,
        }
    }

    /// Имайки итератор по пакети, лесно можем да сериализираме всеки индивидуален пакет в поредица
    /// от байтове със `.serialize()` и да го натъпчем във вектора.
    ///
    fn to_packet_data(&self, packet_size: u8) -> Vec<u8> {
        let mut serialized_data = Vec::<u8>::new();
        let packet_serializer = self.to_packets(packet_size);

        for packet in packet_serializer {
            serialized_data.extend(packet.serialize());
        }

        serialized_data
    }

    /// Обратното на горния метод е тази асоциирана функция -- имайки slice от байтове които са
    /// сериализирана репрезентация на пакети, искаме да десериализираме пакети от този slice, да
    /// им извадим payload-ите, и да ги сглобим в оригиналното съобщение.
    ///
    /// Грешките, които могат да се върнат, са същите, които идват от `.deserialize()`.
    ///
    /// Една допълнителна грешка, която може да се случи е при сглобяване на съобщението -- ако е
    /// имало липсващ пакет, може съчетанието на байтовете да не генерира правилно UTF8 съобщение.
    /// Тогава връщаме `PacketError::CorruptedMessage`.
    ///
    fn from_packet_data(packet_data: &[u8]) -> Result<Self, PacketError> {
        let mut remaining_data: &[u8] = packet_data;
        let mut content: String = String::new();

        while remaining_data.len() > 0 {
            let (packet, remainder) = Packet::deserialize(remaining_data)?;
            let parsed_payload = std::str::from_utf8(packet.payload())?;
            
            content.push_str(parsed_payload);
            remaining_data = remainder;
        }

        Ok(content)
    }
}
