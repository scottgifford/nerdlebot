use rand::Rng;

use crate::eq::Equation;
use crate::expr::{Expression, ExpressionNumber, ExpressionPart, ExpressionOperator, ExpressionOperatorPlus, ExpressionOperatorMinus, ExpressionOperatorTimes, ExpressionOperatorDivide, ExpressionOperatorEnum, mknum, mknump};

use crate::constraint::{find_num_with_constraint, EquationConstraint, ExpressionNumberConstraint, NoMatchFound};

const ATTEMPTS: u32 = 10000;

pub fn eqgen_constrained<F>(constraint: &EquationConstraint<F>) -> Result<Equation, NoMatchFound>
where
    F: Fn(&Equation) -> bool,
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

    for _try in 1..ATTEMPTS {
        let mut chars_remaining: i32 = 10 - 1 /* for = */ -1 /* for operator chosen above */;

        let c = rng.gen_range(match op {
            ExpressionOperatorEnum::Times => 1024..=9801, // 32*32 to 99*99, other values won't have 10 digits
            _ => 1..=999
        });
        let c_obj = mknum(c);
        // println!("Searching for solution for op {} and c {}", op, c_obj);
        chars_remaining -= c_obj.len() as i32;

        let chars_reserved = 1; // Leave space for the other number (b)
        let accept = |n: &ExpressionNumber| n.len() as i32 <= (chars_remaining - 1);
        let describer = | | format!("chars < {}", (chars_remaining - chars_reserved));

        let a_obj = match op {
            ExpressionOperatorEnum::Plus => find_num_with_constraint(&mut rng, &ExpressionNumberConstraint { range: 0..=c, description: describer(), accept }),
            ExpressionOperatorEnum::Minus => find_num_with_constraint(&mut rng, &ExpressionNumberConstraint{ range: c..=999, description: describer(), accept }),
            ExpressionOperatorEnum::Times => find_num_with_constraint(&mut rng, &ExpressionNumberConstraint { range: 1..=c/2, description: describer(), accept: |n| c % n.value == 0 && mknum(c/n.value).len() + n.len() == chars_remaining as usize && accept(n) }),
            ExpressionOperatorEnum::Divide => find_num_with_constraint(&mut rng, &ExpressionNumberConstraint { range: 1..=c*c, description: describer(), accept: |n| n.value % c == 0 && accept(n) }),
        };
        let a_obj = match a_obj {
            Ok(a) => a,
            Err(_err) => continue,
        };
        let a = a_obj.value;
        chars_remaining -= a_obj.len() as i32;
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
        chars_remaining -= b_obj.len() as i32;
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
    eqgen_constrained(&EquationConstraint::new(|_| true))
}