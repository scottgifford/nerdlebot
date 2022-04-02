use std::ops::RangeInclusive;
use rand::Rng;

use crate::constraint::NoMatchFound;

pub fn range_rand_or_only(range: &RangeInclusive<u32>) -> Result<u32, NoMatchFound> {
    if range.is_empty() {
        if (range.start() == range.end()) {
            Ok(*range.start())
        } else {
            Err(NoMatchFound { message: format!("Could not choose item from range {}..={}", range.start(), range.end())})
        }
    } else {
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(*range))
    }
}
