use commons::time::TimeValue;
use std::cmp::Ordering;

pub struct Event {
    time: TimeValue,
    id: usize,
    function: Box<dyn FnOnce() + Send>,
}

impl Event {
    pub fn new(id: usize, time: TimeValue, function: impl FnOnce() + Send + 'static) -> Self {
        return Self {
            time,
            id,
            function: Box::new(function),
        };
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn get_time(&self) -> &TimeValue {
        return &self.time;
    }

    pub fn run(self) {
        (self.function)();
    }
}

impl Eq for Event {}

impl PartialEq<Self> for Event {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}

impl PartialOrd<Self> for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let time_ord = self.time.partial_cmp(&other.time);

        if let Some(Ordering::Equal) = time_ord {
            return self.id.partial_cmp(&other.id);
        } else {
            return time_ord;
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        let time_ord = self.time.cmp(&other.time);

        if Ordering::Equal == time_ord {
            return self.id.cmp(&other.id);
        } else {
            return time_ord;
        }
    }
}
