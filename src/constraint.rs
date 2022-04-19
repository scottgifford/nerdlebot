use std::fmt;
use std::rc::Rc;
use std::cmp::{min, max};
use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::nerdle::{NERDLE_NUM_MAX, NERDLE_MAX_OPS};
use crate::expr::{ExpressionNumber, ExpressionOperator, ExpressionPart, mknum};
use crate::eq::{Equation};
use crate::util::range_rand_or_only;

const ATTEMPTS: u32 = 1000;
const DEFAULT_RANGE: RangeInclusive<i32> = 0..=NERDLE_NUM_MAX;

thread_local! {
    // Must be thread_local because Rc is not threadsafe
    static ACCEPT_ANY_EXPRESSION_NUMBER_RC: Rc<dyn Fn(&ExpressionNumber) -> bool> = Rc::new(|_| true);
    static ACCEPT_ANY_EQUATION_RC: Rc<dyn Fn(&Equation) -> bool> = Rc::new(|_| true);
}

pub struct ExpressionNumberConstraint
{
    pub range: RangeInclusive<i32>,
    pub description: String,
    pub accept: Rc<dyn Fn(&ExpressionNumber) -> bool>,
}

impl ExpressionNumberConstraint {
    pub fn intersect(a: &ExpressionNumberConstraint, b: &ExpressionNumberConstraint) -> ExpressionNumberConstraint {
        let a_accept = a.accept.clone();
        let b_accept = b.accept.clone();
        ExpressionNumberConstraint {
            range: range_intersect(&a.range, &b.range),
            description: format!("{} & {}", &a.description, &b.description),
            accept: Rc::new(move |n| {
                (a_accept)(n) && (b_accept)(n)
            }),
        }
    }

    pub fn accept(&self, num: &ExpressionNumber) -> Result<(), NoMatchFound> {
        match num.int_value() {
            Ok(value) => {
                if !self.range.contains(&value) {
                    Err(NoMatchFound { message: format!("Value {} is not in range: {}", num, self)})
                } else if !(self.accept)(&num) {
                    Err(NoMatchFound { message: format!("Value {} did not match accept function: {}", num, self)})
                } else {
                    Ok(())
                }
            },
            Err(err) => Err(NoMatchFound { message: format!("Value {} was not an integer: {}", num, err)})
        }
    }
}

impl Default for ExpressionNumberConstraint {
    fn default() -> Self {
        ACCEPT_ANY_EXPRESSION_NUMBER_RC.with(|accept_anything| Self {
            range: DEFAULT_RANGE.clone(),
            description: format!("Default range: {}..{}", DEFAULT_RANGE.start(), DEFAULT_RANGE.end()),
            accept: accept_anything.clone(),
        })
    }
}

impl fmt::Display for ExpressionNumberConstraint
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExpressionNumberConstraint \"{}\": range={}..={}", self.description, self.range.start(), self.range.end())
    }
}

pub fn find_num_with_constraint(constraint: &ExpressionNumberConstraint) -> Result<ExpressionNumber, NoMatchFound>
{
    let constraint_range_size = (constraint.range.end() - constraint.range.start() + 1) as u32;
    for _attempt in 0..min(constraint_range_size, ATTEMPTS) {
        let candidate = match range_rand_or_only(constraint.range.clone()) {
            Ok(num) => num,
            Err(err) => return Err(NoMatchFound { message: format!("Could not find possibility for constraint {}: {}", constraint, err)}),
        };
        let candidate = mknum(candidate);
        if !(constraint.accept)(&candidate) {
            // println!("  Rejected {} with constraint {}", candidate, constraint);
            continue;
        }
        // println!("Found num {} in {} tries for constraint {}", candidate, attempt, constraint);

        return Ok(candidate);
    }
    // println!("Could not fund num in {} tries for constraint {}", ATTEMPTS, constraint);
    Err(NoMatchFound { message: format!("No match found for constraint {} after {} tries", constraint, ATTEMPTS)})
}

pub struct EquationConstraint
{
    pub accept: Rc<dyn Fn(&Equation) -> bool>,
    pub a_constraint: ExpressionNumberConstraint,
    pub b_constraint: ExpressionNumberConstraint,
    pub b2_constraint: ExpressionNumberConstraint,
    pub c_constraint: ExpressionNumberConstraint,
    pub operator: HashMap<u8, RangeInclusive<u32>>,
    pub num_ops: RangeInclusive<u32>,
    pub accept_description: String,
}

impl Default for EquationConstraint {
    fn default() -> Self {
        ACCEPT_ANY_EQUATION_RC.with(|accept_anything| Self {
            accept: accept_anything.clone(),
            a_constraint: ExpressionNumberConstraint::default(),
            b_constraint: ExpressionNumberConstraint::default(),
            b2_constraint: ExpressionNumberConstraint::default(),
            c_constraint: ExpressionNumberConstraint::default(),
            operator: HashMap::new(),
            num_ops: 1..=NERDLE_MAX_OPS,
            accept_description: "No further contraints".to_string(),
        })
    }
}

impl EquationConstraint {
    pub fn accept(&self, eq: &Equation) -> Result<(),NoMatchFound> {
        let accept_a_op1_b = |a: &ExpressionNumber, op1: &Box<dyn ExpressionOperator>, b: &ExpressionNumber, num_operators: &u32| {
            self.a_constraint.accept(&a)?;
            self.b_constraint.accept(&b)?;
            if !self.num_ops.contains(&num_operators) { // 1 operator
                return Err(NoMatchFound { message: format!("Equation had 1 operator: {}", self)});
            }
            if !self.can_have_op(&op1) {
                return Err(NoMatchFound { message: format!("Equation had disallowed operator {}: {}", &op1, self)});
            }
            Ok(())
        };

        self.c_constraint.accept(&eq.res)?;
        match eq.expr.parts.as_slice() {
            [ExpressionPart::Number(a), ExpressionPart::Operator(op1), ExpressionPart::Number(b)] => {
                accept_a_op1_b(&a, &op1, &b, &1)?;
            },
            [ExpressionPart::Number(a), ExpressionPart::Operator(op1), ExpressionPart::Number(b), ExpressionPart::Operator(op2), ExpressionPart::Number(b2)] => {
                accept_a_op1_b(&a, &op1, &b, &2)?;
                self.b2_constraint.accept(&b2)?;
                if !self.can_have_op(&op2) {
                    return Err(NoMatchFound { message: format!("Equation had disallowed operator: {}", &self)});
                }
            },
            _ => return Err(NoMatchFound { message: format!("Unrecognized pattern for equation: {}", &eq)})
        }
        if !(self.accept)(&eq) {
            return Err(NoMatchFound { message: format!("Accept function failed for constraint: {}", self)})
        }
        Ok(())
    }

    pub fn can_have_op_byte(&self, byte: u8) -> bool {
        self.operator.get(&byte).map(|range| range.end() >= &1).unwrap_or(true)
    }

    pub fn can_have_op(&self, op: &Box<dyn ExpressionOperator>) -> bool {
        let op_str = op.to_string();
        let byte = op_str.as_bytes().iter().next().unwrap();
        self.can_have_op_byte(*byte)
    }
}

impl fmt::Display for EquationConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{} Operator(s), (", self.num_ops.start(), self.num_ops.end())?;
        for (key, ent) in self.operator.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.end() > &0 { write!(f, "{}[{}-{}] ", *key as char, ent.start(), ent.end())?; },
                _ => { }
            }
        }
        write!(f, ") and not (")?;
        for (key, ent) in self.operator.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.end() == &0 { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, ")")?;

        write!(f, ", a: {}, b: {}, b2: {}, c: {}", &self.a_constraint, &self.b_constraint, &self.b2_constraint, &self.c_constraint)?;

        write!(f, ", {}", &self.accept_description)
    }
}

#[derive(Clone)]
pub struct NoMatchFound {
    pub message: String,
}

impl fmt::Display for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoMatchFound: {}", self.message)
    }
}

impl fmt::Debug for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "NoMatchFound: {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}

pub fn range_intersect<T>(a: &RangeInclusive<T>, b: &RangeInclusive<T>) -> RangeInclusive<T>
    where T: Ord + Copy,
{
    RangeInclusive::new(*max(a.start(), b.start()), *min(a.end(), b.end()))
}


#[cfg(test)]
#[test]
fn range_intersect_test() {
    assert_eq!(range_intersect(&(0..=1), &(0..=1)), 0..=1);
    assert_eq!(range_intersect(&(0..=10), &(5..=15)), 5..=10);
    assert_eq!(range_intersect(&(0..=10), &(10..=15)), 10..=10);
    assert!(range_intersect(&(0..=10), &(15..=20)).is_empty());
}

pub fn range_for_digits(digits: usize, allow_zero: bool) -> RangeInclusive<i32> {
    let single_digit_range_start = if allow_zero {
        0
    } else {
        1
    };
    match digits {
        1 => single_digit_range_start..=9,
        2 => 10..=99,
        3 => 100..=999,
        4 => 1000..=9999,
        _ => 1..=NERDLE_NUM_MAX,
    }
}

pub fn range_for_digits_or_less(digits: usize, allow_zero: bool) -> RangeInclusive<i32> {
    let single_digit_range_start = if allow_zero {
        0
    } else {
        1
    };
    match digits {
        1 => single_digit_range_start..=9,
        2 => single_digit_range_start..=99,
        3 => single_digit_range_start..=999,
        4 => single_digit_range_start..=9999,
        _ => single_digit_range_start..=NERDLE_NUM_MAX,
    }
}
