use std::fmt;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

use crate::eq::Equation;
use crate::expr::ExpressionNumber;
use crate::expr::Expression;
use crate::expr::ExpressionPart;
use crate::expr;

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

pub fn eqgen() -> Result<Equation, NoMatchFound> {
    let mut rng = rand::thread_rng();

    let op: Operators = rand::random();

    for _try in 1..1000 {

        let c = rng.gen_range(0..999);
        let a = match op {
            Operators::Plus => rng.gen_range(0..=c),
            Operators::Minus => rng.gen_range(c..=999),
            Operators::Times => rng.gen_range(1..=(c as f64).sqrt() as u32),
            Operators::Divide => rng.gen_range(1..=c*c),
        };
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
        let op: Box<dyn expr::ExpressionOperator> = match op {
            Operators::Plus => Box::new(expr::ExpressionOperatorPlus { }),
            Operators::Minus => Box::new(expr::ExpressionOperatorMinus { }),
            Operators::Times => Box::new(expr::ExpressionOperatorTimes { }),
            Operators::Divide => Box::new(expr::ExpressionOperatorDivide { }),
        };
        let op = ExpressionPart::Operator(op);

        let eq = Equation {
            expr: Expression { parts: Vec::from([
                mknump(a),
                op,
                mknump(b),
            ]) },
            res: mknum(c),
        };

        if !eq.computes().unwrap_or(false) {
            println!("Equation unexpectedly did not compute: {}", eq);
            continue;
        }

        return Ok(eq);
    }

    return Err(NoMatchFound { message: format!("Failed to generate equation after 100 attempts") })
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
