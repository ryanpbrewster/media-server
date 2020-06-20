use image::imageops::FilterType;
use image::GenericImageView;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();
    println!("opening {:?}...", opts.input);
    let mut img = image::open(&opts.input).expect("failed to open image");
    println!("{:?}", img.dimensions());
    img = img.resize(400, 300, FilterType::Lanczos3);
    println!("{:?}", img.dimensions());
    println!("saving to {:?}...", opts.output);
    img.save(&opts.output).expect("failed to save image");
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    input: PathBuf,
    #[structopt(long, parse(from_os_str))]
    output: PathBuf,
}
