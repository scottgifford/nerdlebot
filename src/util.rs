use std::ops::RangeInclusive;
use rand::Rng;
use rand::distributions::uniform::SampleUniform;

use crate::constraint::NoMatchFound;

pub fn range_rand_or_only<T>(range: RangeInclusive<T>) -> Result<T, NoMatchFound>
where
    T: PartialOrd,
    T: std::fmt::Display,
    T: SampleUniform,
    T: Copy,
{
    if range.is_empty() {
        if range.start() == range.end() {
            Ok(*(range.start()))
        } else {
            Err(NoMatchFound { message: format!("Could not choose item from range {}..={}", range.start(), range.end())})
        }
    } else {
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(range))
    }
}

#[test]
fn range_rand_or_only_test() {
    assert_eq!(3, range_rand_or_only::<u32>(3..=3).unwrap());
    for _i in 0..100 {
        let x = range_rand_or_only::<u32>(1..=10).unwrap();
        assert!((1..=10).contains(&x));
    }
    assert!(!range_rand_or_only::<u32>(3..=2).is_ok());
}
