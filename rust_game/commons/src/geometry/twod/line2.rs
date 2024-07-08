use crate::geometry::twod::{
    Intersection,
    Vector2,
};

pub enum Line2 {
    NonVertical { m: f64, b: f64 },
    Vertical { x: f64 },
}

impl Line2 {
    pub fn new(start: Vector2, end: Vector2) -> Self {
        if start.get_x() == end.get_x() {
            return Line2::Vertical { x: start.get_x() };
        } else {
            let m = (end.get_y() - start.get_y()) / (end.get_x() - start.get_x());
            let b = start.get_y() - m * start.get_x();
            return Line2::NonVertical { m, b };
        }
    }

    pub fn get_y(&self, x: f64) -> Intersection {
        return match self {
            Line2::NonVertical { m, b } => Intersection::Point(m * x + b),
            Line2::Vertical { x: vertical_x } => {
                if *vertical_x == x {
                    Intersection::All
                } else {
                    Intersection::None
                }
            }
        };
    }

    pub fn get_intersection(&self, other: &Self) -> Intersection {
        return match self {
            Line2::NonVertical { m: m1, b: b1 } => match other {
                Line2::NonVertical { m: m2, b: b2 } => get_intersection(*m1, *b1, *m2, *b2),
                Line2::Vertical { x } => Intersection::Point(*x),
            },
            Line2::Vertical { x: x1 } => match other {
                Line2::NonVertical { .. } => Intersection::Point(*x1),
                Line2::Vertical { x: x2 } => {
                    if x1 == x2 {
                        Intersection::All
                    } else {
                        Intersection::None
                    }
                }
            },
        };
    }
}

fn get_intersection(m1: f64, b1: f64, m2: f64, b2: f64) -> Intersection {
    if m1 == m2 {
        if b1 == b2 {
            return Intersection::All;
        } else {
            return Intersection::None;
        }
    } else {
        let x = (b2 - b1) / (m1 - m2);
        return Intersection::Point(x);
    }
}
