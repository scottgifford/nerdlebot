use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::cmp::{min, max};
use regex::Regex;

use crate::eq::Equation;
use crate::nerdle::{NerdleResult, NerdlePositionResult, NerdleError, NERDLE_CHARACTERS, NERDLE_NUM_MAX, NERDLE_OPERAND_MAX_DIGITS, NERDLE_MAX_OPS};
use crate::eqgen::{eqgen_constrained};
use crate::constraint::{EquationConstraint, ExpressionNumberConstraint, NoMatchFound};
use crate::expr::{ExpressionNumber};

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
}

impl Default for NerdleSolverData {
    fn default() -> Self {
        let positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize] = Default::default();
        NerdleSolverData {
            char_info: HashMap::new(),
            positions,
            equal_pos: None,
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

    pub fn constraint(&self) -> EquationConstraint {
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

        let mut min_ops = 0;
        let mut max_ops = 2;
        for op in OPERATOR_STR.as_bytes().iter() {
            match data.char_info.get(op) {
                Some(info) => {
                    let max_count = min(info.max_count, NERDLE_MAX_OPS);
                    min_ops += info.min_count;
                    max_ops += max_count;
                    constraint.operator.insert(*op, info.min_count..=max_count);
                }
                None => { }
            }
        }
        constraint.num_ops = min_ops..=min(max_ops, NERDLE_MAX_OPS);

        match data.equal_pos {
            Some(pos) => {
                let digits = NERDLE_CHARACTERS as usize - pos - 1;
                let range = range_for_digits(digits, true);
                let description = format!("Updating c_range to {}..={} because = is in pos {} leaving {} digits", range.start(), range.end(), pos, digits);
                constraint.c_constraint = ExpressionNumberConstraint {
                    range,
                    description,
                    ..Default::default()
                };
            },
            _ => {}
        };

        let is_op_at = |pos: usize| {
            let mut ret: Option<bool> = Some(false);
            for op in ['+','-','/','*'] {
                match data.positions[pos].get(&(op as u8)) {
                    Some(true) => return Some(true),
                    Some(false) => { },
                    None => { ret = None; },
                }
            }
            ret
        };

        let op1_pos_opt = (0..NERDLE_CHARACTERS as usize).find(|i| is_op_at(*i as usize).unwrap_or(false));
        let op2_pos_opt = match op1_pos_opt {
            Some(op1_pos) => ((op1_pos+1)..NERDLE_CHARACTERS as usize).find(|i| is_op_at(*i as usize).unwrap_or(false)),
            None => None
        };

        let max_digits = NERDLE_OPERAND_MAX_DIGITS as usize;
        // println!("Pattern check: ({}, {}, {})", op1_pos_opt.unwrap_or(99), op2_pos_opt.unwrap_or(99), data.equal_pos.unwrap_or(99));
        match (op1_pos_opt, op2_pos_opt, data.equal_pos) {
            (Some(op1_pos), Some(op2_pos), Some(equal_pos)) => {
                // println!("Pattern 1: op1_pos={}, op2_pos={}, equal_pos={}", op1_pos, op2_pos, equal_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, false, false, "a");
                constraint.b_constraint = self.constraint_for_digits_start_end(op1_pos, op2_pos, false, false, "b");
                constraint.b2_constraint = self.constraint_for_digits_start_end(op2_pos, equal_pos, false, false, "b2");
                constraint.c_constraint = self.constraint_for_digits_start_end(equal_pos, NERDLE_CHARACTERS as usize, false, true, "c");
                constraint.num_ops = 2..=2;
            },
            (Some(op1_pos), Some(op2_pos), None) => {
                // println!("Pattern 2: op1_pos={}, op2_pos={}", op1_pos, op2_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, false, false, "a");
                constraint.b_constraint = self.constraint_for_digits_start_end(op1_pos, op2_pos, false, false, "b");
                constraint.num_ops = 2..=2;
            },
            (Some(op1_pos), None, Some(equal_pos)) if op1_pos < 3 && (equal_pos - op1_pos) <= 3 => {
                // (equal_pos - p1_pos) < 3, must be just one op
                // println!("Pattern 3b: op1_pos={}, equal_pos={}", op1_pos, equal_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, false, false, "a");
                constraint.b_constraint = self.constraint_for_digits_start_end(op1_pos, equal_pos, false, false, "b");
                constraint.c_constraint = self.constraint_for_digits_start_end(equal_pos, NERDLE_CHARACTERS as usize, false, true, "c");
                constraint.num_ops = 1..=1;
            },
            (Some(op1_pos), _, Some(equal_pos)) if op1_pos < 3 => {
                // op1_pos < 3, we know there is not another operator before op1_pos
                // println!("Pattern 3: op1_pos={}, equal_pos={}", op1_pos, equal_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, false, false, "a");
                constraint.b_constraint = self.constraint_for_digits_start_end(op1_pos, equal_pos, true, false, "b");
                // constraint.b2_constraint = self.constraint_for_digits_or_less(op_equal_pos - op1_pos - 1, false, "b2");
                constraint.c_constraint = self.constraint_for_digits_start_end(equal_pos, NERDLE_CHARACTERS as usize, false, true, "c");
            },
            (Some(op1_pos), None, Some(equal_pos)) => {
                // op1_pos >= 3, there may or may not be another operator before op1_pos
                // max digits for b and b2 is 2, possibilities are:
                //    v Position 3
                //     v Position 4 - Can't be after this, not enough room for =
                // a+b+B=cc
                // a+bb+B=c
                // aaa-bb=c
                // a+b+BB=c
                // println!("Pattern 3a: op1_pos={}, equal_pos={}", op1_pos, equal_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, true, false, "a");
                constraint.b_constraint = self.constraint_for_digits(2, None, true, false, "b");
                constraint.b2_constraint = self.constraint_for_digits(2, None, true, false, "b2");
                constraint.c_constraint = self.constraint_for_digits_start_end(equal_pos, NERDLE_CHARACTERS as usize, false, true, "c");
            },
            (Some(op1_pos), _, _) if op1_pos < 3 => {
                // println!("Pattern 4: op1_pos={}", op1_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, false, false, "a");
                constraint.b_constraint = self.constraint_for_digits(max_digits, None, true, false, "b");
                constraint.b2_constraint = self.constraint_for_digits(max_digits, None, true, false, "b2");
                constraint.c_constraint = self.constraint_for_digits(max_digits, None, true, true, "c");
            },
            // TODO: Lots more combinations
            _ => {
                // println!("Pattern 99");
                constraint.a_constraint = self.constraint_for_digits(max_digits, Some(0), true, false, "a");
                constraint.b_constraint = self.constraint_for_digits(max_digits, None, true, false, "b");
                constraint.b2_constraint = self.constraint_for_digits(max_digits, None, true, false, "b2");
                constraint.c_constraint = self.constraint_for_digits(max_digits, None, true, true, "c");
            }
        }

        constraint
    }

    pub fn take_guess(&self) -> Result<Equation, NoMatchFound> {
        let constraint = self.constraint();
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

            let char_info = data.char_info.entry(guess_ch).or_insert(NerdleCharInfo::new());
            (*char_info).positions[i] = new_char_pos_info;

            // Special handling for equal sign and operators
            match guess_ch as char {
                '=' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        data.equal_pos = Some(i);
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
                            '+' | '-' | '*' | '/' => true,
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

    fn constraint_for_digits_start_end(&self, start: usize, end: usize, min: bool, allow_zero: bool, name: &str) -> ExpressionNumberConstraint {
        // println!("constraint_for_digits_start_end(&self, {}, {}, {}, {})", &start, &end, &min, &name);
        let (start, digits) = if start == 0 {
            (0, end)
        } else {
            (start + 1, end - start - 1)
        };

        self.constraint_for_digits(digits, Some(start), min, allow_zero, name)
    }

    fn constraint_for_digits(&self, digits: usize, start: Option<usize>, min: bool, allow_zero: bool, name: &str) -> ExpressionNumberConstraint {
        // println!("Finding constraints for {}", &name);
        let range = if min {
            range_for_digits_or_less(digits, allow_zero)
        } else {
            range_for_digits(digits, allow_zero)
        };
        let regex = match start {
            Some(start) => self.regex_for_digits_at(start, digits, min),
            None => self.regex_for_digits_anywhere(digits, min),
        };
        let description = format!("{} has {} {} digits range {}..={} regex /{}/",
            &name,
            if min {
                &"up to"
            } else {
                &"exactly"
            },
            &digits,
            &range.start(), &range.end(),
            &regex.as_str());
        let accept = Rc::new(move |n: &ExpressionNumber| regex.is_match(&n.to_string()));
        ExpressionNumberConstraint {
            range,
            description,
            accept,
            ..Default::default()
        }
    }

    fn regex_for_digits_at(&self, start: usize, digits: usize, min: bool) -> Regex {
        // println!("regex_for_digits_at(&self, {}, {}, {})", &start, &digits, &min);
        let mut regex = String::new();
        regex.push_str("(?-u)^");
        for pos in start..(start+digits) {
            regex.push_str("[");
            for byte in self.possibilities_for_pos(pos).iter() {
                let chr = *byte as char;
                match chr {
                    '0'..='9' => regex.push(chr),
                    _ => { }
                }
            }
            regex.push_str("]");
            if min {
                regex.push_str("?");
            }
        }
        regex.push_str("$");

        // TODO: Better error handling?
        Regex::new(&regex).unwrap()
    }

    // TODO: Lots of duplication from above
    fn regex_for_digits_anywhere(&self, digits: usize, min: bool) -> Regex {
        let data = self.data.borrow();
        let mut regex = String::new();
        regex.push_str("(?-u)^[");

        for chr in '0'..='9' {
            let byte = chr as u8;
            match data.char_info.get(&byte) {
                None => {
                    regex.push(chr);
                },
                Some(info) if info.max_count > 0 => {
                    regex.push(chr);
                },
                _ => { }
            }
        }

        regex.push_str("]{");
        if min {
            regex.push_str(&format!("1,{}", digits));
        } else {
            regex.push_str(&format!("{}", digits));
        }
        regex.push_str("}$");

        // TODO: Better error handling?
        Regex::new(&regex).unwrap()
    }

}

impl fmt::Display for NerdleSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();

        write!(f, "Equal sign at: {}\n", data.equal_pos.map(|x| x.to_string()).unwrap_or("?".to_string()))?;

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

        Ok(())
    }
}

fn range_for_digits(digits: usize, allow_zero: bool) -> RangeInclusive<i32> {
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

fn range_for_digits_or_less(digits: usize, allow_zero: bool) -> RangeInclusive<i32> {
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
