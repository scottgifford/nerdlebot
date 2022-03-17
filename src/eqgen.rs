use std::fmt;
use rand::Rng;

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

enum Operators {
    Plus,
    Minus,
    Times,
    // Divide,
}

pub fn eqgen() -> Result<Equation, NoMatchFound> {
    let ALL_OPS = [
        Operators::Plus,
        Operators::Minus,
        Operators::Times,
    ];

    let mut rng = rand::thread_rng();

    // TODO: Use constant
    for _try in 1..100 {
        let op = &ALL_OPS[rng.gen_range(0..ALL_OPS.len())];

        let c = rng.gen_range(0..999);
        let a = match op {
            Operators::Plus => rng.gen_range(0..c),
            Operators::Minus => rng.gen_range(c..1000),
            Operators::Times => rng.gen_range(1..(c as f64).sqrt() as u32),
        };
        let b = match op {
            Operators::Plus => c - a,
            Operators::Minus => a - c,
            Operators::Times => {
                if c % a == 0 {
                    c / a
                } else {
                    println!("Couldn't find b for {} * b = {}", a, c);
                    continue
                }
            }
        };
        let op: Box<dyn expr::ExpressionOperator> = match op {
            Operators::Plus => Box::new(expr::ExpressionOperatorPlus { }),
            Operators::Minus => Box::new(expr::ExpressionOperatorMinus { }),
            Operators::Times => Box::new(expr::ExpressionOperatorTimes { }),

        };
        let op = ExpressionPart::Operator(op);

        // TODO: Verify it computes, log an error if not
        return Ok(Equation {
            expr: Expression { parts: Vec::from([
                mknump(a),
                op,
                mknump(b),
            ]) },
            res: mknum(c),
        })
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
