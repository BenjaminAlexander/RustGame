use commons::geometry::twod::Vector2;
use serde::{Deserialize, Serialize};
use crate::simplegame::line2::{Line2, Intersection};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LineSegment2 {
    start: Vector2,
    end: Vector2,
}

impl LineSegment2 {

    pub fn new(start: Vector2, end: Vector2) -> Self {
        if start.get_x() <= end.get_x() {
            return Self {
                start,
                end
            };
        } else {
            return Self {
                end,
                start
            };
        }
    }

    pub fn get_line2(&self) -> Line2 {
        return Line2::new(self.start, self.end);
    }

    pub fn get_y(&self, x: f64) -> Intersection {
        if self.start.get_x() <= x &&
            x <= self.end.get_x() {

            return self.get_line2().get_y(x);
        } else {
            return Intersection::None;
        }
    }

    pub fn get_intersection(&self, other: Self) -> Intersection {
        let self_line2 = self.get_line2();
        let other_line2 = other.get_line2();

        return match self_line2.get_intersection(&other_line2) {
            Intersection::All => Intersection::All,
            Intersection::None => Intersection::None,
            Intersection::Point (x) =>
                match self.get_y(x) {
                    Intersection::All => Intersection::Point(x),
                    Intersection::None => Intersection::None,
                    Intersection::Point(_y) => Intersection::Point(x),
                }
        }
    }
}