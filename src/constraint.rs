use std::fmt;
use std::rc::Rc;
use std::cmp::{min, max};
use rand::Rng;
use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::expr::{ExpressionNumber, mknum};
use crate::eq::{Equation};

const ATTEMPTS: u32 = 1000;

pub struct ExpressionNumberConstraint
{
    pub range: RangeInclusive<u32>,
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

impl fmt::Display for ExpressionNumberConstraint
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExpressionNumberConstraint \"{}\": range={}..={}", self.description, self.range.start(), self.range.end())
    }
}

pub fn find_num_with_constraint(rng: &mut impl Rng, constraint: &ExpressionNumberConstraint) -> Result<ExpressionNumber, NoMatchFound>
{
    if constraint.range.is_empty() {
        return Err(NoMatchFound { message: format!("Invalid range in constraint: {}", constraint)});
    }

    for _try in 1..ATTEMPTS {
        let candidate = rng.gen_range(constraint.range.clone());
        let candidate = mknum(candidate);
        if !(constraint.accept)(&candidate) {
            // println!("  Rejected {} with constraint {}", candidate, constraint);
            continue;
        }
        return Ok(candidate);
    }
    Err(NoMatchFound { message: format!("No match found for constraint {} after {} tries", constraint, ATTEMPTS)})
}

pub struct EquationConstraint<F>
    where F: Fn(&Equation) -> bool,
{
    pub accept: F,
    // TODO: Convert below to ExpressionNumberConstraint
    pub a_range: RangeInclusive<u32>,
    pub b_range: RangeInclusive<u32>,
    pub c_range: RangeInclusive<u32>,
    pub operator: HashMap<u8, bool>,
}

impl<F> EquationConstraint<F>
    where F: Fn(&Equation) -> bool,
{
    pub fn new(accept: F) -> EquationConstraint<F> {
        EquationConstraint {
            accept,
            a_range: 0..=99999,
            b_range: 0..=99999,
            c_range: 0..=99999,
            operator: HashMap::new(),
        }
    }
}

pub fn write_formatted_range(f: &mut fmt::Formatter, range: &RangeInclusive<u32>) -> fmt::Result {
    write!(f, "{}..={}", range.start(), range.end())
}

impl<F> fmt::Display for EquationConstraint<F>
    where F: Fn(&Equation) -> bool,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operator is {} and is not: (", self.operator.iter().find(|(_key, ent)| **ent).map(|(key, _ent)| *key as char).unwrap_or('?'))?;
        for (key, ent) in self.operator.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if !ent { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, ")")?;

        write!(f, ", a: ")?;
        write_formatted_range(f, &self.a_range)?;
        write!(f, ", b:")?;
        write_formatted_range(f, &self.b_range)?;
        write!(f, ", c:")?;
        write_formatted_range(f, &self.c_range)
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
