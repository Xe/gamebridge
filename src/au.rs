#[derive(Copy, Clone)]
pub(crate) struct Lerper {
    extended_tick: u64,
    lerp_time: f64,
    goal: i64,
    pub(crate) scalar: i64,
    max: i64,
    min: i64,
}

impl Lerper {
    pub(crate) fn init(lerp_time: f64, max: i64, min: i64, goal: i64) -> Lerper {
        Lerper {
            extended_tick: 0,
            lerp_time: lerp_time,
            goal: goal,
            scalar: 0, // I hope to GOD that 0 is the resting point
            max: max,
            min: min,
        }
    }

    pub(crate) fn add(&mut self, new_scalar: i64) {
        self.scalar += new_scalar;
    }

    pub(crate) fn update(&mut self, new_scalar: i64) {
        self.scalar = new_scalar;
    }

    pub(crate) fn apply(&mut self, now: u64) -> i64 {
        let scalar = self.scalar;
        self.scalar = match scalar {
            _ if scalar == self.goal => self.goal,
            _ if scalar >= self.max => {
                self.extended_tick = now;
                scalar - 1
            }
            _ if scalar <= self.min => {
                self.extended_tick = now;
                scalar + 1
            }
            _ => {
                let t = (now - self.extended_tick) as f64 / self.lerp_time;
                lerp(self.scalar, 0, t)
            }
        };

        if self.scalar >= self.max {
            return self.max;
        }

        if self.scalar <= self.min {
            return self.min;
        }

        self.scalar
    }

    pub(crate) fn pressed(&mut self, threshold: i64) -> bool {
        if self.scalar <= threshold {
            self.scalar = 0;
        }

        self.scalar >= threshold
    }
}

fn lerp(start: i64, end: i64, t: f64) -> i64 {
    (start as f64 * (1.0 - t) + (end as f64) * t) as i64
}

#[cfg(test)]
mod test {
    #[test]
    fn lerp_scale() {
        for case in [(0.1, 10), (0.5, 31)].iter() {
            let t = case.0;
            let start = 127.0 * t;
            assert_eq!(super::lerp(start as i64, 0, t), case.1);
        }
    }

    #[test]
    fn lerper() {
        use super::Lerper;
        let mut lerper = Lerper::init(15.0, 127, -128, 0);

        for case in [(127, 3, 126), (100, 8, 66), (-124, 8, -82)].iter() {
            let scalar = case.0;
            let now = case.1;
            let want = case.2;

            lerper.update(scalar);
            let result = lerper.apply(now);
            assert_eq!(result, want);
        }
    }
}
