use image::{DynamicImage, GenericImageView};

pub enum EncodedType {
    Text(String),
    RgbaPng(DynamicImage),
}

impl EncodedType {
    // Headers must be the same length
    fn header(&self) -> &str {
        match *self {
            EncodedType::Text(_) => "txt",
            EncodedType::RgbaPng(_) => "img"
        }
    }

    fn byte_header(&self) -> Vec<Byte> {
        self.header().bytes().map(|c| Byte::new(c)).collect()
    }

    pub fn to_bytes(&self) -> Vec<Byte> {
        let mut bytes = self.byte_header();
        match self {
            EncodedType::Text(s) => {
                for b in s.bytes() {
                    bytes.push(Byte::new(b));
                }
            }
            EncodedType::RgbaPng(image) => {
                let (width, height) = image.dimensions();

                // Stores the images dimensions bytes to be embedded
                //        w   h
                // header800|600|<data>
                let mut dims: Vec<Byte> = Vec::new();
                dims.append(&mut Byte::from_u32(width));
                dims.push(Byte::from('|'));
                dims.append(&mut Byte::from_u32(height));
                dims.push(Byte::from('|'));

                bytes.append(&mut dims);
                for (_, _, pixel) in image.pixels() {
                    for color in pixel.0 {
                        bytes.push(Byte::new(color));
                    }
                }
            }
        }
        bytes.append(&mut Byte::ending_seq());
        bytes
    }

    pub fn to_bits(&self) -> Vec<bool> {
        self.to_bytes().iter().map(|b| b.get_bits()).flatten().collect()
    }

    pub fn header_size() -> usize {
        3
    }
}

#[derive(PartialEq, Clone)]
pub struct Byte {
    pub byte: u8,
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


    pub fn from_u32(num: u32) -> Vec<Byte> {
        num.to_be_bytes().to_vec().iter().map(|byte| Byte::new(*byte)).collect()
    }

    pub fn from_str(s: &str) -> Vec<Byte> {
        let mut result: Vec<Byte> = Vec::new();
        for c in s.chars() {
            result.push(Byte::from(c));
        }
        result
    }

    pub fn ending_seq() -> Vec<Byte> {
        Byte::from_str("$t36an0s")
    }

    pub fn get_bits(&self) -> Vec<bool> {
        let mut bits: Vec<bool> = Vec::new();
        for n in 0..8 {
            bits.push(get_lsb(self.byte >> n));
        }
        bits.reverse();
        bits
    }

    pub fn as_char(&self) -> char {
        self.byte as char
    }

    pub fn print_bits(&self) {
        for bit in self.get_bits() {
            print!("{}", bit as u8);
        }
        println!();
    }
}

impl From<char> for Byte {
    fn from(c: char) -> Self {
        Self {
            byte: c as u8
        }
    }
}

impl From<&[bool]> for Byte {
    fn from(arr: &[bool]) -> Self {
        if arr.len() > 8 {
            panic!("Cannot form a byte from a bool array of size: {}", arr.len());
        }
        let mut byte_str: String = String::new();
        for b in arr {
            byte_str.push_str(&*(*b as u8).to_string());
        }


        match u8::from_str_radix(&byte_str[..], 2) {
            Ok(byte) => {
                Self {
                    byte
                }
            }
            Err(err) => {
                eprintln!("Error parsing integer, byte is 0. {}", err);
                Self {
                    byte: 0
                }
            }
        }
    }
}

// Gets the least significant bit from the `byte` param
// 0b01001010
// ---------^
pub fn get_lsb(byte: u8) -> bool {
    byte & 1 != 0
}

