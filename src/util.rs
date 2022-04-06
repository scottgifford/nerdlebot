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

// From https://stackoverflow.com/questions/1489830/efficient-way-to-determine-number-of-digits-in-an-integer
pub fn num_digits(x: i32) -> u32 {
    // TODO: What about int_min?
    // if (x == INT32_MIN) return 10 + 1;
    if x < 0 {
        return num_digits(-x) + 1;
    }

    if x >= 10000 {
        if x >= 10000000 {
            if x >= 100000000 {
                if x >= 1000000000 {
                    return 10;
                }
                return 9;
            }
            return 8;
        }
        if x >= 100000 {
            if x >= 1000000 {
                return 7;
            }
            return 6;
        }
        return 5;
    }
    if x >= 100 {
        if x >= 1000 {
            return 4;
        }
        return 3;
    }
    if x >= 10 {
        return 2;
    }
    return 1;
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

#[test]
fn num_digits_test() {
    for i in 0..=9 {
        assert_eq!(1, num_digits(i));
    }
    for i in 10..=99 {
        assert_eq!(2, num_digits(i));
    }
    for i in 100..=999 {
        assert_eq!(3, num_digits(i));
    }
    for i in 1000..=9999 {
        assert_eq!(4, num_digits(i));
    }
}
