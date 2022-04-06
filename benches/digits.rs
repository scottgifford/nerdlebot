use criterion::{criterion_group, criterion_main, Criterion};

fn bench_wrap(cb: &dyn Fn(i32) -> i32) {
    for i in 0..=9 {
        // let r = cb(i);
        // println!("{} has {} digits", i, r);
        assert_eq!(1, cb(i));
    }
    for i in 10..=99 {
        assert_eq!(2, cb(i));
    }
    for i in 100..=999 {
        assert_eq!(3, cb(i));
    }
    for i in 1000..=9999 {
        assert_eq!(4, cb(i));
    }
    for i in 10_000..=99_999 {
        assert_eq!(5, cb(i));
    }
    for i in 100_000..=999_999 {
        assert_eq!(6, cb(i));
    }
}
fn stringify(i: i32) -> i32 {
    i.to_string().len() as i32
}

fn log10(i: i32) -> i32 {
    if i == 0 {
        1
    } else {
        (i as f32).log10().floor() as i32 + 1
    }
}

// Repeated Division
fn repeated_div(i: i32) -> i32 {
    let mut j = i;
    let mut digits = 1;
    while j >= 10 {
        j /= 10;
        digits += 1;
    }
    digits
}


// Clever stuff from the Web
// From https://stackoverflow.com/questions/1489830/efficient-way-to-determine-number-of-digits-in-an-integer
fn int_log2(x: i32) -> i32 {
    let mut result = -1;
    let mut x = x;
    while x != 0 {
        x >>= 1;
        result += 1;
    }
    result
}

fn int_log10(x: i32) -> i32 {
    let powers_of_10 = [1,         10,        100,     1000,
                               10000,     100000,    1000000, 10000000,
                               100000000, 1000000000 ];
    let aprox = (int_log2(x) + 1) * 1233 >> 12;
    aprox - (if x < powers_of_10[aprox as usize] { 1 } else { 0 })
}

fn clever_digits(x: i32) -> i32 {
    if x == 0 {
        1
    } else {
        int_log10(x) + 1
    }
}


// From https://stackoverflow.com/questions/1489830/efficient-way-to-determine-number-of-digits-in-an-integer
fn lookup_digits(x: i32) -> i32 {
    // TODO: What about int_min?
    // if (x == INT32_MIN) return 10 + 1;
    if x < 0 {
        return lookup_digits(-x) + 1;
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stringify", |b| b.iter(|| bench_wrap(&stringify)));
    c.bench_function("log10", |b| b.iter(|| bench_wrap(&log10)));
    c.bench_function("repeated integer division", |b| b.iter(|| bench_wrap(&repeated_div)));
    c.bench_function("clever web version", |b| b.iter(|| bench_wrap(&clever_digits)));
    c.bench_function("lookup", |b| b.iter(|| bench_wrap(&lookup_digits)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
