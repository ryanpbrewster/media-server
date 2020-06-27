use image::{GenericImageView, ImageBuffer, Rgba};
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();
    let img = image::open(&opts.input).unwrap();
    let (width, height) = img.dimensions();
    let hash = blurhash::encode(opts.x, opts.y, width, height, &img.into_rgba().into_vec());
    println!("{}", hash);
    let blur: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_vec(width, height, blurhash::decode(&hash, width, height, 1.0)).unwrap();
    if let Some(output) = opts.output {
        blur.save(output).unwrap();
    }
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    input: PathBuf,

    #[structopt(long, parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(long, default_value = "5")]
    x: u32,
    #[structopt(long, default_value = "5")]
    y: u32,
}
