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

//

impl Shape {
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
    pub fn distance(&self, p: Vec2) -> f32 {
        let a = self.from;
        let b = self.to;
        let a_to_p = p - a;
        let a_to_b = b - a;

        let t = (a_to_p.dot(a_to_b) / a_to_b.length_squared())
            .min(1.0)
            .max(0.0);

        ((a + a_to_b * t) - p).length()
    }
}

impl From<Line> for Shape {
    fn from(val: Line) -> Self {
        Self::Line(val)
    }
}
