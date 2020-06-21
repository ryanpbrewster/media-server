use image::gif::{Encoder, GifDecoder};
use image::imageops;
use image::{AnimationDecoder, Frame};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();
    let fin = BufReader::new(File::open(&opts.input).unwrap());
    let decoder = GifDecoder::new(fin).unwrap();
    let fout = BufWriter::new(File::create(&opts.output).unwrap());
    let mut encoder = Encoder::new(fout);
    for frame in decoder.into_frames() {
        let scaled = imageops::thumbnail(frame.unwrap().buffer(), 128, 128);
        encoder.encode_frame(Frame::new(scaled)).unwrap();
    }
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    input: PathBuf,
    #[structopt(long, parse(from_os_str))]
    output: PathBuf,
}
