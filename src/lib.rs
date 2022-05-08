// This SDF rasterizer is not the most optimal,
// but this is simple and gets the job done

use fontdue::FontSettings;
use geom::Geometry;
use glam::Vec2;
use ordered_float::OrderedFloat;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use ttf_parser::{Face, Rect};

//

pub use fontdue::{Metrics, OutlineBounds};

//

mod geom;
mod math;

//

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: HashMap<char, (Geometry, Rect)>,
    oo_units_per_em: f32,

    inner: fontdue::Font,
}

impl Font {
    pub fn form_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let inner = fontdue::Font::from_bytes(bytes, FontSettings::default())?;
        let face = Face::from_slice(bytes, 0).unwrap();

        let oo_units_per_em = 1.0 / face.units_per_em() as f32;

        let glyphs = inner
            .chars()
            .keys()
            .filter_map(|&c| {
                let glyph_id = face.glyph_index(c)?;
                let mut geom = Geometry::new();
                let bb = face.outline_glyph(glyph_id, &mut geom)?;
                Some((c, (geom, bb)))
            })
            .collect();

        Ok(Self {
            glyphs,
            oo_units_per_em,

            inner,
        })
    }

    pub fn rasterize_sdf(&self, code_point: char, px: f32) -> (Metrics, Vec<u8>) {
        let scale_factor = px * self.oo_units_per_em;
        let (geom, bb) = self.glyphs.get(&code_point).unwrap();

        let radius = (128.0 * scale_factor).ceil() as usize;
        let offset_x = bb.x_min as f32 * scale_factor;
        let offset_y = bb.y_min as f32 * scale_factor;
        let width = (bb.x_max as f32 * scale_factor - offset_x) as usize + radius * 2;
        let height = (bb.y_max as f32 * scale_factor - offset_y) as usize + radius * 2;

        let image = (0..height)
            .rev()
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| {
                Vec2::new(
                    x as f32 - radius as f32 + offset_x,
                    y as f32 - radius as f32 + offset_y,
                ) / scale_factor
            })
            .map(|p| {
                let is_inside = geom.is_inside(p);
                let mut x = geom
                    .iter_lines()
                    .map(move |s| s.distance(p))
                    .map(OrderedFloat)
                    .min()
                    .unwrap_or(OrderedFloat(0.0))
                    .0;

                if !is_inside {
                    x *= -1.0;
                }

                (x + 128.0) as u8
            })
            .collect();

        let mut metrics = self.inner.metrics(code_point, px);

        metrics.xmin -= radius as i32;
        metrics.ymin -= radius as i32;
        metrics.width = width;
        metrics.height = height;

        (metrics, image)
    }
}

//

impl Deref for Font {
    type Target = fontdue::Font;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Font {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
