use fontsdf::Font;
use image::{GenericImage, GrayImage};

//

const FONT_FILE: &[u8] = include_bytes!("../res/roboto/font.ttf");

//

#[test]
fn main_test() {
    let font = Font::from_bytes(FONT_FILE).unwrap();

    let (metrics, simple) = font.rasterize('#', 128.0, false);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, simple)
        .unwrap()
        .save("main_test_1.png")
        .unwrap();

    let (metrics, sdf) = font.rasterize('#', 128.0, true);
    GrayImage::from_raw(metrics.width as _, metrics.height as _, sdf)
        .unwrap()
        .save("main_test_2.png")
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

#[test]
fn all_chars() {
    let font = Font::from_bytes(FONT_FILE).unwrap();

    let images: Vec<_> = (0_u8..=255_u8)
        .map(char::from)
        .filter_map(|character| {
            let (metrics, sdf) = font.rasterize(character, 64.0, true);
            let name = format!("test_all_chars_{}", character as u16);
            println!("{name}");
            Some((
                GrayImage::from_raw(metrics.width as _, metrics.height as _, sdf)?,
                name,
            ))
        })
        .collect();

    let width: u32 = images.iter().map(|(image, _)| image.width()).max().unwrap();
    let height: u32 = images.iter().map(|(image, _)| image.height()).sum();

    let mut combined = GrayImage::new(width, height);
    let mut y = 0;
    for (image, _) in images {
        combined
            .sub_image(0, y, image.width(), image.height())
            .copy_from(&image, 0, 0)
            .unwrap();
        y += image.height();
    }

    combined.save("test_all_chars.png").unwrap();
}
