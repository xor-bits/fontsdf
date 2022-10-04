use fontsdf::Font;
use std::{hint::black_box, time::Instant};

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

fn main() {
    let font = Font::from_bytes(FONT_FILE).unwrap();
    let index = font.lookup_glyph_index('X');

    let now = Instant::now();
    for _ in 0..100000 {
        font.rasterize_indexed(black_box(index), black_box(100.0), true);
    }
    println!("{:?}", now.elapsed());
}
// fn main() {}
