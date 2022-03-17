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
    // Times,
    // Divide,
}

pub fn eqgen() -> Equation {
    let ALL_OPS = [
        Operators::Plus,
        Operators::Minus,
    ];

    let mut rng = rand::thread_rng();

    let op = &ALL_OPS[rng.gen_range(0..ALL_OPS.len())];

    let c = rng.gen_range(0..999);
    let a = match op {
        Operators::Plus => rng.gen_range(0..c),
        Operators::Minus => rng.gen_range(c..1000),
    };
    let b = match op {
        Operators::Plus => c - a,
        Operators::Minus => a - c,
    };
    let op: Box<dyn expr::ExpressionOperator> = match op {
        Operators::Plus => Box::new(expr::ExpressionOperatorPlus { }),
        Operators::Minus => Box::new(expr::ExpressionOperatorMinus { }),
    };
    let op = ExpressionPart::Operator(op);

    Equation {
        expr: Expression { parts: Vec::from([
            mknump(a),
            op,
            mknump(b),
        ]) },
        res: mknum(c),
    }
}
