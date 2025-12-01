use glam::{BVec4A, UVec4, Vec2, Vec4};

use crate::geom::Contour;

//

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub from: Vec2,
    pub to: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub from: Vec2,
    pub by: Vec2,
    pub to: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct Curve {
    pub from: Vec2,
    pub by_a: Vec2,
    pub by_b: Vec2,
    pub to: Vec2,
}

#[derive(Debug, Clone, Copy, Default)]
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

pub trait Segment: Sized {
    fn aabb(self) -> BoundingBox;
    fn iter_lines(self, resolution: usize) -> impl ExactSizeIterator<Item = Line>;
    fn control_points(self) -> impl ExactSizeIterator<Item = Vec2>;
}

//

impl BoundingBox {
    pub fn aabb(self, other: Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpge(other.min).all()
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// squared distance from a point to the
    /// furthest point on this bounding box
    pub fn max_distance_squared(self, points: (Vec4, Vec4)) -> Vec4 {
        let mid = (self.min + self.max) * 0.5;
        let point_side = (points.0 - Vec4::splat(mid.x), points.1 - Vec4::splat(mid.y));

        let point_side_x_is_negative =
            bvec4_to_uvec4(point_side.0.cmple(Vec4::splat(0.0))).as_vec4();
        let point_side_y_is_negative =
            bvec4_to_uvec4(point_side.1.cmple(Vec4::splat(0.0))).as_vec4();

        // i had to make it manually branchless
        // because simd makes it harder not to
        let x = points.0
            - point_side_x_is_negative * Vec4::splat(self.max.x)
            - (Vec4::splat(1.0) - point_side_x_is_negative) * Vec4::splat(self.min.x);
        // ==
        /* let x = point.x
        - if point_side.x <= 0.0 {
            self.max.x
        } else {
            self.min.x
        }; */
        let y = points.1
            - point_side_y_is_negative * Vec4::splat(self.max.y)
            - (Vec4::splat(1.0) - point_side_y_is_negative) * Vec4::splat(self.min.y);

        x * x + y * y
    }

    /// squared distance from a point to the
    /// furthest point on this bounding box
    ///
    /// shamelessly stolen (and modified) from: https://stackoverflow.com/a/18157551
    pub fn min_distance_squared(self, points: (Vec4, Vec4)) -> Vec4 {
        let x = (Vec4::splat(self.min.x) - points.0)
            .max(Vec4::splat(0.0))
            .max(points.0 - Vec4::splat(self.max.x));
        let y = (Vec4::splat(self.min.y) - points.1)
            .max(Vec4::splat(0.0))
            .max(points.1 - Vec4::splat(self.max.y));

        x * x + y * y
    }
}

impl Ray {
    pub fn hit_count(self, other: &Contour) -> Vec4 {
        let bb_min_x = self.from_x.min(self.to_x);
        let bb_min_y = self.from_y.min(self.to_y);
        let bb_max_x = self.from_x.max(self.to_x);
        let bb_max_y = self.from_y.max(self.to_y);

        let bb_other = other.aabb;

        // check for collisions for multiple bounding boxes per one bounding box
        // if any of them collide, then continue
        let collisions = (bb_min_x.cmple(Vec4::splat(bb_other.max.x)).bitmask()
            & bb_max_x.cmpge(Vec4::splat(bb_other.min.x)).bitmask()
            & bb_min_y.cmple(Vec4::splat(bb_other.max.y)).bitmask()
            & bb_max_y.cmpge(Vec4::splat(bb_other.min.y)).bitmask())
            != 0;
        if !collisions {
            return Vec4::ZERO;
        }

        let mut result = Vec4::ZERO;
        for line in other.lines.iter() {
            let side = line.side((self.from_x, self.from_y)).signum();
            let intersects = bvec4_to_uvec4(line.line_ray_intersection(self)).as_vec4();
            result -= side * intersects;
        }
        result
    }
}

impl Segment for Line {
    fn aabb(self) -> BoundingBox {
        BoundingBox {
            min: self.from.min(self.to),
            max: self.from.max(self.to),
        }
    }

    fn iter_lines(self, _: usize) -> impl ExactSizeIterator<Item = Line> {
        core::iter::once(self)
    }

    fn control_points(self) -> impl ExactSizeIterator<Item = Vec2> {
        [self.from, self.to].into_iter()
    }
}

impl Segment for Quad {
    fn aabb(self) -> BoundingBox {
        BoundingBox {
            min: self.from.min(self.by).min(self.to),
            max: self.from.max(self.by).max(self.to),
        }
    }

    fn iter_lines(self, resolution: usize) -> impl ExactSizeIterator<Item = Line> {
        let step = 1.0 / resolution as f32;
        let mut prev = self.from;
        let mut t = step;

        core::iter::repeat_n((), resolution).map(move |_| {
            let from_by = self.from.lerp(self.by, t);
            let by_to = self.by.lerp(self.to, t);
            let next = from_by.lerp(by_to, t);
            let result = Line {
                from: prev.round(),
                to: next.round(),
            };
            prev = next;
            t += step;
            result
        })
    }

    fn control_points(self) -> impl ExactSizeIterator<Item = Vec2> {
        [self.from, self.by, self.to].into_iter()
    }
}

impl Segment for Curve {
    fn aabb(self) -> BoundingBox {
        BoundingBox {
            min: self.from.min(self.by_a).min(self.by_b).min(self.to),
            max: self.from.max(self.by_a).max(self.by_b).max(self.to),
        }
    }

    fn iter_lines(self, resolution: usize) -> impl ExactSizeIterator<Item = Line> {
        let step = 1.0 / resolution as f32;
        let mut prev = self.from;
        let mut t = step;

        core::iter::repeat_n((), resolution).map(move |_| {
            let from_by_a = self.from.lerp(self.by_a, t);
            let by_a_by_b = self.by_a.lerp(self.by_b, t);
            let by_b_to = self.by_b.lerp(self.to, t);

            let from_by_a_by_a_by_b = from_by_a.lerp(by_a_by_b, t);
            let by_a_by_b_by_b_to = by_a_by_b.lerp(by_b_to, t);

            let next = from_by_a_by_a_by_b.lerp(by_a_by_b_by_b_to, t);
            let result = Line {
                from: prev.round(),
                to: next.round(),
            };
            prev = next;
            t += step;
            result
        })
    }

    fn control_points(self) -> impl ExactSizeIterator<Item = Vec2> {
        [self.from, self.by_a, self.by_b, self.to].into_iter()
    }
}

impl Line {
    // TODO:
    // this is critical code
    // this gets ran so many times
    //
    // also optimize this if possible
    pub fn distance_ord(self, p: (Vec4, Vec4)) -> Vec4 {
        let a = (Vec4::splat(self.from.x), Vec4::splat(self.from.y));
        let b = (Vec4::splat(self.to.x), Vec4::splat(self.to.y));
        let a_to_p = (p.0 - a.0, p.1 - a.1);
        let a_to_b = (b.0 - a.0, b.1 - a.1);

        let t = ((a_to_p.0 * a_to_b.0 + a_to_p.1 * a_to_b.1)
            / (a_to_b.0.powf(2.0) + a_to_b.1.powf(2.0)))
        .min(Vec4::splat(1.0))
        .max(Vec4::splat(0.0));

        let tmp = ((a.0 + a_to_b.0 * t) - p.0, (a.1 + a_to_b.1 * t) - p.1);
        tmp.0 * tmp.0 + tmp.1 * tmp.1
    }

    pub fn side(self, p: (Vec4, Vec4)) -> Vec4 {
        let a = (Vec4::splat(self.from.x), Vec4::splat(self.from.y));
        let b = (Vec4::splat(self.to.x), Vec4::splat(self.to.y));
        ((b.0 - a.0) * (p.1 - a.1) - (p.0 - a.0) * (b.1 - a.1)).signum()
    }

    pub fn distance_finalize(d: Vec4) -> Vec4 {
        d.powf(0.5)
    }

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

pub fn uvec4_to_bvec4(v: UVec4) -> BVec4A {
    BVec4A::new(
        v.x != 0,
        v.y != 0,
        v.z != 0,
        v.w != 0,
        //
    )
}
