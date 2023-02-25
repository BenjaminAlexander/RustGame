use std::mem;
use crate::stats::minmax::MinMax::{NoValues, MinAndMax, SingleValue};
use crate::stats::minmax::MinMaxChange::{FirstMinAndMax, NewMax, NewMin};

#[derive(Debug)]
pub enum MinMax<T> {
    NoValues,
    SingleValue(T),
    MinAndMax{
        min: T,
        max: T
    }
}

#[derive(Debug)]
pub enum MinMaxChange<T> {
    FirstMinAndMax(T),
    NewMin(T),
    NewMax(T)
}

impl<T: PartialOrd<T>> MinMax<T> {

    fn take(&mut self) -> Self {
        return mem::replace(self, NoValues);
    }

    pub fn add_value(&mut self, value: T) -> Option<MinMaxChange<&T>> {
        match self.take() {
            NoValues => {
                *self = SingleValue(value);
                return Some(FirstMinAndMax(self.get_min().unwrap()));
            }
            SingleValue(first_value) => {
                if first_value < value {
                    *self = MinAndMax{
                        min: first_value,
                        max: value
                    };

                    return Some(NewMax(self.get_max().unwrap()));
                } else {
                    *self = MinAndMax{
                        min: value,
                        max: first_value
                    };

                    return Some(NewMin(self.get_min().unwrap()));
                }
            }
            MinAndMax { min, max } => {
                if value < min {
                    *self = MinAndMax{
                        min: value,
                        max
                    };

                    return Some(NewMin(self.get_min().unwrap()));
                } else if value > max {
                    *self = MinAndMax{
                        min,
                        max: value
                    };

                    return Some(NewMax(self.get_max().unwrap()));
                } else {
                    return None;
                }
            }
        };
    }

    pub fn get_min(&self) -> Option<&T> {
        match self {
            NoValues => None,
            SingleValue(value) => Some(value),
            MinAndMax { min, .. } => Some(min)
        }
    }

    pub fn get_max(&self) -> Option<&T> {
        match self {
            NoValues => None,
            SingleValue(value) => Some(value),
            MinAndMax { max, .. } => Some(max)
        }
    }
}