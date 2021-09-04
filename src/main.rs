extern crate args;
extern crate ncurses;

mod config;
mod consts;
mod fft;

use config::Config;
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut args = Config::new_args();

    if args.value_of("help").unwrap() {
        println!("{}", args.full_usage());

        return;
    }

    let config = Config::new_from_arguments(&mut args);
    
    let file = BufReader::new(File::open(config.audio_file_path).unwrap());

    let source = Decoder::new(file).unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    stream_handle.play_raw(source.convert_samples()).unwrap();
}
