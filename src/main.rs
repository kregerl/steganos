use std::io::{Write, stdin, stdout};
use std::env;
use std::path::Path;
use std::process::exit;
use image::{RgbaImage, GenericImageView, DynamicImage, Pixel, ColorType, Rgba};
use bitreader::BitReader;

fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    let usage = "USAGE:
    stegnog [OPTIONS]
OPTIONS:
    -in <original_png> -out <output_png> 
    -read <input_png>";

    if args.len() == 5 {
        println!("{:?}", args);
        if args[1] == "in" && args[3] == "out" {
            let img = image::open(&args[2]).unwrap();

            const TEXT: &str = "Hello ed";

            write_image(img, TEXT, &args[4]);
        }
    } else if args.len() == 3 {
        if args[1] == "read" {
            let img = image::open(&args[2]).unwrap();

            println!("{}", read_image(img));
        }
    } else {
        println!("{}", usage);
    }
}

fn read_image(image: DynamicImage) -> String {
    let mut bits: Vec<u8> = Vec::new();
    let mut consecutive_zeros = 0;
    'outer: for (_, _, pixel) in image.pixels() {
        for color in pixel.0 {
            let bit = color & 1;
            if bit == 0 {
                consecutive_zeros += 1;
            } else {
                consecutive_zeros = 0;
            }
            bits.push(bit);
            if consecutive_zeros == 8 {
                break 'outer;
            }
        }
    }
    let mut count = 0;
    for bit in bits {
        print!("{}", bit);
        if count != 0 && count % 8 == 0 {
            println!();
        }
        count += 1;
    }

    String::from("")
}

fn write_image(original: DynamicImage, text: &str, output_name: &str) {
    let bytes: Vec<u8> = text.bytes().collect();
    let mut reader = BitReader::new(&*bytes);
    let mut bits: Vec<u8> = Vec::new();
    while reader.remaining() > 0 {
        let mask = 0 | reader.read_u8(1).unwrap();
        bits.push(mask);
    }
    for _ in 0..8 {
        bits.push(0);
    }
    println!("-------------------------");
    for b in &bits {
        print!("{}", b);
    }
    println!();

    let mut bit_iter = bits.iter();
    let (width, height) = original.dimensions();
    let mut new_image = RgbaImage::new(width, height);
    let mut testing: Vec<u8> = Vec::new();
    for (x, y, mut pixel) in original.pixels() {
        pixel = pixel.map(|color| {
            if let Some(bit) = bit_iter.next() {
                testing.push((color & 254) | bit);
                println!("Mapped: {} to {}", color, (color & 254) | bit);
                return (color & 254) | bit;
            }
            color
        });
        new_image.put_pixel(x, y, pixel);
    }

    let res = image::save_buffer(&Path::new(output_name), &*new_image, width, height, ColorType::Rgba8);
    match res {
        Err(err) => eprintln!("{}", err),
        _ => {}
    }
}