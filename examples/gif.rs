use image::gif::{Encoder, GifDecoder};
use image::imageops;
use image::{AnimationDecoder, Frame};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();
    let fin = File::open(&opts.input).unwrap();
    let decoder = GifDecoder::new(fin).unwrap();
    let fout = BufWriter::new(File::create(&opts.output).unwrap());
    let mut encoder = Encoder::new(fout);
    for frame in decoder.into_frames() {
        let frame = frame.unwrap();
        let mut buf = frame.into_buffer();
        let scaled = imageops::thumbnail(&mut buf, 128, 128);
        println!(
            "frame has dimensions {:?}, shrunk to {:?}",
            buf.dimensions(),
            scaled.dimensions()
        );
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
