use fontsdf::Font;
use image::GrayImage;

//

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

//

#[test]
fn main_test() {
    let font = Font::from_bytes(FONT_FILE).unwrap();

    let (metrics, simple) = font.rasterize('b', 512.0, false);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, simple)
        .unwrap()
        .save("test1.png")
        .unwrap();

    let (metrics, sdf) = font.rasterize('b', 512.0, true);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, sdf)
        .unwrap()
        .save("test2.png")
        .unwrap();
}

#[test]
fn metrics_test() {
    const PX: f32 = 12.0;

    let font = Font::from_bytes(FONT_FILE).unwrap();

    for character in (0_u8..=255_u8).map(char::from) {
        let (a, _) = font.rasterize_sdf(character, PX);
        let b = font.metrics_sdf(character, PX);
        assert_eq!(a, b, "character was: {character}");
    }
}
