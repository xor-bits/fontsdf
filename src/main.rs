/* #![feature(bench_black_box)]

use std::hint::black_box;

use fontsdf::Font;

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

fn main() {
    let font = Font::from_bytes(FONT_FILE).unwrap();
    let index = font.lookup_glyph_index('X');

    for _ in 0..100000 {
        font.rasterize_indexed_sdf(black_box(index), black_box(100.0));
    }
} */
fn main() {}
