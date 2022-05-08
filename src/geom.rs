use crate::math::{Line, Shape};
use glam::Vec2;
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
    pub fn is_inside(&self, point: Vec2) -> bool {
        let half_offs = Vec2::new(0.5, 0.5);
        let tester_a = half_offs + point.round();
        let tester_b = half_offs + Vec2::new(self.min_x - 100.0, point.y).round();
        self.iter_lines()
            .filter(|&Line { from, to }| {
                Self::line_line_intersection(from.round(), to.round(), tester_a, tester_b)
            })
            .count()
            % 2
            == 1
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        self.shapes
            .iter()
            .copied()
            .flat_map(|shape| shape.iter_lines())
    }

    // shamelessly stolen from: https://gamedev.stackexchange.com/a/26022
    fn line_line_intersection(
        point_a1: Vec2,
        point_a2: Vec2,
        point_b1: Vec2,
        point_b2: Vec2,
    ) -> bool {
        let denominator = ((point_a2.x - point_a1.x) * (point_b2.y - point_b1.y))
            - ((point_a2.y - point_a1.y) * (point_b2.x - point_b1.x));
        let numerator1 = ((point_a1.y - point_b1.y) * (point_b2.x - point_b1.x))
            - ((point_a1.x - point_b1.x) * (point_b2.y - point_b1.y));
        let numerator2 = ((point_a1.y - point_b1.y) * (point_a2.x - point_a1.x))
            - ((point_a1.x - point_b1.x) * (point_a2.y - point_a1.y));

        // Detect coincident lines (has a problem, read below)
        /* if denominator.abs() <= f32::EPSILON {
            return numerator1.abs() <= f32::EPSILON && numerator2.abs() <= f32::EPSILON;
        } */

        let r = numerator1 / denominator;
        let s = numerator2 / denominator;

        (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&s)
    }
}

impl OutlineBuilder for Geometry {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = Vec2::new(x, y);
        self.current = to;
        self.min_x = self.min_x.min(to.x);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = Vec2::new(x, y);
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
        let by = Vec2::new(x1, y1);
        let to = Vec2::new(x, y);
        self.shapes.push(Shape::Quad {
            from: self.current,
            by,
            to,
        });
        self.current = to;
        self.min_x = self.min_x.min(by.x).min(to.x);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let by_a = Vec2::new(x1, y1);
        let by_b = Vec2::new(x2, y2);
        let to = Vec2::new(x, y);
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
