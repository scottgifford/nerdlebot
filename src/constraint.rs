// TODO: Which of these do we need?
use std::fmt;
use rand::Rng;
use std::ops::RangeInclusive;

use crate::expr::{ExpressionNumber, mknum};

const ATTEMPTS: u32 = 1000;

pub struct ExpressionNumberConstraint<F>
where
    F: Fn(&ExpressionNumber) -> bool,
{
    pub range: RangeInclusive<u32>,
    pub description: String,
    pub accept: F,
}

impl<F> fmt::Display for ExpressionNumberConstraint<F>
where
    F: Fn(&ExpressionNumber) -> bool,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExpressionNumberConstraint \"{}\": range={}..={}", self.description, self.range.start(), self.range.end())
    }
}


pub fn find_num_with_constraint<F>(rng: &mut impl Rng, constraint: &ExpressionNumberConstraint<F>) -> Result<ExpressionNumber, NoMatchFound>
where
    F: Fn(&ExpressionNumber) -> bool,
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

#[derive(Clone)]
pub struct NoMatchFound {
    pub message: String,
}

impl fmt::Display for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoMatchFound
    : {}", self.message)
    }
}

impl fmt::Debug for NoMatchFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "NoMatchFound
    : {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}
