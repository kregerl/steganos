mod byteutils;

use std::io::{Write, stdin, stdout};
use std::env;
use std::path::Path;
use image::{RgbaImage, GenericImageView, DynamicImage, Pixel, ColorType, Rgba};
use crate::byteutils::{Byte, EncodedType, get_lsb};

// TODO: Switch cmdline args to CLAP https://rust-lang-nursery.github.io/rust-cookbook/cli/arguments.html
fn main() {
    let args: Vec<String> = env::args().collect();

    let usage = "USAGE:
    stegnog [OPTIONS]
OPTIONS:
    -in <original_png> -out <output_png> 
    -read <input_png>
	-in <original_png> -out <output_png> -embed <embed_png>";

    if args.len() == 5 {
        println!("{:?}", args);
        if args[1] == "in" && args[3] == "out" {
            let img = image::open(&args[2]).unwrap();

            const TEXT: &str = "           __
             <(o )___
              ( ._> /
               `---'  ";

            embed_text_in_image(img, TEXT, &args[4]);
        }
    } else if args.len() == 3 {
        if args[1] == "read" {
            let img = image::open(&args[2]).unwrap();

            read_image(img);
        }
    } else if args.len() == 7 {
        // if args[1] == "in" && args[3] == "out" && args[5] == "embed" {
        //     let img = image::open(&args[2]).unwrap();
        //     let embed = image::open(&args[6]).unwrap();
        //
        //     let bits = convert_image_to_bits(embed);
        //     write_bits_to_image(img, &args[4], bits);
        // }
    } else {
        println!("{}", usage);
    }
}

fn read_image(image: DynamicImage) {
    let mut encoded_contents: Vec<bool> = Vec::new();

    let mut num_consecutive_zeros = 0;
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

    let body = bytes.split_off(3);
    let header: String = bytes.iter().map(|c| c.as_char()).collect();

    match header.as_str() {
        "txt" => { println!("Decoded message: \n\t{}", read_bytes_as_text(body)) }
        "img" => {
            read_bytes_as_image(body);
        }
        _ => {
            println!("Unknown header type, dumping output.");
            for byte in bytes[3..].iter() {
                byte.print_bits();
            }
        }
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
    let mut image = RgbaImage::new(width, height);
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

