use std::ffi::OsString;
use clap::{App, Arg};
use image::{DynamicImage};

#[derive(Debug, PartialEq)]
pub struct SteganosArgs {
    original_image_path: String,
    input: String,
    output_image_name: String,
}

impl SteganosArgs {
    pub fn new() -> Self {
        Self::new_from(std::env::args_os().into_iter()).unwrap_or_else(|e| e.exit())
    }

    fn new_from<I, T>(args: I) -> Result<Self, clap::Error> where
        I: Iterator<Item=T>, T: Into<OsString> + Clone {
        let matches = App::new("Steganos")
            .version("0.1.0")
            .about("Embeds text and images into the least significant bits of another image and provides a method of retrieving them again.")
            .arg(Arg::with_name("original")
                .short('o')
                .takes_value(true)
                .help("The original image that will be copied and embedded with other data."))
            .arg(Arg::with_name("input")
                .short('i').
                takes_value(true).
                min_values(0).// TODO: Fix this help, make it clear how the size is calculated.
                help("The data that will be embedded into the original image. \
            This can either be some text or an image however it can embed any file type as long as it will fit within the original image. "))
            .arg(Arg::with_name("name")
                .short('n')
                .takes_value(true)
                .help("The name of the newly created image."))
            .get_matches_from_safe(args)?;

        Ok(Self {
            original_image_path: String::from(matches.value_of("original").expect("The original input image is required.")),
            input: String::from(matches.value_of("input").expect("TODO: Allow this to be empty, get the input from stdin.")),
            output_image_name: String::from(matches.value_of("name").expect("A name for the output image is required.")),
        })
    }

    pub fn get_original_image(&self) -> DynamicImage {
        match image::open(&self.original_image_path) {
            Ok(img) => img,
            Err(err) => {
                panic!("Error reading the original image. {}", err);
            }
        }
    }

    pub fn get_text(&self) -> &str {
        &self.input
    }

    pub fn get_output_name(&self) -> &str {
        &self.output_image_name
    }
}
