mod byteutils;
mod cli;

use std::io::{Write, stdin, stdout};
use std::env;
use std::path::Path;
use clap::{App, Arg};
use image::{RgbaImage, GenericImageView, DynamicImage, Pixel, ColorType, Rgba, Pixels, ImageBuffer};
use crate::byteutils::{Byte, EncodedType, get_lsb};
use crate::cli::{ArgType, Arguments};

fn main() {
    match Arguments::new().args {
        ArgType::Write(wa) => {
            let img = open_image(&wa.input);

            if wa.is_data_file() {
                let image_to_embed = open_image(&wa.data);
                embed_image_in_image(img, image_to_embed, wa.output.as_str());
            } else {
                embed_text_in_image(img, wa.data.as_str(), wa.output.as_str());
            }
        }
        ArgType::Read(ra) => {
            let img = open_image(&ra.input);

            decode_image(img, ra.hexdump);
        }
    }
}

fn decode_image(image: DynamicImage, hexdump: bool) {
    // TODO: Change delimiter again
    // Loop over every channel in the image and pull out the lsb from each, stopping at the '####'
    let pixels: Vec<(u32, u32, Rgba<u8>)> = image.pixels().into_iter().collect();
    let mut consecutive_pound_signs = 0;
    let mut bytes: Vec<Byte> = Vec::new();
    for pixel_group in pixels.chunks(2) {
        let mut bit_list: Vec<bool> = Vec::new();
        for (_, _, pixel) in pixel_group {
            for color in pixel.0 {
                bit_list.push(get_lsb(color));
            }
        }
        let byte = Byte::from(&bit_list[..]);
        if byte.as_char() == '#' {
            consecutive_pound_signs += 1;
        } else {
            consecutive_pound_signs = 0;
        }
        bytes.push(byte);
        if consecutive_pound_signs == 4 {
            break;
        }
    }

    println!("bytes len before: {}", bytes.len());

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
                read_bytes_and_save_image(body, "output.png");
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

    let new_image = write_bits_to_image(original, bits);

    save_image(new_image, output_path);
}

fn embed_image_in_image(original: DynamicImage, image_to_embed: DynamicImage, output_path: &str) {
    let (embed_width, embed_height) = image_to_embed.dimensions();
    let (original_width, original_height) = original.dimensions();
    // Number of bytes in the image (4 channel RGBA).
    let expected_pixels = (original_width * original_height) * 4;
    const NUM_BITS_PER_PIXEL: u32 = 32;

    if (embed_width * embed_height) * NUM_BITS_PER_PIXEL > expected_pixels {
        panic!("The image specified is too big to fit into the desired image. Expected an image with less than {} pixels", expected_pixels);
    }

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
    let mut bit_count = 0;
    for (x, y, mut pixel) in original_image.pixels() {
        pixel = pixel.map(|color| {
            if let Some(bit) = bit_iter.next() {
                bit_count += 1;
                return (color & 254) | *bit as u8;
            }
            color
        });
        image.put_pixel(x, y, pixel);
    }
    println!("Wrote {} bytes containing {} pixels into the image.", bit_count / 8, ((bit_count / 8) - (4 + 3 + 10)) / 4);
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

fn read_bytes_and_save_image(bytes: Vec<Byte>, output_path: &str) {
    println!("num bytes: {}", bytes.len());

    let mut size: Vec<u32> = Vec::with_capacity(2);
    let mut start = 0;
    // Assume the size is in order width|height|
    for (index, byte) in bytes[0..10].to_vec().iter().enumerate() {
        if byte.as_char() == '|' {
            size.push(read_bytes_as_u32(&bytes[start..index]));
            start = index + 1;
        }
    }
    if size.len() != 2 { panic!("Expected a width and height after encountering an img header.") }
    let width = size[0];
    let height = size[1];
    let mut new_image = RgbaImage::new(width, height);

    // Skip the 10 bytes used to size and the last 4 that are the stopping sequence.
    let byte_nums: Vec<u8> = bytes[10..bytes.len() - 4].to_vec().iter().map(|byte| byte.byte).collect();

    let mut byte_iter = byte_nums.chunks(4);

    'y: for y in 0..height {
        for x in 0..width {
            if let Some(pixel) = byte_iter.next() {
                let channels: [u8; 4] = pixel.try_into().unwrap_or_else(|err| panic!("Cannot read slice since it is smaller than 4 bytes {}", err));
                let pixel = Rgba::from(channels);
                new_image.put_pixel(x, y, pixel);
            } else {
                break 'y;
            }
        }
    }

    save_image(new_image, output_path);
}

fn read_bytes_as_u32(bytes: &[Byte]) -> u32 {
    let t: Vec<u8> = bytes.to_vec().iter().map(|byte| byte.byte).collect();
    u32::from_be_bytes(t.try_into().unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length 4 but it was {}", v.len())))
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

