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
        unimplemented!()
    }
}

/// Тази имплементация би трябвало да сработи директно благодарение на горните. При желание, можете
/// да си имплементирате ръчно някои от методите, само внимавайте.
///
impl std::error::Error for PacketError {}

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
        let mut parsed_size = size as usize; // TODO: maybe rename

        if source_length > parsed_size {
            payload = &source[0..parsed_size];
            remainder = &source[parsed_size..source_length];
        } else {
            payload = source;
            remainder = &[];
            parsed_size = source_length;
        }

        let checksum: u32 = payload.iter().map(|&byte| byte as u32).sum();

        (
            Packet {
                version: 1,
                size: parsed_size.try_into().unwrap(), // TODO: maybe introduce error handling
                payload,
                checksum: checksum.to_be_bytes(),
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
        let mut bytes: Vec<u8> = vec! [
            self.version,
            self.size
        ];

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
        unimplemented!()
    }
}

/// Структура, която ще служи за итериране по пакети. Ще я конструираме от някакво съобщение, и
/// итерацията ще връща всеки следващ пакет, докато съобщението не бъде напълно "изпратено".
/// Изберете каквито полета ви трябват.
///
/// Може да е нужно да добавите lifetimes на дефиницията тук и/или на методите в impl блока.
///
pub struct PacketSerializer<'a> {
    // ...
    temp: &'a [u8],
}

impl<'a> Iterator for PacketSerializer<'a> {
    type Item = Packet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
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
        unimplemented!()
    }

    /// Имайки итератор по пакети, лесно можем да сериализираме всеки индивидуален пакет в поредица
    /// от байтове със `.serialize()` и да го натъпчем във вектора.
    ///
    fn to_packet_data(&self, packet_size: u8) -> Vec<u8> {
        unimplemented!()
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
        unimplemented!()
    }
}
