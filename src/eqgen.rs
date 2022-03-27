use std::rc::Rc;

use crate::eq::Equation;
use crate::expr::{Expression, ExpressionNumber, ExpressionPart, ExpressionOperator, ExpressionOperatorPlus, ExpressionOperatorMinus, ExpressionOperatorTimes, ExpressionOperatorDivide, ExpressionOperatorEnum, mknum, mknump};

use crate::constraint::{find_num_with_constraint, EquationConstraint, ExpressionNumberConstraint, NoMatchFound};

const ATTEMPTS: u32 = 10000;

pub fn eqgen_constrained(constraint: &EquationConstraint) -> Result<Equation, NoMatchFound>
{
    // TODO: Is this efficient?  Should this be a global or something?
    let mut rng = rand::thread_rng();

    let op = loop {
        let tmp_op: ExpressionOperatorEnum = rand::random();
        let tmp_op_ch = tmp_op.to_string().as_bytes()[0];
        if !*constraint.operator.get(&tmp_op_ch).unwrap_or(&true) {
            // println!("Rejected operator '{}' because it's been ruled out", tmp_op_ch as char);
            continue;
        }
        // println!("Accepted operator '{}'", tmp_op_ch as char);
        break tmp_op;
    };

    let c_init_range = match op {
        ExpressionOperatorEnum::Times => 1024..=9801, // 32*32 to 99*99, other values won't have 10 digits
        _ => 1..=999
    };
    let description = format!("range {}..={}", c_init_range.start(), c_init_range.end());
    let c_init_constraint = ExpressionNumberConstraint {
        range: c_init_range,
        description,
        ..Default::default()
    };
    let c_constraint = ExpressionNumberConstraint::intersect(&c_init_constraint, &constraint.c_range);

    // TODO: Make this a method
    if c_constraint.range.is_empty() {
        // Give up immediately, our constraints make this impossible
        // println!("Impossible constraints on c_range, giving up");
        return Err(NoMatchFound { message: format!("No valid values for c, with operator {}, constraint {}", &op, c_constraint) })
    }

    for _try in 1..ATTEMPTS {
        let chars_remaining: i32 = 10 - 1 /* for = */ -1 /* for operator chosen above */;

        // TODO: Make this a method?
        let c = find_num_with_constraint(&mut rng, &c_constraint)?;
        let c = c.value;
        let c_obj = mknum(c);
        // println!("Searching for solution for op {} and c {}", op, c_obj);
        let chars_remaining = chars_remaining - c_obj.len() as i32;

        let chars_reserved = 1; // Leave space for the other number (b)
        let accept = move |n: &ExpressionNumber| n.len() as i32 <= (chars_remaining - 1);
        let describer = | | format!("chars < {}", (chars_remaining - chars_reserved));

        let a_init_constraint = match op {
            ExpressionOperatorEnum::Plus => ExpressionNumberConstraint { 
                range: 0..=c,
                description: describer(),
                accept: Rc::new(accept),
            },
            ExpressionOperatorEnum::Minus => ExpressionNumberConstraint{
                range: c..=999,
                description: describer(),
                accept: Rc::new(accept)
            },
            ExpressionOperatorEnum::Times => ExpressionNumberConstraint {
                range: 1..=c/2,
                description: describer(),
                accept: Rc::new(move |n| c % n.value == 0 && mknum(c/n.value).len() + n.len() == chars_remaining as usize && accept(n)),
            },
            ExpressionOperatorEnum::Divide => ExpressionNumberConstraint {
                range: 1..=c*c,
                description: describer(),
                accept: Rc::new(move |n| n.value % c == 0 && accept(n)),
            },
        };
        let a_constraint = ExpressionNumberConstraint::intersect(&a_init_constraint, &constraint.a_range);
        let a_obj = find_num_with_constraint(&mut rng, &a_constraint);
        let a_obj = match a_obj {
            Ok(a) => a,
            Err(_err) => continue,
        };
        let a = a_obj.value;
        let chars_remaining = chars_remaining - a_obj.len() as i32;
        // println!("  Chose a {}, {} chars left", a, chars_remaining);

        let b = match op {
            ExpressionOperatorEnum::Plus => c - a,
            ExpressionOperatorEnum::Minus => a - c,
            ExpressionOperatorEnum::Times => {
                if c % a == 0 {
                    c / a
                } else {
                    // println!("Couldn't find b for {} * b = {}", a, c);
                    continue
                }
            },
            ExpressionOperatorEnum::Divide => {
                if a % c == 0 {
                    a / c
                } else {
                    // println!("Couldn't find b for {} / b = {}", a, c);
                    continue
                }
            }
        };
        let b_obj = mknum(b);
        let chars_remaining = chars_remaining - b_obj.len() as i32;
        if chars_remaining != 0 {
            continue;
        }

        let op: Box<dyn ExpressionOperator> = match op {
            ExpressionOperatorEnum::Plus => Box::new(ExpressionOperatorPlus { }),
            ExpressionOperatorEnum::Minus => Box::new(ExpressionOperatorMinus { }),
            ExpressionOperatorEnum::Times => Box::new(ExpressionOperatorTimes { }),
            ExpressionOperatorEnum::Divide => Box::new(ExpressionOperatorDivide { }),
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

        if !(constraint.accept)(&eq) {
            // println!("Equation did not match constraint: {}", eq);
            continue;
        }

        return Ok(eq);
    }

    Err(NoMatchFound { message: format!("Failed to generate equation for operator {} after {} attempts", op, ATTEMPTS) })
}

pub fn eqgen() -> Result<Equation, NoMatchFound> {
    eqgen_constrained(&EquationConstraint::default())
}
