use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::cmp::{max};

use crate::eq::Equation;
use crate::expr::{ExpressionPart};
use crate::nerdle::{NerdleResult, NerdlePositionResult, NerdleError, NERDLE_CHARACTERS, NERDLE_NUM_MAX};
use crate::eqgen::{eqgen_constrained};
use crate::constraint::{EquationConstraint, ExpressionNumberConstraint, NoMatchFound};

const VALID_CHAR_STR: &str = "1234567890-+*/=";
const OPERATOR_STR: &str = "-+*/";

#[derive(Clone, Copy, Debug)]
pub enum NerdleIsChar {
    Definitely,
    DefinitelyNot,
    Maybe,
}

#[derive(Clone, Debug)]
pub struct NerdleCharInfo {
    pub min_count: u32,
    pub max_count: u32,
    positions: [NerdleIsChar; NERDLE_CHARACTERS as usize]
}

impl NerdleCharInfo {
    pub fn new() -> NerdleCharInfo {
        NerdleCharInfo {
            min_count: 0,
            max_count: NERDLE_CHARACTERS,
            positions: [ NerdleIsChar::Maybe; NERDLE_CHARACTERS as usize ],
        }
    }
}

impl fmt::Display for NerdleCharInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CharInfo: min={}, max={}", self.min_count, self.max_count)
    }
}

struct NerdleSolverData {
    pub char_info: HashMap<u8, NerdleCharInfo>,
    pub positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize],
    pub equal_pos: Option<usize>,
    pub op: Option<u8>,
    pub op_pos: Option<usize>,
}

impl Default for NerdleSolverData {
    fn default() -> Self {
        let positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize] = Default::default();
        NerdleSolverData {
            char_info: HashMap::new(),
            positions,
            equal_pos: None,
            op_pos: None,
            op: None,
        }
    }
}

impl NerdleSolverData {
        // TODO: Switch to a better error type
        pub fn eq_matches(&self, eq: &Equation) -> Result<(), NerdleError> {
            if !eq.computes().unwrap_or(false) {
                return Err(NerdleError { message: format!("Equation does not compute")});
            }

            let eq_str = eq.to_string();
            let eq_bytes = eq_str.as_bytes();
            if eq_str.len() != NERDLE_CHARACTERS as usize {
                return Err(NerdleError { message: format!("Equation as string is too many characters ({} != {})", eq_str.len(), NERDLE_CHARACTERS)});
            }

            // Check characters in positions
            for pos in 0..(NERDLE_CHARACTERS as usize) {
                let guess_ch = eq_bytes[pos];
                match self.positions[pos].get(&guess_ch) {
                    Some(false) => return Err(NerdleError { message: format!("Position {} cannot be {}", pos, guess_ch as char)}),
                    _ => { }
                }
            }

            // Check the operator
            match self.op {
                Some(op) => {
                    match eq.expr.parts.iter().find(|x| match x {
                        ExpressionPart::Operator(_) => true,
                        _ => false
                    }) {
                        Some(eq_op) => {
                            // TODO: Seems hacky
                            let eq_op_str = eq_op.to_string();
                            if (op as char).to_string() != eq_op_str {
                                println!("Rejecting based on operator, got {} but expected {}", eq_op_str, op.to_string());
                                return Err(NerdleError { message: format!("Equation had operator {} but expected {}", eq_op_str, op.to_string())});
                            }
                        }
                        None => return Err(NerdleError { message: format!("Equation had no operator somehow?!")})
                    }
                }
                None => { }
            }

            // Now check counts
            let mut char_counts = HashMap::new();
            for &ch in eq_bytes.iter() {
                let counter = char_counts.entry(ch).or_insert(0);
                *counter += 1;
            }
            for (ch, count) in char_counts.iter() {
                match self.char_info.get(ch) {
                    Some(info) => {
                        if count < &info.min_count {
                            return Err(NerdleError { message: format!("Not enough of character '{}' ({} < {})", *ch as char, count, info.min_count) })
                        }
                        if count > &info.max_count {
                            return Err(NerdleError { message: format!("Too many of character '{}' ({} > {})", *ch as char, count, info.max_count) })
                        }
                    },
                    None => { }
                }
            }

            Ok(())
        }
}

pub struct NerdleSolver {
    data: Rc<RefCell<NerdleSolverData>>,
}

impl NerdleSolver {
    pub fn new() -> NerdleSolver {
        NerdleSolver {
            data: Rc::new(RefCell::new(NerdleSolverData::default())),
        }
    }

    pub fn take_guess(&self) -> Result<Equation, NoMatchFound> {
        let mut constraint = EquationConstraint {
            accept: {
                let data: Rc<RefCell<NerdleSolverData>> = self.data.clone();
                Rc::new(move |eq| {
                    match data.borrow().eq_matches(&eq) {
                        Ok(()) => true,
                        Err(_e) => {
                            // println!("  Equation {} not possible because {}", eq, e);
                            false
                        }
                    }
                })
            },
            ..Default::default()
        };

        let data = self.data.borrow();
        for op in OPERATOR_STR.as_bytes().iter() {
            match data.op {
                Some(op2) if *op == op2 => { constraint.operator.insert(*op, true); },
                Some(_) => { constraint.operator.insert(*op, false); },
                None => match data.char_info.get(op) {
                    Some(info) => if info.max_count < 1 {
                        constraint.operator.insert(*op, false);
                    },
                    None => { }
                },
            };
        }

        match data.op {
            Some(op) => { constraint.operator.insert(op, true); },
            _ => {},
        };

        match data.equal_pos {
            Some(pos) => {
                let digits = NERDLE_CHARACTERS as usize - pos - 1;
                let range = range_for_digits(digits);
                let description = format!("Updating c_range to {}..={} because = is in pos {} leaving {} digits", range.start(), range.end(), pos, digits);
                // TODO: Also add a callback with a regex of acceptable characters
                constraint.c_range = ExpressionNumberConstraint {
                    range,
                    description,
                    ..Default::default()
                };
            },
            _ => {}
        };

        match data.op_pos {
            Some(op_pos) => {
                let digits = op_pos;
                let range = range_for_digits(digits);
                // TODO: Also add a callback with a regex of acceptable characters
                let description = format!("Updating a_range to {}..={} because op is in pos {} leaving {} digits", range.start(), range.end(), op_pos, digits);
                constraint.a_range = ExpressionNumberConstraint {
                    range,
                    description,
                    ..Default::default()
                };
                match data.equal_pos {
                    Some(equal_pos) => {
                        let digits = equal_pos - op_pos - 1;
                        let range = range_for_digits(digits);
                        // TODO: Also add a callback with a regex of acceptable characters
                        let description = format!("Updating b_range to {}..={} because op is in pos {} and equal in pos {} leaving {} digits", range.start(), range.end(), op_pos, equal_pos, digits);
                        constraint.b_range = ExpressionNumberConstraint {
                            range,
                            description,
                            ..Default::default()
                        };
                    },
                    _ => {}
                }
            },
            _ => {}
        };

        println!("Constraint: {}", &constraint);

        let mut r = eqgen_constrained(&constraint);
        for _ in 0..100 {
            if r.is_ok() {
                return r;
            }
            r = eqgen_constrained(&constraint);
        }
        r
    }

    // TODO: Switch to a better error type
    pub fn eq_matches(&self, eq: &Equation) -> Result<(), NerdleError> {
        self.data.borrow().eq_matches(eq)
    }

    pub fn update(&mut self, guess: &Equation, result: &NerdleResult) {
        let mut data = self.data.borrow_mut();

        // let mut state = ParseState::InA;
        let guess_str = guess.to_string();
        let guess = guess_str.as_bytes();

        // First count the total letters (it is hard to take GREENs into account as we go)
        let mut char_occ_count: HashMap<u8, u32> = HashMap::new();
        let mut found_max: HashMap<u8, bool> = HashMap::new();

        for i in 0..NERDLE_CHARACTERS as usize {
            let guess_ch = guess[i];
            let counter = char_occ_count.entry(guess_ch).or_insert(0);
            let new_char_pos_info: NerdleIsChar;
            // First handle general-purpose logic
            match result.positions[i] {
                NerdlePositionResult::Green => {
                    data.positions[i].insert(guess_ch, true);
                    for ch in VALID_CHAR_STR.as_bytes().iter() {
                        if *ch != guess_ch {
                            data.positions[i].insert(*ch, false);
                        }
                    }
                    new_char_pos_info = NerdleIsChar::Definitely;
                    *counter += 1
                },
                NerdlePositionResult::Yellow => {
                    data.positions[i].insert(guess_ch, false);
                    new_char_pos_info = NerdleIsChar::DefinitelyNot;
                    *counter += 1;
                },
                NerdlePositionResult::Gray => {
                    data.positions[i].insert(guess_ch, false);
                    new_char_pos_info = NerdleIsChar::DefinitelyNot;
                    found_max.insert(guess_ch, true);
                }
            }

            {
                let char_info = data.char_info.entry(guess_ch).or_insert(NerdleCharInfo::new());
                (*char_info).positions[i] = new_char_pos_info;
            }

            // Special handling for equal sign and operators
            match guess_ch as char {
                '=' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        data.equal_pos = Some(i);
                    },
                    _ => { }
                },
                '+' | '-' | '*' | '/' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        data.op_pos = Some(i);
                        data.op = Some(guess_ch);
                    },
                    NerdlePositionResult::Yellow => {
                        data.op = Some(guess_ch);
                    },
                    _ => { }
                },
                _ => { }
            }
        }

        for (ch, count) in char_occ_count.iter() {
            let mut ent = data.char_info.entry(*ch).or_insert(NerdleCharInfo::new());
            (*ent).min_count = max((*ent).min_count, *count);
            if found_max.contains_key(ch) {
                (*ent).max_count = *count;
            }
        }
    }

    pub fn print_hint(&self) {
        let data = self.data.borrow();

        print!("Equal sign ");
        match data.equal_pos {
            Some(x) => print!("at {}", x),
            None => {
                print!("not at ");
                match data.char_info.get(&('=' as u8)) {
                    Some(x) => {
                        for pos in 0..(NERDLE_CHARACTERS as usize) {
                            match x.positions[pos] {
                                NerdleIsChar::DefinitelyNot => print!("{} ", pos),
                                _ => { }
                            }
                        }
                    },
                    None => { }
                }
            }
        }
        println!("");

        print!("Operator is {} at {} and is not: ", data.op.map(|x| x as char).unwrap_or('?'), data.op_pos.map(|x| x.to_string()).unwrap_or("?".to_string()));
        for (key, ent) in data.char_info.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.max_count < 1 { print!("{} ", *key as char); },
                _ => { }
            }
        }
        println!("");

        let mut known_pos: HashMap<u8, u32> = HashMap::new();
        for (key, val) in data.char_info.iter() {
            known_pos.insert(*key, val.positions.iter().fold(0, |sum, status| sum + match status {
                NerdleIsChar::Definitely => 1,
                _ => 0
            }));
        }

        for pos in 0..(NERDLE_CHARACTERS as usize) {
            print!("Position {} ", pos);
            let poss = self.possibilities_for_pos(pos);
            match poss.len() {
                0 => print!("NO POSSIBILITIES?!"),
                1 => print!("is"),
                _ => print!("could be")
            }
            let mut sorted: Vec<&u8> = poss.iter().collect::<Vec<_>>();
            sorted.sort();
            for p in sorted.iter() {
                print!(" {}", **p as char);
            }
            print!("\n");
        }
    }

    fn possibilities_for_pos(&self, pos: usize) -> HashSet<u8> {
        let data = self.data.borrow();

        // TODO: Move to state or something?  Or maybe this should be done up update() to pre-calculate all this?
        let mut known_pos: HashMap<u8, u32> = HashMap::new();
        for (key, val) in data.char_info.iter() {
            known_pos.insert(*key, val.positions.iter().fold(0, |sum, status| sum + match status {
                NerdleIsChar::Definitely => 1,
                _ => 0
            }));
        }

        let mut ret = HashSet::new();
        match data.positions[pos].iter().find_map(|(key, value)| if *value { Some(key) } else { None }) {
            Some(known) => { ret.insert(*known); },
            None => {
                for ch in VALID_CHAR_STR.as_bytes().iter() {
                    let info: Option<&NerdleCharInfo> = data.char_info.get(&ch);
                    let known_ch_count = known_pos.get(ch).unwrap_or(&0);
                    let max_ch_count = info.map(|x| x.max_count).unwrap_or(NERDLE_CHARACTERS);
                    if known_ch_count >= &max_ch_count {
                        continue;
                    }
                    let char_pos_info = info.map(|x| &x.positions[pos]).unwrap_or(&NerdleIsChar::Maybe);
                    // println!("  ch {} pos {} known {} info {}", *ch as char, pos, known, info);         
                    if match char_pos_info {
                        NerdleIsChar::Definitely => true, // Should never happen
                        NerdleIsChar::DefinitelyNot => false,
                        NerdleIsChar::Maybe => match *ch as char {
                            '=' => data.equal_pos == None,
                            '+' | '-' | '*' | '/' => match data.op {
                                None => true,
                                Some(x) if x == *ch => match data.op_pos {
                                    None => true,
                                    Some(y) if y == pos => true,
                                    Some(_) => false,
                                }
                                Some(_) => false,
                            },
                            '0'..='9' => true,
                            _ => panic!("Unexpected character '{}'", *ch)
                        }
                    } {
                        ret.insert(*ch);
                    }
                };
            }
        }

        ret
    }
}

impl fmt::Display for NerdleSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();

        write!(f, "Equal sign at: {}\n", data.equal_pos.map(|x| x.to_string()).unwrap_or("?".to_string()))?;

        write!(f, "Operator is {} at {} and is not: ", data.op.map(|x| x as char).unwrap_or('?'), data.op_pos.map(|x| x.to_string()).unwrap_or("?".to_string()))?;
        for (key, ent) in data.char_info.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.max_count < 1 { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, "\n")?;

        for ch in VALID_CHAR_STR.as_bytes().iter() {
            write!(f, "Character {}: {}\n", *ch as char, data.char_info.get(ch).unwrap_or(&NerdleCharInfo::new()))?;
        }

        for pos in 0..(NERDLE_CHARACTERS as usize) {
            write!(f, "Position {} is not: ", pos)?;
            for (key, ent) in data.positions[pos].iter() {
                match ent {
                    false => write!(f, "{} ", *key as char)?,
                    _ => {}
                }
            }
            write!(f, "\n")?;
        }

        // TODO: Create "could be" list
        Ok(())
    }
}

fn range_for_digits(digits: usize) -> RangeInclusive<u32> {
    match digits {
        1 => 1..=9,
        2 => 10..=99,
        3 => 100..=999,
        4 => 1000..=9999,
        _ => 1..=NERDLE_NUM_MAX,
    }
}
