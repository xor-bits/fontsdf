use crate::{
    CURVE_RESOLUTION,
    math::{BoundingBox, Curve, Line, Quad, Ray, Segment},
};
use alloc::vec::Vec;
use glam::{BVec4A, Vec2, Vec4};
use ttf_parser::OutlineBuilder;

//

#[derive(Debug, Clone, Default)]
pub struct Geometry {
    current: Vec2,
    min_x: f32,
    contours: Vec<Contour>,

    current_contour: Contour,
    current_contour_edge_sum: f32,
    current_contour_first_control_point: Option<Vec2>,
    prev_control_point: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct Contour {
    pub aabb: BoundingBox,
    pub lines: Vec<Line>,
    pub mode: ContourMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContourMode {
    #[default]
    Additive,
    Subtractive,
}

//

impl Geometry {
    pub fn new() -> Self {
        Self::default()
    }

    /// check if the point is 'inside' this character
    /// by drawing a line to left and for each entry:
    ///  - increase counter by 1 if entering a contour
    ///  - decrease counter by 1 if exiting a contour
    ///
    /// if counter is greater than zero,
    /// then the point was inside the character
    pub fn is_inside(&self, point: (Vec4, Vec4)) -> BVec4A {
        let half = Vec4::ONE * 0.5;

        let from = (half + point.0.round(), half + point.1.round());

        let ray = Ray {
            from_x: from.0,
            from_y: from.1,
            to_x: half + Vec4::splat(self.min_x - 100.0),
            to_y: from.1,
        };

        let mut hit_counts = Vec4::ZERO;
        for contour in self.contours.iter() {
            hit_counts += ray.hit_count(contour);
        }
        hit_counts.cmpgt(Vec4::ZERO)
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        self.iter_parts()
            .flat_map(|shape| shape.lines.iter())
            .copied()
    }

    pub fn iter_parts(&self) -> impl Iterator<Item = &'_ Contour> + '_ {
        self.contours.iter()
    }

    pub fn add_shape(&mut self, shape: impl Segment + Copy) {
        let iter = shape.iter_lines(CURVE_RESOLUTION);
        self.current_contour.lines.reserve(iter.len());
        self.current_contour.lines.extend(iter);
        self.current_contour.aabb = self.current_contour.aabb.union(shape.aabb());

        let mut iter = shape.control_points();
        let first = iter.next();
        if let Some(first) = first
            && self.current_contour_first_control_point.is_none()
        {
            self.current_contour_first_control_point = Some(first);
            self.prev_control_point = first;
        }

        for extra_control_point in iter {
            self.current_contour_edge_sum += (extra_control_point.x - self.prev_control_point.x)
                * (extra_control_point.y + self.prev_control_point.y);
            self.prev_control_point = extra_control_point;
        }
    }

    pub fn finish_part(&mut self) {
        if let Some(first_control_point) = self.current_contour_first_control_point {
            self.current_contour_edge_sum += (first_control_point.x - self.prev_control_point.x)
                * (first_control_point.y + self.prev_control_point.y);
        }
        self.current_contour.mode = if self.current_contour_edge_sum >= 0.0 {
            ContourMode::Additive
        } else {
            ContourMode::Subtractive
        };
        self.current_contour_edge_sum = 0.0;
        self.current_contour_first_control_point = None;
        self.prev_control_point = Vec2::ZERO;
        self.contours
            .push(core::mem::take(&mut self.current_contour));
    }
}

impl OutlineBuilder for Geometry {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = Vec2::new(x, y).round();
        self.current = to;
        self.min_x = self.min_x.min(to.x);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = Vec2::new(x, y).round();
        self.add_shape(Line {
            from: self.current,
            to,
        });
        self.current = to;
        self.min_x = self.min_x.min(to.x);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let by = Vec2::new(x1, y1).round();
        let to = Vec2::new(x, y).round();
        self.add_shape(Quad {
            from: self.current,
            by,
            to,
        });
        self.current = to;
        self.min_x = self.min_x.min(by.x).min(to.x);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let by_a = Vec2::new(x1, y1).round();
        let by_b = Vec2::new(x2, y2).round();
        let to = Vec2::new(x, y).round();
        self.add_shape(Curve {
            from: self.current,
            by_a,
            by_b,
            to,
        });
        self.current = to;
        self.min_x = self.min_x.min(by_a.x).min(by_b.x).min(to.x);
    }

    fn close(&mut self) {
        self.finish_part();
    }
}
