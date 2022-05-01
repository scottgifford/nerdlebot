use std::fmt;

use crate::eq::Equation;
use crate::constraint::{NoMatchFound};
use crate::nerdle::{NerdleResult, NerdleError};
use crate::nerdsolver::NerdleSolver;


pub trait Strategy {
    fn take_guess(&self) -> Result<Equation, NoMatchFound>;
    fn update(&mut self, guess: &Equation, result: &NerdleResult);
    fn print_hint(&self);
    fn answer_ok(&self, eq: &Equation) -> Result<(), NerdleError>;
}

pub enum StrategyEnum {
    FirstPossible(NerdleSolver),
}

impl StrategyEnum {
    pub fn by_name(name: &str) -> Result<StrategyEnum, NoSuchStrategyError> {
        match name {
            "first_possible" => Ok(StrategyEnum::FirstPossible(NerdleSolver::new())),
            _ => Err(NoSuchStrategyError { message: format!("No strategy named '{}'", name)})
        }
    }

    pub fn as_strategy(&self) -> &dyn Strategy {
        match self {
            StrategyEnum::FirstPossible(solver) => solver,
        }
    }

    pub fn as_strategy_mut(&mut self) -> &mut dyn Strategy {
        match self {
            StrategyEnum::FirstPossible(solver) => solver,
        }
    }
}

impl Strategy for StrategyEnum {
    fn take_guess(&self) -> Result<Equation, NoMatchFound> {
        self.as_strategy().take_guess()
    }

    fn update(&mut self, guess: &Equation, result: &NerdleResult) {
        self.as_strategy_mut().update(guess, result)
    }

    fn print_hint(&self) {
        self.as_strategy().print_hint()
    }

    fn answer_ok(&self, guess: &Equation) -> Result<(), NerdleError> {
        self.as_strategy().answer_ok(guess)
    }
}

impl fmt::Display for StrategyEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategyEnum::FirstPossible(solver) => solver.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct NoSuchStrategyError {
    pub message: String,
}

impl fmt::Display for NoSuchStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoSuchStrategyError: {}", self.message)
    }
}

impl fmt::Debug for NoSuchStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "NoSuchStrategyError : {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}
