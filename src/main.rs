/* #![feature(bench_black_box)]

use fontsdf::Font;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

fn main() {
    let font = Arc::new(Font::from_bytes(FONT_FILE).unwrap());
    // let elapsed = Arc::new(AtomicU64::new(0));

    const THREADS: u32 = 20;
    const OPS: u32 = 50;
    const SIZE: f32 = 128.0;

    let elapsed: Duration = (0..THREADS)
        .map(|_| {
            let font = font.clone();
            thread::spawn(move || {
                let i = Instant::now();
                for _ in 0..OPS {
                    std::hint::black_box(font.rasterize('@', SIZE, true));
                }
                i.elapsed()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|join| join.join().unwrap())
        .sum();

    println!("{:?} per raster", elapsed / (THREADS * OPS));
}
 */

fn main() {}
