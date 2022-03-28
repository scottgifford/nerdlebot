use std::rc::Rc;
use rand::Rng;

use crate::eq::Equation;
use crate::nerdle::{NERDLE_CHARACTERS, NERDLE_A_MAX, NERDLE_C_MUL_MIN, NERDLE_C_MUL_MAX, NERDLE_C_OTHER_MIN, NERDLE_C_OTHER_MAX};
use crate::expr::{Expression, ExpressionNumber, ExpressionPart, ExpressionOperator, ExpressionOperatorPlus, ExpressionOperatorMinus, ExpressionOperatorTimes, ExpressionOperatorDivide, ExpressionOperatorEnum, mknum, mknump};
use crate::constraint::{find_num_with_constraint, EquationConstraint, ExpressionNumberConstraint, NoMatchFound};

const ATTEMPTS: u32 = 10000;

pub fn eqgen_constrained(constraint: &EquationConstraint) -> Result<Equation, NoMatchFound>
{
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
        ExpressionOperatorEnum::Times => NERDLE_C_MUL_MIN..=NERDLE_C_MUL_MAX,
        _ => NERDLE_C_OTHER_MIN..=NERDLE_C_OTHER_MAX
    };
    let description = format!("range {}..={}", c_init_range.start(), c_init_range.end());
    let c_init_constraint = ExpressionNumberConstraint {
        range: c_init_range,
        description,
        ..Default::default()
    };
    let c_constraint = ExpressionNumberConstraint::intersect(&c_init_constraint, &constraint.c_constraint);

    // TODO: Make this a method
    if c_constraint.range.is_empty() {
        // Give up immediately, our constraints make this impossible
        // println!("Impossible constraints on c_constraint, giving up");
        return Err(NoMatchFound { message: format!("No valid values for c, with operator {}, constraint {}", &op, c_constraint) })
    }

    for _try in 1..ATTEMPTS {
        let chars_remaining: i32 = NERDLE_CHARACTERS as i32 - 1 /* for = */ -1 /* for operator chosen above */;

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
                range: c..=NERDLE_A_MAX,
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
        let a_constraint = ExpressionNumberConstraint::intersect(&a_init_constraint, &constraint.a_constraint);
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

// TODO: CopyPasta from main.rs
macro_rules! skip_fail {
    ($res:expr, $message:expr) => {
        match $res {
            Ok(val) => val,
            Err(_e) => {
                // println!("{} (Error {})", $message, e);
                continue;
            }
        }
    };
}

pub fn eqgen_3_operands_constrained(constraint: &EquationConstraint) -> Result<Equation, NoMatchFound> {
    let mut rng = rand::thread_rng();

    for _try in 1..ATTEMPTS {
        let mut parts: Vec<ExpressionPart> = Vec::new();
        let extra_ops = if constraint.max_ops > 1 {
            rng.gen_range(0..constraint.max_ops)
        } else {
            0
        };
        println!("Trying with {} extra operators", extra_ops);
        let operand_range = if extra_ops < 1 {
            1..=99 // TODO: Is this right?  Maybe not for all operators?
        } else {
            1..=9
        };

        let op1 = gen_operator_constrained(constraint);
        let a_base_range = match op1 {
            ExpressionOperatorEnum::Divide if extra_ops < 1 => 100..=999,
            _ => operand_range.clone()
        };
        let a_base_constraint = ExpressionNumberConstraint {
            description: format!("{}..={}", a_base_range.start(), a_base_range.end()),
            range: a_base_range,
            ..Default::default()
        };
        let a = find_num_with_constraint(&mut rng, &ExpressionNumberConstraint::intersect(&a_base_constraint, &constraint.a_constraint))?;

        let mut b_base_constraint = ExpressionNumberConstraint {
            range: operand_range.clone(),
            description: format!("{}..={}", operand_range.start(), operand_range.end()),
            ..Default::default()
        };
        match op1 {
            ExpressionOperatorEnum::Divide => {
                // TODO: Naming is a real mess
                // TODO: Should we always validate this way?
                let op1_2 = op2op(&op1);
                let a = a.clone();
                match op1_2 {
                    ExpressionPart::Operator(op1_3) => {
                        b_base_constraint.range = 1..=9;
                        b_base_constraint.accept = Rc::new(move |b| op1_3.operate(&a, &b).is_ok() );
                    },
                    _ => continue // Should never happen
                }
            },
            _ => { },
        };
        let b = find_num_with_constraint(&mut rng, &ExpressionNumberConstraint::intersect(&b_base_constraint, &constraint.b_constraint))?;

        parts.push(ExpressionPart::Number(a));
        parts.push(op2op(&op1));
        parts.push(ExpressionPart::Number(b));

        for _i in 0..extra_ops {
            let op2 = gen_operator_constrained(constraint);
            parts.push(op2op(&op2));

            let b2_base_constraint = ExpressionNumberConstraint {
                range: 1..=9,
                description: format!("1..=9"),
                ..Default::default()
            };
            let b2 = find_num_with_constraint(&mut rng, &ExpressionNumberConstraint::intersect(&b2_base_constraint, &constraint.b2_constraint))?;
            parts.push(ExpressionPart::Number(b2));
        }

        let expr = Expression { parts };
        let res = skip_fail!(expr.calculate(), format!("Error calculating expression {}", expr));
        let eq = Equation { expr, res };
        if eq.len() != NERDLE_CHARACTERS as usize {
            println!("Equation '{}' is wrong length ({} chars != {})", eq, eq.len(), NERDLE_CHARACTERS);
            continue;
        }
        return Ok(eq);
    }

    Err(NoMatchFound { message: format!("Failed to generate equation after {} attempts", ATTEMPTS) })
}

fn op2op (op: &ExpressionOperatorEnum) -> ExpressionPart {
    let op: Box<dyn ExpressionOperator> = match &op {
        ExpressionOperatorEnum::Plus => Box::new(ExpressionOperatorPlus { }),
        ExpressionOperatorEnum::Minus => Box::new(ExpressionOperatorMinus { }),
        ExpressionOperatorEnum::Times => Box::new(ExpressionOperatorTimes { }),
        ExpressionOperatorEnum::Divide => Box::new(ExpressionOperatorDivide { }),
    };
    ExpressionPart::Operator(op)
}

pub fn eqgen() -> Result<Equation, NoMatchFound> {
    eqgen_constrained(&EquationConstraint::default())
}

pub fn eqgen_3_operands() -> Result<Equation, NoMatchFound> {
    eqgen_3_operands_constrained(&EquationConstraint {
        max_ops: 2,
        ..Default::default()
    })
}


pub fn gen_operator_constrained(constraint: &EquationConstraint) -> ExpressionOperatorEnum {
    loop {
        let tmp_op: ExpressionOperatorEnum = rand::random();
        let tmp_op_ch = tmp_op.to_string().as_bytes()[0];
        if !*constraint.operator.get(&tmp_op_ch).unwrap_or(&true) {
            // println!("Rejected operator '{}' because it's been ruled out", tmp_op_ch as char);
            continue;
        }
        // println!("Accepted operator '{}'", tmp_op_ch as char);
        break tmp_op;
    }
}