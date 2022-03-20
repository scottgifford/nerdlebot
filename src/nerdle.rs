use crate::eq::Equation;
// use crate::expr;
use std::fmt;
use std::collections::HashMap;

use crate::expr;

pub const NERDLE_CHARACTERS: u32 = 10;
pub const NERDLE_TURNS: u32 = 6;

pub fn nerdle_str(guess: &str, answer: &str) -> Result<NerdleResult, NerdleError> {
    let mut result = NerdleResult { positions: [NerdlePositionResult::Gray; NERDLE_CHARACTERS as usize] };

    if guess.len() != NERDLE_CHARACTERS as usize {
        return Err(NerdleError { message: format!("Guess is {} characters but must be {}", guess.len(), NERDLE_CHARACTERS)})
    }
    let guess = guess.as_bytes();

    if answer.len() != NERDLE_CHARACTERS as usize {
        return Err(NerdleError { message: format!("Answer is {} characters but must be {}", answer.len(), NERDLE_CHARACTERS)})
    }
    let answer = answer.as_bytes();

    // First count everything up
    let mut remaining: HashMap<u8, i32> = HashMap::new();

    for &ch in answer.iter() {
        let counter = remaining.entry(ch).or_insert(0);
        *counter += 1;
    }

    // println!("Initial Counts: {:?}", remaining);

    // First take care of items which are in the right place
    for pos in 0..(NERDLE_CHARACTERS as usize) {
        let guess_pos = guess[pos];
        if guess_pos == answer[pos] {
            result.positions[pos] = NerdlePositionResult::Green;
            remaining.entry(guess_pos).and_modify(|counter| *counter -= 1);
        }
    }
    // println!("Remaining after green: {:?}", remaining);

    // Now take care of any other items
    for pos in 0..(NERDLE_CHARACTERS as usize) {
        let guess_pos = guess[pos];
        // If they are equal we handled them above
        if guess_pos != answer[pos] {
            // TODO: or_insert shouldn't be necessary here, not sure how to simply assert it will be there
            let counter = remaining.entry(guess_pos).or_insert(0);
            // println!("At position {} guess '{}' ({}) remaining {}", pos, guess_pos as char, guess_pos, *counter);
            if *counter > 0 {
                result.positions[pos] = NerdlePositionResult::Yellow;
                *counter -= 1;
            }
        } else {
            // println!("At position {} guess '{}' answer '{}' GREEN", pos, guess_pos as char, answer[pos] as char);
        }
    }

    // println!("Final remaining: {:?}", remaining);

    Ok(result)
}

pub fn nerdle(guess: &Equation, answer: &Equation) -> Result<NerdleResult, NerdleError> {
    if !guess.computes()? {
        return Err(NerdleError { message: format!("Guess does not compute: {}", guess)});
    }
    if !answer.computes()? {
        return Err(NerdleError { message: format!("Answer does not compute: {}", answer)});
    }
    nerdle_str(&guess.to_string(), &answer.to_string())
}

#[derive(Clone, Copy)]
pub enum NerdlePositionResult {
    Yellow,
    Green,
    Gray,
}

impl fmt::Display for NerdlePositionResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            NerdlePositionResult::Yellow => "Y",
            NerdlePositionResult::Green => "G",
            NerdlePositionResult::Gray => "-",
        })
    }
}

pub struct NerdleResult {
    pub positions: [NerdlePositionResult; NERDLE_CHARACTERS as usize],
}

impl NerdleResult {
    pub fn won(&self) -> bool {
        for pos in 0..(NERDLE_CHARACTERS as usize) {
            match self.positions[pos] {
                NerdlePositionResult::Green => { },
                _ => return false
            }
        }
        true
    }
}

impl fmt::Display for NerdleResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for pos in 0..(NERDLE_CHARACTERS as usize) {
            write!(f, "{}", self.positions[pos])?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct NerdleError {
    pub message: String,
}

impl fmt::Display for NerdleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NerdleError: {}", self.message)
    }
}

impl fmt::Debug for NerdleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "NerdleError : {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}

impl From<expr::InvalidExpressionError> for NerdleError {
    fn from(error: expr::InvalidExpressionError) -> Self {
        NerdleError { message : format!("Invalid expression: {}", error) }
    }
}
