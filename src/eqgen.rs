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

        let(op1, op2_opt) = gen_ops(constraint)?;
        let op2_str = match &op2_opt {
            Some(op) => op.to_string(),
            None => "None".to_string()
        };

        // println!("Trying with op1 {}, op2 {}", &op1, &op2_opt.map(|op| op.to_string()).unwrap_or("None"));
        let operand_range = if op2_opt.is_none() {
            1..=NERDLE_A_MAX
        } else {
            1..=99
        };

        let a_base_range = match op1 {
            ExpressionOperatorEnum::Divide if op2_opt.is_none() => 100..=999,
            _ => operand_range.clone()
        };
        let a_base_constraint = ExpressionNumberConstraint {
            description: format!("{}..={} (from op1 {}, op2 {})", a_base_range.start(), a_base_range.end(), &op1, &op2_str),
            range: a_base_range,
            ..Default::default()
        };
        let a_constraint = &ExpressionNumberConstraint::intersect(&a_base_constraint, &constraint.a_constraint);
        let a = skip_fail!(find_num_with_constraint(&mut rng, a_constraint), "Failed to generate a");
        // println!("Generated a {} from constraint: {}", a, &a_constraint);

        let mut b_base_constraint = ExpressionNumberConstraint {
            range: operand_range.clone(),
            description: format!("{}..={} (from op1 {}, op2 {})", operand_range.start(), operand_range.end(), &op1, &op2_str),
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

        match op2_opt {
            Some(op2) => {
                parts.push(op2op(&op2));

                let b2_base_constraint = ExpressionNumberConstraint {
                    range: operand_range.clone(),
                    description: format!("{}..={}", operand_range.start(), operand_range.end()),
                    ..Default::default()
                };
                let b2 = find_num_with_constraint(&mut rng, &ExpressionNumberConstraint::intersect(&b2_base_constraint, &constraint.b2_constraint))?;
                parts.push(ExpressionPart::Number(b2));
            },
            None => { }
        }

        let expr = Expression { parts };
        let res = skip_fail!(expr.calculate(), format!("Error calculating expression {}", expr));
        match constraint.c_constraint.accept(&res) {
            Err(err) => {
                // println!("c {} from expr {} did not match c_constraint {}: {}", &res, &expr, constraint.c_constraint, err);
                continue;
            },
            Ok(()) => { }
        }

        let eq = Equation { expr, res };
        if eq.len() != NERDLE_CHARACTERS as usize {
            // println!("Equation '{}' is wrong length ({} chars != {})", eq, eq.len(), NERDLE_CHARACTERS);
            continue;
        }
        if !eq.computes().unwrap_or(false) {
            println!("Equation unexpectedly did not compute: {}", eq);
            continue;
        }

        match constraint.accept(&eq) {
            Err(err) => {
                // println!("Equation {} did not match constraint: {}", eq, err);
                continue;
            },
            Ok(()) => { }
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

fn gen_ops(constraint: &EquationConstraint) -> Result<(ExpressionOperatorEnum, Option<ExpressionOperatorEnum>), NoMatchFound> {
    // let must_have: Vec<u8> = constraint.operator.iter().filter(|(_op, info)| info.start() > &0).map(|(op, _info)| *op).collect();
    // let may_have: Vec<u8> = constraint.operator.iter().filter(|(_op, info)| info.end() > &0).map(|(op, _info)| *op).collect();

    // fn b2op(b: &u8) -> Result<ExpressionOperatorEnum, NoMatchFound> {
    //     match ExpressionOperatorEnum::from_char_byte(&b) {
    //         Ok(op) => Ok(op),
    //         Err(err) => Err(NoMatchFound { message: format!("Invalid operator: {}", err)})
    //     }
    // }

    let num_ops_range = &constraint.num_ops;
    let num_ops = if num_ops_range.start() == num_ops_range.end() {
        // We know how many there are
        *(num_ops_range.start())
    } else {
        range_rand_or_only(constraint.num_ops.clone())?
    };

    return Ok((
        gen_operator_constrained(&constraint),
        if num_ops == 1 {
            None
        } else {
            Some(gen_operator_constrained(&constraint))
        }
    ));
    // let op1 = gen_operator_constrained(&constraint);
    // let op
    // if num_ops == 1 {
    //     if must_have.len() > 0 {
    //         // TODO: Randomly choose order
    //         return Ok((b2op(&must_have[0])?, None));
    //     } else {
    //         return Ok((b2op(&may_have[0])?, None));
    //     }
    // } else {
    //     // 2 operators
    //     if must_have.len() == 2 {
    //         // We can just choose the order
    //         // TODO: Randomly choose order
    //         return Ok((b2op(&must_have[0])?, Some(b2op(&must_have[1])?)));
    //     } else if must_have.len() > 0 {
    //         // TODO: Randomly choose order
    //         return Ok((b2op(&must_have[0])?, Some(b2op(&may_have[0])?)));
    //     } else {
    //         return Ok((b2op(&may_have[0])?, Some(b2op(&may_have[1])?)));
    //     }
    // }
}

pub fn gen_operator_constrained(constraint: &EquationConstraint) -> ExpressionOperatorEnum {
    loop {
        let tmp_op: ExpressionOperatorEnum = rand::random();
        let tmp_op_ch = tmp_op.to_string().as_bytes()[0];
        if !constraint.can_have_op_byte(tmp_op_ch) {
            // println!("Rejected operator '{}' because it's been ruled out", tmp_op_ch as char);
            continue;
        }
        // println!("Accepted operator '{}'", tmp_op_ch as char);
        break tmp_op;
    }
}