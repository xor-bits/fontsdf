use image::GrayImage;
use sdf_font::Font;

#[test]
fn main_test() {
    const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");
    let font = Font::form_bytes(FONT_FILE).unwrap();

    let (metrics, simple) = font.rasterize('b', 512.0);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, simple)
        .unwrap()
        .save("test1.png")
        .unwrap();

    let (metrics, sdf) = font.rasterize_sdf('b', 512.0);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, sdf)
        .unwrap()
        .save("test2.png")
        .unwrap();
}
