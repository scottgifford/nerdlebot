use std::fmt;
use std::rc::Rc;
use std::cmp::{min, max};
use rand::Rng;
use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::nerdle::{NERDLE_NUM_MAX, NERDLE_MAX_OPS};
use crate::expr::{ExpressionNumber, mknum};
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

// TODO: No longer need rng object
pub fn find_num_with_constraint(_rng: &mut impl Rng, constraint: &ExpressionNumberConstraint) -> Result<ExpressionNumber, NoMatchFound>
{
    for _try in 1..ATTEMPTS {
        let candidate = match range_rand_or_only(constraint.range.clone()) {
            Ok(num) => num,
            Err(err) => return Err(NoMatchFound { message: format!("Could not find possibility for constraint {}: {}", constraint, err)}),
        };
        let candidate = mknum(candidate);
        if !(constraint.accept)(&candidate) {
            // println!("  Rejected {} with constraint {}", candidate, constraint);
            continue;
        }
        return Ok(candidate);
    }
    Err(NoMatchFound { message: format!("No match found for constraint {} after {} tries", constraint, ATTEMPTS)})
}

pub struct EquationConstraint
{
    pub accept: Rc<dyn Fn(&Equation) -> bool>,
    pub a_constraint: ExpressionNumberConstraint,
    pub b_constraint: ExpressionNumberConstraint,
    pub b2_constraint: ExpressionNumberConstraint,
    pub c_constraint: ExpressionNumberConstraint,
    pub operator: HashMap<u8, bool>,
    pub num_ops: RangeInclusive<u32>,
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
        })
    }
}

impl fmt::Display for EquationConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{} Operator(s), (", self.num_ops.start(), self.num_ops.end())?;
        for (key, ent) in self.operator.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if *ent { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, ") and not (")?;
        for (key, ent) in self.operator.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if !*ent { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, ")")?;

        write!(f, ", a: {}, b: {}, c: {}", &self.a_constraint, &self.b_constraint, &self.c_constraint)
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
