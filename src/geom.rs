use crate::math::{bvec4_to_uvec4, Line, Ray, Shape};
use glam::{BVec4A, UVec4, Vec2, Vec4};
use ttf_parser::OutlineBuilder;

//

#[derive(Debug, Clone, Default)]
pub struct Geometry {
    current: Vec2,
    min_x: f32,
    shapes: Vec<Shape>,
}

//

impl Geometry {
    pub fn new() -> Self {
        Self::default()
    }

    /// check if the point is 'inside' this character
    /// by drawing a line to left and checking the collision count
    ///
    /// if the collision count is divisible by 2
    /// the point is outside of the character
    ///
    /// illustration in the source code bellow:
    //
    //                     +-----------------+
    //                     |                 |
    //       one collision |  another one    |
    //                  \  |  \              |
    //                   \ |   \ +-----+     |
    //                    \|    \|     |     |
    //                o====|=====|==o  |     |
    //                     |     |     |     |
    //   two collisions    |     +-----+     |
    //   so the point is   |                 |
    //   outside of the    |                 |
    //   character         |                 |
    //                     |     +-----+     |
    //                     |     |     |     |
    //                     |     |     |     |
    //      one collision  |     |     |     |
    //                   \ |     +-----+     |
    //                    \|                 |
    //   this point   o====|=====o           |
    //   is inside         |                 |
    //   the character     +-----------------+
    pub fn is_inside(&self, point: (Vec4, Vec4)) -> BVec4A {
        let half = Vec4::ONE * 0.5;

        let from = (half + point.0.round(), half + point.1.round());

        let ray = Ray {
            from_x: from.0,
            from_y: from.1,
            to_x: half + Vec4::splat(self.min_x - 100.0),
            to_y: from.1,
        };

        let mut hit_counts = UVec4::ZERO;

        for shape in self.shapes.iter() {
            hit_counts += bvec4_to_uvec4(ray.collision(*shape));
        }

        BVec4A::new(
            hit_counts.x % 2 == 1,
            hit_counts.y % 2 == 1,
            hit_counts.z % 2 == 1,
            hit_counts.w % 2 == 1,
        )
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        self.iter_shapes().flat_map(|shape| shape.iter_lines())
    }

    pub fn iter_shapes(&self) -> impl Iterator<Item = Shape> + '_ {
        self.shapes.iter().copied()
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
        self.shapes.push(
            Line {
                from: self.current,
                to,
            }
            .into(),
        );
        self.current = to;
        self.min_x = self.min_x.min(to.x);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let by = Vec2::new(x1, y1).round();
        let to = Vec2::new(x, y).round();
        self.shapes.push(Shape::Quad {
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
        self.shapes.push(Shape::Curve {
            from: self.current,
            by_a,
            by_b,
            to,
        });
        self.current = to;
        self.min_x = self.min_x.min(by_a.x).min(by_b.x).min(to.x);
    }

    fn close(&mut self) {}
}
