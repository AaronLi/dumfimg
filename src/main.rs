mod image_view;

use std::collections::{HashMap, HashSet};
use std::env::args;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use cursive::{Cursive, CursiveExt};
use cursive::views::{DebugView, ResizedView};
use image::{imageops, ImageResult};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use crate::image_view::ImageView;

#[derive(Parser, Debug)]
#[command(version, author, about)]
struct Args {
    #[arg(short, long)]
    input_image: String
}

fn main() {
    let args = Args::parse();
    let image = match ImageReader::open(args.input_image) {
        Ok(image) => {
            match image.decode() {
                Ok(image_data) => {
                    image_data.to_rgb8()
                }
                Err(e) => {
                    println!("Failed to read image data: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            println!("Failed to open image: {}", e);
            exit(1);
        }
    };
    let mut base = Cursive::new();
    base.add_fullscreen_layer(ResizedView::with_full_screen(ImageView::new(image)));
    base.run();
}