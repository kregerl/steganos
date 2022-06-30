// TODO: Refactor this to accept the actual thing its encoding and have a function for getting it as bytes.
// Allows for the header to be placed in the bytes along with the 0b00000000 separator.
pub enum EncodedType {
    Text(String),
    RgbaPng,
}

impl EncodedType {
    fn header(&self) -> &str {
        match *self {
            EncodedType::Text(_) => "text",
            EncodedType::RgbaPng => "rgba"
        }
    }

    fn byte_header(&self) -> Vec<Byte> {
        self.header().bytes().map(|c| Byte::new(c)).collect()
    }
    pub fn to_bytes(&self) -> Vec<Byte> {
        let mut bytes = self.byte_header();
        bytes.push(Byte::zero());
        match self {
            EncodedType::Text(s) => {
                for b in s.bytes() {
                    bytes.push(Byte::new(b));
                }
            }
            EncodedType::RgbaPng => {}
        }
        bytes
    }
}

pub struct Byte {
    byte: u8,
}

impl Byte {
    pub fn new(byte: u8) -> Self {
        Self {
            byte
        }
    }

    pub fn zero() -> Self {
        Self {
            byte: 0
        }
    }

    pub fn get_lsb(&self) -> bool {
        get_lsb(self.byte)
    }

    pub fn get_bits(&self) -> Vec<bool> {
        let mut bits: Vec<bool> = Vec::new();
        for n in 0..8 {
            bits.push(get_lsb(self.byte >> n));
        }
        bits.reverse();
        bits
    }
}

impl From<char> for Byte {
    fn from(c: char) -> Self {
        Self {
            byte: c as u8
        }
    }
}

pub fn get_lsb(byte: u8) -> bool {
    byte & 1 != 0
}







