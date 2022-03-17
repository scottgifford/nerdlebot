use std::fmt;
use rand::Rng;

use crate::eq::Equation;
use crate::expr::ExpressionNumber;
use crate::expr::Expression;
use crate::expr::ExpressionPart;


pub fn eqgen() -> Equation {
    let mut rng = rand::thread_rng();
    let n = ExpressionNumber { value: rng.gen_range(0..999) };
    Equation {
        expr: Expression { parts: Vec::from([ExpressionPart::Number(n.clone())]) },
        res: n,
    }
}
