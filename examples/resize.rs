use image::GenericImageView;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();
    let mut img = image::open(&opts.input).expect("failed to open image");
    println!("original dimensions = {:?}", img.dimensions());
    img = img.thumbnail(128, 128);
    println!("thumbnail dimensions = {:?}", img.dimensions());
    img.save(&opts.output).expect("failed to save image");
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    input: PathBuf,
    #[structopt(long, parse(from_os_str))]
    output: PathBuf,
}
