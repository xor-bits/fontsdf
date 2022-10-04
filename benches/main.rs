use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fontsdf::Font;

//

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

//

fn benchmarks(c: &mut Criterion) {
    let font = Font::from_bytes(FONT_FILE).unwrap();
    let index = font.lookup_glyph_index('X');

    let mut group = c.benchmark_group("raster group");
    group.sample_size(100).bench_function("draw X", |b| {
        b.iter(|| font.rasterize_indexed_sdf(black_box(index), black_box(100.0)))
    });
}

//

criterion_group!(benches, benchmarks);
criterion_main!(benches);
