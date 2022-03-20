use crate::eq::Equation;
use std::fmt;
use std::collections::HashMap;
use crate::nerdle::{NerdleResult, NerdlePositionResult, NerdleError, NERDLE_CHARACTERS};
use std::cmp::{max};

const VALID_CHAR_STR: &str = "1234567890-+*/=";

#[derive(Clone, Debug)]
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
            positions: [
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
                NerdleIsChar::Maybe,
            ]
        }
    }
}

impl fmt::Display for NerdleCharInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CharInfo: min={}, max={}", self.min_count, self.max_count)
    }
}

// enum ParseState {
//     InA,
//     InB,
//     InC,
// }

pub struct NerdleSolver {
    pub char_info: HashMap<u8, NerdleCharInfo>,
    pub positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize],
    pub equal_pos: Option<usize>,
    pub op: Option<u8>,
    pub op_pos: Option<usize>,
}

impl NerdleSolver {
    pub fn new() -> NerdleSolver {
        NerdleSolver {
            char_info: HashMap::new(),
            positions: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            equal_pos: None,
            op_pos: None,
            op: None,
        }
    }

    // pub fn take_guess(&self) -> Equation {

    // }

    pub fn update(&mut self, guess: &Equation, result: &NerdleResult) {
        // let mut state = ParseState::InA;
        let guess_str = guess.to_string();
        let guess = guess_str.as_bytes();

        // First count the total letters (it is hard to take GREENs into account as we go)
        let mut char_occ_count: HashMap<u8, u32> = HashMap::new();
        let mut found_max: HashMap<u8, bool> = HashMap::new();

        for i in 0..NERDLE_CHARACTERS as usize {
            let guess_ch = guess[i];
            let counter = char_occ_count.entry(guess_ch).or_insert(0);
            let char_info = self.char_info.entry(guess_ch).or_insert(NerdleCharInfo::new());
            // First handle general-purpose logic
            match result.positions[i] {
                NerdlePositionResult::Green => {
                    self.positions[i].insert(guess_ch, true);
                    (*char_info).positions[i] = NerdleIsChar::Definitely;
                    *counter += 1
                },
                NerdlePositionResult::Yellow => {
                    self.positions[i].insert(guess_ch, false);
                    (*char_info).positions[i] = NerdleIsChar::DefinitelyNot;
                    *counter += 1;
                },
                NerdlePositionResult::Gray => {
                    self.positions[i].insert(guess_ch, false);
                    (*char_info).positions[i] = NerdleIsChar::DefinitelyNot;
                    found_max.insert(guess_ch, true);
                }
            }

            // Special handling for equal sign and operators
            match guess_ch as char {
                '=' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        self.equal_pos = Some(i);
                    },
                    _ => { }
                },
                '+' | '-' | '*' | '/' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        self.op_pos = Some(i);
                        self.op = Some(guess_ch);
                    },
                    NerdlePositionResult::Yellow => {
                        self.op = Some(guess_ch);
                    },
                    _ => { }
                },
                _ => { }
            }
        }

        for (ch, count) in char_occ_count.iter() {
            let mut ent = self.char_info.entry(*ch).or_insert(NerdleCharInfo::new());
            (*ent).min_count = max((*ent).min_count, *count);
            if found_max.contains_key(ch) {
                (*ent).max_count = *count;
            }
        }
    }

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

    pub fn print_hint(&self) {
        print!("Equal sign ");
        match self.equal_pos {
            Some(x) => print!("at {}", x),
            None => {
                print!("not at ");
                match self.char_info.get(&('=' as u8)) {
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

        print!("Operator is {} at {} and is not: ", self.op.map(|x| x as char).unwrap_or('?'), self.op_pos.map(|x| x.to_string()).unwrap_or("?".to_string()));
        for (key, ent) in self.char_info.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.max_count < 1 { print!("{} ", *key as char); },
                _ => { }
            }
        }
        println!("");

        let mut known_pos: HashMap<u8, u32> = HashMap::new();
        for (key, val) in self.char_info.iter() {
            known_pos.insert(*key, val.positions.iter().fold(0, |sum, status| sum + match status {
                NerdleIsChar::Definitely => 1,
                _ => 0
            }));
        }

        for pos in 0..(NERDLE_CHARACTERS as usize) {
            print!("Position {} ", pos);
            match self.positions[pos].iter().find_map(|(key, value)| if *value { Some(key) } else { None }) {
                Some(known) => print!(" is {}", *known as char),
                None => {
                    print!(" could be: ");
                    for ch in VALID_CHAR_STR.as_bytes().iter() {
                        if match *ch as char {
                            '=' => self.equal_pos == None,
                            '+' | '-' | '*' | '/' => self.op_pos == None,
                            _ => match self.char_info.get(ch) {
                                Some(info) => match info.positions[pos] {
                                    NerdleIsChar::Maybe => {
                                        match self.char_info.get(ch) {
                                            Some(info) => {
                                                let known = known_pos.get(ch).unwrap_or(&0);
                                                // println!("  ch {} pos {} known {} info {}", *ch as char, pos, known, info);
                                                known < &info.max_count
                                            },
                                            None => true,
                                        }
                                    },
                                    _ => false
                                },
                                None => true
                            }
                        } {
                            print!("{} ", *ch as char);
                        }
                    }        
                }
            }
            print!("\n");
        }
    }
}

impl fmt::Display for NerdleSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Equal sign at: {}\n", self.equal_pos.map(|x| x.to_string()).unwrap_or("?".to_string()))?;

        write!(f, "Operator is {} at {} and is not: ", self.op.map(|x| x as char).unwrap_or('?'), self.op_pos.map(|x| x.to_string()).unwrap_or("?".to_string()))?;
        for (key, ent) in self.char_info.iter() {
            match *key as char {
                '+' | '-' | '/' | '*' => if ent.max_count < 1 { write!(f, "{} ", *key as char)?; },
                _ => { }
            }
        }
        write!(f, "\n")?;

        for ch in VALID_CHAR_STR.as_bytes().iter() {
            write!(f, "Character {}: {}\n", *ch as char, self.char_info.get(ch).unwrap_or(&NerdleCharInfo::new()))?;
        }

        for pos in 0..(NERDLE_CHARACTERS as usize) {
            write!(f, "Position {} is not: ", pos)?;
            for (key, ent) in self.positions[pos].iter() {
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
