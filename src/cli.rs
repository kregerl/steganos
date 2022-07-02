extern crate proc_macro;

use clap::{App, Arg, SubCommand};
use image::{DynamicImage};

#[derive(Debug, PartialEq)]
pub enum ArgType {
    Write(WriteArgument),
    Read(ReadArgument),
}

#[derive(Debug, PartialEq)]
pub struct WriteArgument {
    pub input: String,
    pub data: String,
    pub output: String,
}

#[derive(Debug, PartialEq)]
pub struct ReadArgument {
    pub input: String,
    pub hexdump: bool,
}

impl WriteArgument {
    pub fn new(input: String, data: String, output: String) -> Self {
        Self {
            input,
            data,
            output,
        }
    }
}


impl ReadArgument {
    pub fn new(input: String, hexdump: bool) -> Self {
        Self {
            input,
            hexdump,
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Arguments {
    pub(crate) args: ArgType,
}

impl Arguments {
    pub fn new() -> Self {
        let matches = App::new("Steganos")
            .version("0.1.0")
            .about("Embeds text and images into the least significant bits of another image and provides a method of retrieving them again.")
            .subcommand(SubCommand::with_name("write")
                .about("Use if writing data into an image")
                .arg(Arg::with_name("input")
                    .short('i')
                    .long("input")
                    .takes_value(true)
                    .required(true)
                    .help("The input image that will be copied and embedded with other data."))
                .arg(Arg::with_name("data")
                    .short('d')
                    .long("data")
                    .takes_value(true)
                    .required(true)
                    .help("The data that will be embedded into the original image. \
                          This can either be some text or an image however it can embed any file type as long as it will fit within the original image."))
                .arg(Arg::with_name("output")
                    .short('o')
                    .long("output")
                    .takes_value(true)
                    .required(true)
                    .help("The name of the newly created image.")))
            .subcommand(SubCommand::with_name("read")
                .about("Use for reading data from the least significant bits an image.")
                .arg(Arg::with_name("input")
                    .short('i')
                    .long("input")
                    .required(true)
                    .takes_value(true)
                    .required(true)
                    .help("The path a file with embedded data to extract."))
                .arg(Arg::with_name("hexdump")
                    .short('d')
                    .long("hexdump")
                    .help("If enabled, instead of attempting to parse the text, the bytes will be dumped in hexadecimal format."))
            ).get_matches();


        if let Some(read) = matches.subcommand_matches("read") {
            Self {
                args: ArgType::Read(ReadArgument::new(read.value_of("input").unwrap().to_string(), read.is_present("hexdump")))
            }
        } else if let Some(write) = matches.subcommand_matches("write") {
            let write_args = WriteArgument::new(
                write.value_of("input").unwrap().to_string(),
                write.value_of("data").unwrap().to_string(),
                write.value_of("output").unwrap().to_string());
            Self {
                args: ArgType::Write(write_args)
            }
        } else {
            panic!("How did we get here? Please submit an issue on github at https://github.com/kregerl/steganos/issues with steps to reproduce this error.")
        }
    }
}
