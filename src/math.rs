use glam::Vec2;

//

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Line(Line),

    Quad {
        from: Vec2,
        by: Vec2,
        to: Vec2,
    },

    Curve {
        from: Vec2,
        by_a: Vec2,
        by_b: Vec2,
        to: Vec2,
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

//

impl BoundingBox {
    pub fn aabb(self, other: Self) -> bool {
        (self.min.x < other.max.x && self.max.x >= other.min.x)
            && (self.min.y < other.max.y && self.max.y >= other.min.y)
    }

    /// returns the squared distance from the furthest point to this point
    ///
    /// shamelessly stolen (and modified) from: https://stackoverflow.com/a/18157551
    pub fn max_distance_squared(self, point: Vec2) -> f32 {
        ((self.min - point).max(Vec2::ZERO).max(point - self.max) /* + (self.min - self.max).abs() */)
            .length_squared()
    }
}

impl Shape {
    pub fn collision(self, other: Shape) -> bool {
        if !self.bounding_box().aabb(other.bounding_box()) {
            return false;
        }

        // note: lines have only one line obviously
        // so collision detection from line to
        // shape is pretty fast
        self.iter_lines()
            .flat_map(|a| other.iter_lines().map(move |b| (a, b)))
            .any(|(a, b)| a.line_line_intersection(b))
    }

    pub fn bounding_box(self) -> BoundingBox {
        match self {
            Shape::Line(Line { from, to }) => BoundingBox {
                min: from.min(to),
                max: from.max(to),
            },
            Shape::Quad { from, by, to } => BoundingBox {
                min: from.min(by).min(to),
                max: from.max(by).max(to),
            },
            Shape::Curve {
                from,
                by_a,
                by_b,
                to,
            } => BoundingBox {
                min: from.min(by_a).min(by_b).min(to),
                max: from.max(by_a).max(by_b).max(to),
            },
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
            Shape::Line(line) => ShapeIter::I0(Some(line).into_iter()),

            // bézier curve with 1 control point
            Shape::Quad { from, by, to } => {
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
    pub fn distance_ord(&self, p: Vec2) -> f32 {
        let a = self.from;
        let b = self.to;
        let a_to_p = p - a;
        let a_to_b = b - a;

        let t = (a_to_p.dot(a_to_b) / a_to_b.length_squared())
            .min(1.0)
            .max(0.0);

        ((a + a_to_b * t) - p).length_squared()
    }

    pub fn distance_finalize(d: f32) -> f32 {
        d.sqrt()
    }

    /// shamelessly stolen from: https://gamedev.stackexchange.com/a/26022
    fn line_line_intersection(self, other: Self) -> bool {
        let point_a1 = self.from;
        let point_a2 = self.to;
        let point_b1 = other.from;
        let point_b2 = other.to;

        let a1_a2 = point_a2 - point_a1;
        let b1_b2 = point_b2 - point_b1;
        let b1_a1 = point_a1 - point_b1;

        let denominator = a1_a2.x * b1_b2.y - a1_a2.y * b1_b2.x;
        let numerator1 = b1_a1.y * b1_b2.x - b1_a1.x * b1_b2.y;
        let numerator2 = b1_a1.y * a1_a2.x - b1_a1.x * a1_a2.y;

        let r = numerator1 / denominator;
        let s = numerator2 / denominator;

        (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&s)
    }
}

impl From<Line> for Shape {
    fn from(val: Line) -> Self {
        Self::Line(val)
    }
}
