use glam::{BVec4A, UVec4, Vec2, Vec4};

//

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Line {
        line: Line,

        bb: BoundingBox,
    },

    Quad {
        from: Vec2,
        by: Vec2,
        to: Vec2,

        bb: BoundingBox,
    },

    Curve {
        from: Vec2,
        by_a: Vec2,
        by_b: Vec2,
        to: Vec2,

        bb: BoundingBox,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub from: Vec2,
    pub to: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
}

/// 4 rays packed to allow simd
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub from_x: Vec4,
    pub from_y: Vec4,
    pub to_x: Vec4,
    pub to_y: Vec4,
}

//

impl BoundingBox {
    pub fn aabb(self, other: Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// returns the squared distance from the furthest point to this point
    ///
    /// shamelessly stolen (and modified) from: https://stackoverflow.com/a/18157551
    pub fn max_distance_squared(self, point: Vec2) -> f32 {
        ((self.min - point).max(Vec2::ZERO).max(point - self.max) /* + (self.min - self.max).abs() */)
            .length_squared()
    }
}

impl Ray {
    pub fn collision(self, other: Shape) -> BVec4A {
        let bb_min_x = self.from_x.min(self.to_x);
        let bb_min_y = self.from_y.min(self.to_y);
        let bb_max_x = self.from_x.max(self.to_x);
        let bb_max_y = self.from_y.max(self.to_y);

        let bb_other = other.bounding_box();

        // check for collisions for multiple bounding boxes per one bounding box
        // if any of them collide, then continue
        let collisions = (bb_min_x.cmple(Vec4::splat(bb_other.max.x)).bitmask()
            & bb_max_x.cmpge(Vec4::splat(bb_other.min.x)).bitmask()
            & bb_min_y.cmple(Vec4::splat(bb_other.max.y)).bitmask()
            & bb_max_y.cmpge(Vec4::splat(bb_other.min.y)).bitmask())
            != 0;
        if !collisions {
            return BVec4A::default(); // false
        }

        let mut result = BVec4A::default(); // false
        for line in other.iter_lines() {
            result |= line.line_ray_intersection(self);
        }
        result
    }
}

impl Shape {
    pub fn bounding_box(self) -> BoundingBox {
        match self {
            Shape::Line { bb, .. } => bb,
            Shape::Quad { bb, .. } => bb,
            Shape::Curve { bb, .. } => bb,
        }
    }

    pub fn iter_lines(self) -> impl Iterator<Item = Line> {
        enum ShapeIter<I0, I1, I2>
        where
            I0: Iterator<Item = Line>,
            I1: Iterator<Item = Line>,
            I2: Iterator<Item = Line>,
        {
            I0(I0),
            I1(I1),
            I2(I2),
        }

        impl<I0, I1, I2> Iterator for ShapeIter<I0, I1, I2>
        where
            I0: Iterator<Item = Line>,
            I1: Iterator<Item = Line>,
            I2: Iterator<Item = Line>,
        {
            type Item = Line;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    ShapeIter::I0(i0) => i0.next(),
                    ShapeIter::I1(i1) => i1.next(),
                    ShapeIter::I2(i2) => i2.next(),
                }
            }
        }

        const RES: usize = 8;
        const STEP: f32 = 1.0 / RES as f32;

        match self {
            // just a line
            Shape::Line { line, .. } => ShapeIter::I0(Some(line).into_iter()),

            // bézier curve with 1 control point
            Shape::Quad { from, by, to, .. } => {
                let mut prev = from;

                ShapeIter::I1((1..=RES).map(|i| i as f32 * STEP).map(move |t| {
                    let from_by = from.lerp(by, t);
                    let by_to = by.lerp(to, t);
                    let next = from_by.lerp(by_to, t);
                    let result = Line {
                        from: prev,
                        to: next,
                    };
                    prev = next;
                    result
                }))
            }

            // bézier curve with 2 control points
            Shape::Curve {
                from,
                by_a,
                by_b,
                to,
                ..
            } => {
                let mut prev = from;

                ShapeIter::I2((1..=RES).map(|i| i as f32 * STEP).map(move |t| {
                    let from_by_a = from.lerp(by_a, t);
                    let by_a_by_b = by_a.lerp(by_b, t);
                    let by_b_to = by_b.lerp(to, t);

                    let from_by_a_by_a_by_b = from_by_a.lerp(by_a_by_b, t);
                    let by_a_by_b_by_b_to = by_a_by_b.lerp(by_b_to, t);

                    let next = from_by_a_by_a_by_b.lerp(by_a_by_b_by_b_to, t);
                    let result = Line {
                        from: prev,
                        to: next,
                    };
                    prev = next;
                    result
                }))
            }
        }
    }
}

impl Line {
    // TODO:
    // this is critical code
    // this gets ran so many times
    //
    // also optimize this if possible
    pub fn distance_ord(&self, p: (Vec4, Vec4)) -> Vec4 {
        let a = (Vec4::splat(self.from.x), Vec4::splat(self.from.y));
        let b = (Vec4::splat(self.to.x), Vec4::splat(self.to.y));
        let a_to_p = (p.0 - a.0, p.1 - a.1);
        let a_to_b = (b.0 - a.0, b.1 - a.1);

        let t = ((a_to_p.0 * a_to_b.0 + a_to_p.1 * a_to_b.1)
            / (a_to_b.0.powf(2.0) + a_to_b.1.powf(2.0)))
        .min(Vec4::splat(1.0))
        .max(Vec4::splat(0.0));

        ((a.0 + a_to_b.0 * t) - p.0).powf(2.0) + ((a.1 + a_to_b.1 * t) - p.1).powf(2.0)
    }

    pub fn distance_finalize(d: Vec4) -> Vec4 {
        d.powf(0.5)
    }
}

impl Line {
    /// shamelessly stolen from: https://gamedev.stackexchange.com/a/26022
    fn line_ray_intersection(self, other: Ray) -> BVec4A {
        let point_a1 = (Vec4::splat(self.from.x), Vec4::splat(self.from.y));
        let point_a2 = (Vec4::splat(self.to.x), Vec4::splat(self.to.y));
        let point_b1 = (other.from_x, other.from_y);
        let point_b2 = (other.to_x, other.to_y);

        let a1_a2 = (point_a2.0 - point_a1.0, point_a2.1 - point_a1.1);
        let b1_b2 = (point_b2.0 - point_b1.0, point_b2.1 - point_b1.1);
        let b1_a1 = (point_a1.0 - point_b1.0, point_a1.1 - point_b1.1);

        let denominator = a1_a2.0 * b1_b2.1 - a1_a2.1 * b1_b2.0;
        let numerator1 = b1_a1.1 * b1_b2.0 - b1_a1.0 * b1_b2.1;
        let numerator2 = b1_a1.1 * a1_a2.0 - b1_a1.0 * a1_a2.1;

        let r = numerator1 / denominator;
        let s = numerator2 / denominator;

        Vec4::splat(0.0).cmple(r)
            & r.cmple(Vec4::splat(1.0))
            & Vec4::splat(0.0).cmple(s)
            & s.cmple(Vec4::splat(1.0))
    }
}

impl From<Line> for Shape {
    fn from(line: Line) -> Self {
        Self::Line {
            line,
            bb: BoundingBox {
                min: line.from.min(line.to),
                max: line.from.max(line.to),
            },
        }
    }
}

//

pub fn bvec4_to_uvec4(v: BVec4A) -> UVec4 {
    let v = v.bitmask();
    UVec4::new(
        v & 0b1,
        (v & 0b10) >> 1,
        (v & 0b100) >> 2,
        (v & 0b1000) >> 3,
    )
}
