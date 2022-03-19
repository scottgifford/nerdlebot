use std::fmt;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

use crate::eq::Equation;
use crate::expr::ExpressionNumber;
use crate::expr::Expression;
use crate::expr::ExpressionPart;
use crate::expr;

const ATTEMPTS: u32 = 1000;

pub fn mknum(x:u32) -> ExpressionNumber {
    ExpressionNumber {
        value: x
    }
}

pub fn mknump(x:u32) -> ExpressionPart {
    ExpressionPart::Number(mknum(x))
}

#[derive(Debug)]
enum Operators {
    Plus,
    Minus,
    Times,
    Divide,
}

impl Distribution<Operators> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operators {
        match rng.gen_range(0..4) {
            0 => Operators::Plus,
            1 => Operators::Minus,
            2 => Operators::Times,
            3 => Operators::Divide,
            _ => panic!("Out-of-range random number chosen!")
        }
    }
}

impl fmt::Display for Operators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Operators::Plus => "+",
            Operators::Minus => "-",
            Operators::Times => "*",
            Operators::Divide => "/",
        })
    }
}

pub struct EqGenNumConstraint<F>
where
    F: Fn(&ExpressionNumber) -> bool,
{
    min: u32,
    max: u32,
    description: String,
    accept: F,
}

impl<F> fmt::Display for EqGenNumConstraint<F>
where
    F: Fn(&ExpressionNumber) -> bool,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EqGenNumConstraint \"{}\": min={} max={}", self.description, self.min, self.max)
    }
}


pub fn find_num_with_constraint<F>(rng: &mut impl Rng, constraint: &EqGenNumConstraint<F>) -> Result<ExpressionNumber, NoMatchFound>
where
    F: Fn(&ExpressionNumber) -> bool,
{
    if constraint.max < constraint.min {
        return Err(NoMatchFound { message: format!("Invalid constraint: {}", constraint)});
    }

    for _try in 1..ATTEMPTS {
        let candidate = rng.gen_range(constraint.min..=constraint.max);
        let candidate = mknum(candidate);
        if !(constraint.accept)(&candidate) {
            // println!("  Rejected {} with constraint {}", candidate, constraint);
            continue;
        }
        return Ok(candidate);
    }
    Err(NoMatchFound { message: format!("No match found for constraint {} after {} tries", constraint, ATTEMPTS)})
}

pub fn eqgen() -> Result<Equation, NoMatchFound> {
    let mut rng = rand::thread_rng();

    let op: Operators = rand::random();

    for _try in 1..ATTEMPTS {
        let mut chars_remaining: i32 = 10 - 1 /* for = */ -1 /* for operator chosen above */;

        let c = rng.gen_range(1..999);
        let c_obj = mknum(c);
        // println!("Searching for solution for op {} and c {}", op, c_obj);
        chars_remaining -= c_obj.len() as i32;

        let chars_reserved = 1; // Leave space for the other number (b)
        let accept = |n: &ExpressionNumber| n.len() as i32 <= (chars_remaining - 1);
        let describer = | | format!("chars < {}", (chars_remaining - chars_reserved));

        // TODO: Use closure instead of repeating function
        let a_obj = match op {
            Operators::Plus => find_num_with_constraint(&mut rng, &EqGenNumConstraint { min: 0, max: c, description: describer(), accept }),
            Operators::Minus => find_num_with_constraint(&mut rng, &EqGenNumConstraint{ min: c, max: 999, description: describer(), accept }),
            Operators::Times => find_num_with_constraint(&mut rng, &EqGenNumConstraint { min: 1, max: c/2, description: describer(), accept: |n| c % n.value == 0 && mknum(c/n.value).len() + n.len() == chars_remaining as usize && accept(n) }),
            Operators::Divide => find_num_with_constraint(&mut rng, &EqGenNumConstraint { min: 1, max: c*c, description: describer(), accept: |n| n.value % c == 0 && accept(n) }),
        };
        let a_obj = match a_obj {
            Ok(a) => a,
            Err(_err) => continue,
        };
        let a = a_obj.value;
        chars_remaining -= a_obj.len() as i32;
        // println!("  Chose a {}, {} chars left", a, chars_remaining);

        let b = match op {
            Operators::Plus => c - a,
            Operators::Minus => a - c,
            Operators::Times => {
                if c % a == 0 {
                    c / a
                } else {
                    // println!("Couldn't find b for {} * b = {}", a, c);
                    continue
                }
            },
            Operators::Divide => {
                if a % c == 0 {
                    a / c
                } else {
                    // println!("Couldn't find b for {} / b = {}", a, c);
                    continue
                }
            }
        };
        let b_obj = mknum(b);
        chars_remaining -= b_obj.len() as i32;
        if chars_remaining != 0 {
            continue;
        }

        let op: Box<dyn expr::ExpressionOperator> = match op {
            Operators::Plus => Box::new(expr::ExpressionOperatorPlus { }),
            Operators::Minus => Box::new(expr::ExpressionOperatorMinus { }),
            Operators::Times => Box::new(expr::ExpressionOperatorTimes { }),
            Operators::Divide => Box::new(expr::ExpressionOperatorDivide { }),
        };
        let op = ExpressionPart::Operator(op);

        let eq = Equation {
            expr: Expression { parts: Vec::from([
                ExpressionPart::Number(a_obj),
                op,
                mknump(b),
            ]) },
            res: c_obj,
        };

        if !eq.computes().unwrap_or(false) {
            println!("Equation unexpectedly did not compute: {}", eq);
            continue;
        }

        return Ok(eq);
    }

    Err(NoMatchFound { message: format!("Failed to generate equation for operator {} after {} attempts", op, ATTEMPTS) })
}

#[derive(Clone)]
pub struct NoMatchFound {
    message: String,
}

impl fmt::Display for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoMatchFound
    : {}", self.message)
    }
}

impl fmt::Debug for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "NoMatchFound
    : {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}
