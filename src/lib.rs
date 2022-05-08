// This SDF rasterizer is not the most optimal,
// but this is simple and gets the job done

use fontdue::FontSettings;
use geom::Geometry;
use glam::Vec2;
use ordered_float::OrderedFloat;
use std::ops::{Deref, DerefMut};
use ttf_parser::{Face, Rect};

//

pub use fontdue::{Metrics, OutlineBounds};

//

pub mod geom;
pub mod math;

//

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: Vec<(Geometry, Rect)>,
    oo_units_per_em: f32,

    inner: fontdue::Font,
}

struct InternalMetrics {
    sf: f32,
    radius: usize,
    offset_x: f32,
    offset_y: f32,
    width: usize,
    height: usize,
}

//

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let inner = fontdue::Font::from_bytes(bytes, FontSettings::default())?;
        let face = Face::from_slice(bytes, 0).unwrap();

        let oo_units_per_em = 1.0 / face.units_per_em() as f32;

        let initial = (
            Geometry::new(),
            Rect {
                x_min: 0,
                y_min: 0,
                x_max: 0,
                y_max: 0,
            },
        );
        let mut glyphs = vec![initial; face.number_of_glyphs() as usize];
        for (&c, &i) in inner.chars().iter() {
            (|| {
                let mut geom = Geometry::new();
                let bb = face.outline_glyph(face.glyph_index(c)?, &mut geom)?;
                glyphs[i.get() as usize] = (geom, bb);
                Some(())
            })();
        }

        Ok(Self {
            glyphs,
            oo_units_per_em,

            inner,
        })
    }

    pub fn scale_factor(&self, px: f32) -> f32 {
        px * self.oo_units_per_em
    }

    pub fn radius(&self, px: f32) -> usize {
        let scale_factor = self.scale_factor(px);
        (255.0 * scale_factor).ceil() as _
    }

    #[inline(always)]
    pub fn metrics(&self, character: char, px: f32, sdf: bool) -> Metrics {
        if sdf {
            self.metrics_sdf(character, px)
        } else {
            self.inner.metrics(character, px)
        }
    }

    pub fn metrics_sdf(&self, character: char, px: f32) -> Metrics {
        self.metrics_indexed_sdf(self.lookup_glyph_index(character), px)
    }

    pub fn metrics_indexed_sdf(&self, index: u16, px: f32) -> Metrics {
        let (_, bb) = self.glyphs.get(index as usize).unwrap();
        let metrics = self.internal_metrics(px, bb);
        self.modify_metrics(index, px, metrics.radius, metrics.width, metrics.height)
    }

    pub fn rasterize_sdf(&self, character: char, px: f32) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed_sdf(self.lookup_glyph_index(character), px)
    }

    pub fn rasterize_indexed_sdf(&self, index: u16, px: f32) -> (Metrics, Vec<u8>) {
        let (geom, bb) = self.glyphs.get(index as usize).unwrap();

        let metrics = self.internal_metrics(px, bb);

        let image = (0..metrics.height)
            .rev()
            .flat_map(|y| (0..metrics.width).map(move |x| (x, y)))
            .map(|(x, y)| {
                Vec2::new(
                    x as f32 - metrics.radius as f32 + metrics.offset_x,
                    y as f32 - metrics.radius as f32 + metrics.offset_y,
                ) / metrics.sf
            })
            .map(|p| {
                let is_inside = geom.is_inside(p);
                let mut x = geom
                    .iter_lines()
                    .map(move |s| s.distance(p) * 0.5)
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

        (
            self.modify_metrics(index, px, metrics.radius, metrics.width, metrics.height),
            image,
        )
    }

    #[inline(always)]
    pub fn metrics_indexed(&self, index: u16, px: f32, sdf: bool) -> Metrics {
        if sdf {
            self.metrics_indexed_sdf(index, px)
        } else {
            self.inner.metrics_indexed(index, px)
        }
    }

    #[inline(always)]
    pub fn rasterize(&self, character: char, px: f32, sdf: bool) -> (Metrics, Vec<u8>) {
        if sdf {
            self.rasterize_sdf(character, px)
        } else {
            self.inner.rasterize(character, px)
        }
    }

    #[inline(always)]
    pub fn rasterize_indexed(&self, index: u16, px: f32, sdf: bool) -> (Metrics, Vec<u8>) {
        if sdf {
            self.rasterize_indexed_sdf(index, px)
        } else {
            self.inner.rasterize_indexed(index, px)
        }
    }

    fn internal_metrics(&self, px: f32, bb: &Rect) -> InternalMetrics {
        let sf = self.scale_factor(px);
        let radius = self.radius(px);
        let offset_x = bb.x_min as f32 * sf;
        let offset_y = bb.y_min as f32 * sf;
        InternalMetrics {
            sf,
            radius,
            offset_x,
            offset_y,
            width: (bb.x_max as f32 * sf - offset_x) as usize + radius * 2,
            height: (bb.y_max as f32 * sf - offset_y) as usize + radius * 2,
        }
    }

    fn modify_metrics(
        &self,
        index: u16,
        px: f32,
        radius: usize,
        width: usize,
        height: usize,
    ) -> Metrics {
        let mut metrics = self.inner.metrics_indexed(index, px);
        metrics.xmin -= radius as i32;
        metrics.ymin -= radius as i32;
        metrics.width = width;
        metrics.height = height;
        metrics
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
