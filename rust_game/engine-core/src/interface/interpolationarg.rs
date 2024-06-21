use commons::time::TimeDuration;

pub struct InterpolationArg {
    weight: f64,
    duration_since_game_start: TimeDuration,
}

impl InterpolationArg {
    pub fn new(weight: f64, duration_since_game_start: TimeDuration) -> Self {
        return Self {
            weight,
            duration_since_game_start,
        };
    }

    pub fn get_weight(&self) -> f64 {
        return self.weight;
    }

    pub fn get_duration_since_game_start(&self) -> TimeDuration {
        return self.duration_since_game_start;
    }
}
