// This SDF rasterizer is not the most optimal,
// but this is simple and gets the job done

#![no_std]

//

extern crate alloc;

//

use alloc::{vec, vec::Vec};
use core::num::NonZeroU16;
use fontdue::FontSettings;
use geom::Geometry;
use glam::{UVec4, Vec4};
use hashbrown::HashMap;
use math::Line;
use ttf_parser::{Face, Rect};

//

pub use fontdue::{Metrics, OutlineBounds};

use self::math::bvec4_to_uvec4;

//

pub mod geom;
pub mod math;

pub const CURVE_RESOLUTION: usize = 8;

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
    pub fn inner(&self) -> &fontdue::Font {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut fontdue::Font {
        &mut self.inner
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let inner = fontdue::Font::from_bytes(bytes, FontSettings::default())?;
        let face = Face::parse(bytes, 0).unwrap();

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
        let mut glyphs = Vec::new();
        glyphs.resize(face.number_of_glyphs() as usize, initial);
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

    pub fn chars(&self) -> &HashMap<char, NonZeroU16> {
        self.inner.chars()
    }

    pub fn scale_factor(&self, px: f32) -> f32 {
        px * self.oo_units_per_em
    }

    pub fn radius(&self, px: f32) -> usize {
        let scale_factor = self.scale_factor(px);
        (255.0 * scale_factor).ceil() as usize + 1
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
        let (geom, bb) = self.geometry_indexed(index);

        let metrics = self.internal_metrics(px, bb);
        let inv_sf = 1.0 / metrics.sf;

        // process in chunks of 4
        let w = metrics.width as u32;
        let h = metrics.height as u32;
        let chunk_count = (w * h).div_ceil(4); // divide and round UP, (more if the last chunk doesn't cover everything)
        let mut image = vec![0; (chunk_count * 4) as usize]; // maybe not zero init as they are not read before written to
        for (idx, p) in (0..w * h / 4).map(|i| i * 4).map(|i| {
            (
                i as usize,
                (
                    (UVec4::new(i % w, (i + 1) % w, (i + 2) % w, (i + 3) % w).as_vec4()
                        - metrics.radius as f32
                        + metrics.offset_x)
                        * inv_sf,
                    (UVec4::new(
                        h - 1 - i / w,
                        h - 1 - (i + 1) / w,
                        h - 1 - (i + 2) / w,
                        h - 1 - (i + 3) / w,
                    )
                    .as_vec4()
                        - metrics.radius as f32
                        + metrics.offset_y)
                        * inv_sf,
                ),
            )
        }) {
            let is_inside = geom.is_inside(p);

            // if false {
            //     image[idx] = is_inside.test(0) as u8 * 255;
            //     image[idx + 1] = is_inside.test(1) as u8 * 255;
            //     image[idx + 2] = is_inside.test(2) as u8 * 255;
            //     image[idx + 3] = is_inside.test(3) as u8 * 255;
            //     continue;
            // }

            let distance_squared = geom
                .iter_lines()
                .map(|s| s.distance_ord(p))
                .reduce(|acc, next| acc.min(next))
                .unwrap_or(Vec4::ONE);

            // invert pixels that are 'inside' the geometry
            let sign = bvec4_to_uvec4(is_inside).as_vec4() * 2.0 - 1.0;
            let d = Line::distance_finalize(distance_squared) * 0.5 * sign;

            // convert to pixels
            let distances = (d + Vec4::splat(128.0)).as_uvec4();
            image[idx] = distances.x as u8;
            image[idx + 1] = distances.y as u8;
            image[idx + 2] = distances.z as u8;
            image[idx + 3] = distances.w as u8;
        }

        // cut out those extra pixels
        image.truncate((w * h) as usize);

        (
            self.modify_metrics(index, px, metrics.radius, metrics.width, metrics.height),
            image,
        )
    }

    pub fn geometry(&self, character: char) -> &'_ (Geometry, Rect) {
        self.geometry_indexed(self.lookup_glyph_index(character))
    }

    pub fn geometry_indexed(&self, index: u16) -> &'_ (Geometry, Rect) {
        self.glyphs.get(index as usize).unwrap()
    }

    #[inline(always)]
    pub fn metrics(&self, character: char, px: f32, sdf: bool) -> Metrics {
        if sdf {
            self.metrics_sdf(character, px)
        } else {
            self.inner.metrics(character, px)
        }
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

    pub fn lookup_glyph_index(&self, ch: char) -> u16 {
        self.inner.lookup_glyph_index(ch)
    }

    fn internal_metrics(&self, px: f32, bb: &Rect) -> InternalMetrics {
        let sf = self.scale_factor(px);
        let radius = self.radius(px);
        let offset_x = bb.x_min as f32 * sf;
        let offset_y = bb.y_min as f32 * sf;

        let width = bb.x_max.abs_diff(bb.x_min);
        let height = bb.y_max.abs_diff(bb.y_min);

        let width = if width == 0 {
            0
        } else {
            (width as f32 * sf) as usize + radius * 2
        };
        let height = if height == 0 {
            0
        } else {
            (height as f32 * sf) as usize + radius * 2
        };

        InternalMetrics {
            sf,
            radius,
            offset_x,
            offset_y,
            width,
            height,
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
