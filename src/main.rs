extern crate args;
extern crate ncurses;

mod config;
mod fft;

use config::Config;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;

fn main() {
    Config::try_create_default_config();

    let mut args = Config::create_args();

    if args.value_of("help").unwrap() {
        println!("{}", args.full_usage());

        return;
    }

    let config = Config::new_from_arguments(&mut args);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let file = BufReader::new(File::open(config.audio_file_path).unwrap());
    let source = Decoder::new(file).unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(source);

    sink.sleep_until_end();
}
