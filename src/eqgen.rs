use std::rc::Rc;

use crate::eq::Equation;
use crate::nerdle::{NERDLE_CHARACTERS, NERDLE_A_MAX};
use crate::expr::{Expression, ExpressionPart, ExpressionOperator, ExpressionOperatorPlus, ExpressionOperatorMinus, ExpressionOperatorTimes, ExpressionOperatorDivide, ExpressionOperatorEnum};
use crate::constraint::{find_num_with_constraint, EquationConstraint, ExpressionNumberConstraint, NoMatchFound};
use crate::util::range_rand_or_only;

const ATTEMPTS: u32 = 10000;

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

pub fn eqgen_constrained(constraint: &EquationConstraint) -> Result<Equation, NoMatchFound> {
    let mut rng = rand::thread_rng();

    // println!("Incoming constraint: {}", constraint);

    for _try in 1..ATTEMPTS {
        let mut parts: Vec<ExpressionPart> = Vec::new();
        let num_ops = range_rand_or_only(constraint.num_ops.clone())?;

        let extra_ops = if num_ops > 1 {
            1
        } else {
            0
        };
        // println!("Trying with {} extra operators", extra_ops);
        let operand_range = if extra_ops < 1 {
            1..=NERDLE_A_MAX
        } else {
            1..=9
        };

        let op1 = gen_operator_constrained(constraint);
        let a_base_range = match op1 {
            ExpressionOperatorEnum::Divide if extra_ops < 1 => 100..=999,
            _ => operand_range.clone()
        };
        let a_base_constraint = ExpressionNumberConstraint {
            description: format!("{}..={} (from operator {}, extra_ops {})", a_base_range.start(), a_base_range.end(), op1, extra_ops),
            range: a_base_range,
            ..Default::default()
        };
        let a_constraint = &ExpressionNumberConstraint::intersect(&a_base_constraint, &constraint.a_constraint);
        let a = skip_fail!(find_num_with_constraint(&mut rng, a_constraint), "Failed to generate a");

        let mut b_base_constraint = ExpressionNumberConstraint {
            range: operand_range.clone(),
            description: format!("{}..={} (from operator {}, extra_ops {})", operand_range.start(), operand_range.end(), op1, extra_ops),
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
        let b_constraint = &ExpressionNumberConstraint::intersect(&b_base_constraint, &constraint.b_constraint);
        let b = skip_fail!(find_num_with_constraint(&mut rng, &b_constraint), "Failed to generate b");

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
            // println!("Equation '{}' is wrong length ({} chars != {})", eq, eq.len(), NERDLE_CHARACTERS);
            continue;
        }
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