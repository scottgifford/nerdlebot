use std::rc::Rc;
use std::collections::HashMap;

use crate::eq::Equation;
use crate::nerdle::{NERDLE_CHARACTERS, NERDLE_A_MAX};
use crate::expr::{Expression, ExpressionPart, ExpressionOperator, ExpressionOperatorPlus, ExpressionOperatorMinus, ExpressionOperatorTimes, ExpressionOperatorDivide, ExpressionOperatorEnum};
use crate::constraint::{find_num_with_constraint, EquationConstraint, ExpressionNumberConstraint, NoMatchFound, range_for_digits_or_less};
use crate::util::{range_rand_or_only};

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
    // println!("Incoming constraint: {}", constraint);

    for _try in 1..ATTEMPTS {
        let mut parts: Vec<ExpressionPart> = Vec::new();
        let mut remaining_chars = NERDLE_CHARACTERS as usize;

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
        // println!("Finding value for a");
        let a = skip_fail!(find_num_with_constraint(a_constraint), "Failed to generate a");
        remaining_chars -= a.len();
        remaining_chars -= 1; // op1
        // println!("Generated a {} from constraint: {}", a, &a_constraint);

        let b_remaining_chars = remaining_chars
            - if op2_opt.is_some() { 2 } else { 0 } // second op and b2 (if present)
            - 2 // =c
        ;
        let b_range = range_for_digits_or_less(b_remaining_chars, false);
        let mut b_base_constraint = ExpressionNumberConstraint {
            description: format!("{}..={} for {}-digit number", b_range.start(), b_range.end(), b_remaining_chars),
            range: b_range,
            ..Default::default()
        };

        // Optimize b selection for one-operator case
        if op2_str.is_empty() {
            if let ExpressionPart::Operator(op1_obj) = op2op(&op1) {
                let a_clone = a.clone();
                let c_constraint_accept = constraint.c_constraint.accept.clone();
                b_base_constraint.accept = Rc::new(move |b| {
                    let c = op1_obj.operate(&a_clone, &b);
                    match c {
                        Err(_) => false,
                        Ok(c) => c_constraint_accept(&c)
                    }
                });
            }
        }
        let b_constraint = &ExpressionNumberConstraint::intersect(&b_base_constraint, &constraint.b_constraint);
        // println!("Finding value for b");
        let b = skip_fail!(find_num_with_constraint(&b_constraint), "Failed to generate b");
        // println!("Generated b {} from constraint: {}", b, &b_constraint);

        remaining_chars -= b.len();

        parts.push(ExpressionPart::Number(a));
        parts.push(op2op(&op1));
        parts.push(ExpressionPart::Number(b));

        match op2_opt {
            Some(op2) => {
                parts.push(op2op(&op2));
                let b2_remaining_chars = remaining_chars
                    - 1 // op2
                    - 2 // =c
                ;

                let range = range_for_digits_or_less(b2_remaining_chars, false);
                let b2_base_constraint = ExpressionNumberConstraint {
                    description: format!("{}..={} for {}-digit number", range.start(), range.end(), b_remaining_chars),
                    range,
                    ..Default::default()
                };
                // println!("Finding value for b2");
                let b2 = find_num_with_constraint(&ExpressionNumberConstraint::intersect(&b2_base_constraint, &constraint.b2_constraint))?;
                parts.push(ExpressionPart::Number(b2));
            },
            None => { }
        }

        let expr = Expression { parts };
        let res = skip_fail!(expr.calculate(), format!("Error calculating expression {}", expr));
        match constraint.c_constraint.accept(&res) {
            Err(_err) => {
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
            Err(_err) => {
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
    let mut tries = 0;
    return Ok(loop {
        tries += 1;
        if tries > ATTEMPTS {
            return Err(NoMatchFound { message: format!("Could not find operator after {} tries for constraint {}", tries, &constraint)})
        }

        let num_ops = range_rand_or_only(constraint.num_ops.clone())?;

        let (op1, op2_opt): (ExpressionOperatorEnum, Option<ExpressionOperatorEnum>) =
            if num_ops == 1 {
                ( rand::random(), None )
            } else {
                ( rand::random(), Some(rand::random()) )
            }
        ;

        if are_ops_ok(&op1, &op2_opt, &constraint, tries) {
            break (op1, op2_opt);
        } else {
            continue;
        }
    })
}

fn are_ops_ok(op1: &ExpressionOperatorEnum, op2_opt: &Option<ExpressionOperatorEnum>, constraint: &EquationConstraint, _tries: u32) -> bool {
    let mut op_count: HashMap<u8, u32> = HashMap::new();
    op_count.insert(op1.to_char_byte(), 1);
    match &op2_opt {
        Some(op2) => {
            let count = op_count.entry(op2.to_char_byte()).or_insert(0);
            *count += 1;
        },
        None => { }
    }

    // let op2_str = || match &op2_opt {
    //     Some(op2) => op2.to_string(),
    //     None => String::from("None")
    // };

    for (op, info) in constraint.operator.iter() {
        match op_count.get(op) {
            None => {
                if info.start() >= &1 {
                    // println!("Rejected case 1 operators ({}, {}) on try {}: operator {} should appear at least {} times but appeared {}, constraint: {}",
                    //     &op1, op2_str(), tries,
                    //     *op as char, info.start(), "none",
                    //     &constraint);
                    return false
                } else {
                    // println!("Passed case 1 operators ({}, {}) on try {}: operator {} should appear at least {} times and appeared {}, constraint: {}",
                    //     &op1, op2_str(), tries,
                    //     *op as char, info.start(), "none",
                    //     &constraint);
                }
            },
            Some(count) => {
                if count < info.start() || count > info.end() {
                    // println!("Rejected case 2 operators ({}, {}) on try {}: operator {} should appear between {} and {} times and appeared {}, constraint {}",
                    //     &op1, op2_str(), tries,
                    //     *op as char, info.start(), info.end(), count,
                    //     &constraint);
                    return false;
                } else {
                    // println!("Passed case 2 operators ({}, {}) on try {}: operator {} should appear between {} and {} times and appeared {}, constraint {}",
                    //     &op1, op2_str(), tries,
                    //     *op as char, info.start(), info.end(), count,
                    //     &constraint);
                }
            }
        }
    }

    // println!("Accepted operators ({}, {}) on try {} from constraint: {}", &op1, op2_str(), tries, &constraint);
    true
}

#[cfg(test)]
#[test]
fn are_ops_ok_test() {
    // Simple cases with no constraints
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &None,
        &EquationConstraint::default(),
        1
    ));
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Minus,
        &None,
        &EquationConstraint::default(),
        1
    ));
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Divide,
        &None,
        &EquationConstraint::default(),
        1
    ));
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Times,
        &None,
        &EquationConstraint::default(),
        1
    ));


    // Simple cases where operators meet constraints
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &None,
        &EquationConstraint {
            operator: HashMap::from([
                ('+' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Minus,
        &None,
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Divide,
        &None,
        &EquationConstraint {
            operator: HashMap::from([
                ('/' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Times,
        &None,
        &EquationConstraint {
            operator: HashMap::from([
                ('*' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    // Must appear once, and appears in second operator
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Minus,
        &Some(ExpressionOperatorEnum::Plus),
        &EquationConstraint {
            operator: HashMap::from([
                ('+' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Divide),
        &EquationConstraint {
            operator: HashMap::from([
                ('/' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Times),
        &EquationConstraint {
            operator: HashMap::from([
                ('*' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    // Must appear twice, and does
    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Plus),
        &EquationConstraint {
            operator: HashMap::from([
                ('+' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Minus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Divide,
        &Some(ExpressionOperatorEnum::Divide),
        &EquationConstraint {
            operator: HashMap::from([
                ('/' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(are_ops_ok(
        &ExpressionOperatorEnum::Times,
        &Some(ExpressionOperatorEnum::Times),
        &EquationConstraint {
            operator: HashMap::from([
                ('*' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));

    // Operator does not appear but must
    assert!(!are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &None,
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 1..=1),
            ]),
            ..Default::default()
        },
        1
    ));

    // Operator appears but must not
    assert!(!are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('+' as u8, 0..=0),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(!are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 0..=0),
            ]),
            ..Default::default()
        },
        1
    ));

    // Operator appears once but must appear twice
    assert!(!are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('-' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));

    assert!(!are_ops_ok(
        &ExpressionOperatorEnum::Plus,
        &Some(ExpressionOperatorEnum::Minus),
        &EquationConstraint {
            operator: HashMap::from([
                ('+' as u8, 2..=2),
            ]),
            ..Default::default()
        },
        1
    ));
}
