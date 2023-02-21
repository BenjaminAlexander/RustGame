use std::mem;
use crate::stats::minmax::MinMax::{NoValues, MinAndMax, SingleValue};
use crate::stats::minmax::MinMaxChange::{FirstMinAndMax, NewMax, NewMin, NoChange};

pub enum MinMax<T> {
    NoValues,
    SingleValue(T),
    MinAndMax{
        min: T,
        max: T
    }
}

pub enum MinMaxChange<'a, T> {
    NoChange,
    FirstMinAndMax(&'a T),
    NewMin(&'a T),
    NewMax(&'a T)
}

impl<T: PartialOrd<T>> MinMax<T> {

    fn take(&mut self) -> Self {
        return mem::replace(self, NoValues);
    }

    pub fn add_value(&mut self, value: T) -> MinMaxChange<T> {
        match self.take() {
            NoValues => {
                *self = SingleValue(value);
                return FirstMinAndMax(self.get_min().unwrap());
            }
            SingleValue(first_value) => {
                if first_value < value {
                    *self = MinAndMax{
                        min: first_value,
                        max: value
                    };

                    return NewMax(self.get_max().unwrap());
                } else {
                    *self = MinAndMax{
                        min: value,
                        max: first_value
                    };

                    return NewMin(self.get_min().unwrap());
                }
            }
            MinAndMax { min, max } => {
                if value < min {
                    *self = MinAndMax{
                        min: value,
                        max
                    };

                    return NewMin(self.get_min().unwrap());
                } else if value > max {
                    *self = MinAndMax{
                        min,
                        max: value
                    };

                    return NewMax(self.get_max().unwrap());
                } else {
                    return NoChange;
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