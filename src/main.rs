mod byteutils;
mod cli;

use std::io::{Write, stdin, stdout};
use std::env;
use std::path::Path;
use clap::{App, Arg};
use image::{RgbaImage, GenericImageView, DynamicImage, Pixel, ColorType, Rgba};
use crate::byteutils::{Byte, EncodedType, get_lsb};
use crate::cli::{ArgType, Arguments};

fn main() {
    match Arguments::new().args {
        ArgType::Write(wa) => {
            let img = open_image(&wa.input);

            embed_text_in_image(img, wa.data.as_str(), wa.output.as_str());
        }
        ArgType::Read(ra) => {
            let img = open_image(&ra.input);

            decode_image(img, ra.hexdump);
        }
    }
}

fn decode_image(image: DynamicImage, hexdump: bool) {
    let mut encoded_contents: Vec<bool> = Vec::new();

    let mut num_consecutive_zeros = 0;
    // Loop over every channel in the image and pull out the lsb from each, stopping at the ending(null)
    // byte
    // TODO: Change this from a nullbyte to a sequence of chars. Makes the embedded data larger but is easier to stop at.
    'outer: for (_, _, pixel) in image.pixels() {
        for color in pixel.0 {
            let bit = get_lsb(color);
            if bit {
                num_consecutive_zeros = 0;
            } else {
                num_consecutive_zeros += 1;
            }
            encoded_contents.push(bit);
            if num_consecutive_zeros == 8 {
                break 'outer;
            }
        }
    }

    let mut bytes: Vec<Byte> = encoded_contents.chunks(8).map(|chunk| Byte::from(chunk)).collect();

    if hexdump {
        dump_bytes(bytes);
    } else {
        // Split the body from the header at index `header_size`, leaves the first `header_size` bytes on
        // the `bytes` vec.
        let body = bytes.split_off(EncodedType::header_size());
        let header: String = bytes.iter().map(|c| c.as_char()).collect();

        match header.as_str() {
            "txt" => { println!("Decoded message: \n\t{}", read_bytes_as_text(body)) }
            "img" => {
                read_bytes_as_image(body);
            }
            _ => {
                println!("Unknown header type, dumping output.");
                for byte in bytes[EncodedType::header_size()..].iter() {
                    byte.print_bits();
                }
            }
        }
    }
}

fn dump_bytes(bytes: Vec<Byte>) {
    for chunk in bytes.chunks(8) {
        for byte in chunk {
            print!("{:#04x}\t", byte.byte);
        }
        println!();
    }
}

fn embed_text_in_image(original: DynamicImage, text: &str, output_path: &str) {
    let bits = EncodedType::Text(String::from(text)).to_bits();
    // let bits: Vec<bool> = bytes.iter().map(|b| b.get_bits()).flatten().collect();

    let new_image = write_bits_to_image(original, bits);

    save_image(new_image, output_path);
}

fn embed_image_in_image(original: DynamicImage, image_to_embed: DynamicImage, output_path: &str) {
    let bits = EncodedType::RgbaPng(image_to_embed).to_bits();

    let new_image = write_bits_to_image(original, bits);

    save_image(new_image, output_path);
}

fn write_bits_to_image(original_image: DynamicImage, bits: Vec<bool>) -> RgbaImage {
    let mut bit_iter = bits.iter();
    let (width, height) = original_image.dimensions();
    // Create new blank image with same dimensions
    let mut image = RgbaImage::new(width, height);
    /* Copy each pixel from the original image into the new image but replace the lsb of each color
    channel with a bit from the embedded data. */
    for (x, y, mut pixel) in original_image.pixels() {
        pixel = pixel.map(|color| {
            if let Some(bit) = bit_iter.next() {
                return (color & 254) | *bit as u8;
            }
            color
        });
        image.put_pixel(x, y, pixel);
    }
    image
}

fn read_bytes_as_text(bytes: Vec<Byte>) -> String {
    let mut result = String::new();

    // Read bytes, ignoring null bytes
    for byte in bytes {
        if Byte::zero() != byte {
            result.push(byte.as_char());
        }
    }

    result
}

fn read_bytes_as_image(bytes: Vec<Byte>) {
    todo!("Save the image's dimensions so it can be read again. Otherwise the size of the image is unknown.")
}

fn save_image(image: RgbaImage, output_path: &str) {
    let (width, height) = image.dimensions();
    let result = image::save_buffer(&Path::new(output_path), &*image, width, height, ColorType::Rgba8);
    match result {
        Ok(_) => { println!("Wrote image to {}", output_path); }
        Err(err) => { eprintln!("Error writing image to path: {}", err); }
    }
}

fn open_image(path: &str) -> DynamicImage {
    match image::open(path) {
        Ok(img) => img,
        Err(err) => {
            panic!("Error reading the original image. {}", err);
        }
    }
}

