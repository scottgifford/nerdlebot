use std::fmt;
use std::collections::{HashMap, HashSet};
use std::cmp::{max};

use crate::eq::Equation;
use crate::nerdle::{NerdleResult, NerdlePositionResult, NerdleError, NERDLE_CHARACTERS, NERDLE_VALID_CHAR_BYTES};

pub struct NerdleData {
    pub char_info: HashMap<u8, NerdleCharInfo>,
    pub positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize],
    pub equal_pos: Option<usize>,
}

impl Default for NerdleData {
    fn default() -> Self {
        let positions: [HashMap<u8, bool>; NERDLE_CHARACTERS as usize] = Default::default();
        NerdleData {
            char_info: HashMap::new(),
            positions,
            equal_pos: None,
        }
    }
}

impl NerdleData {
    // TODO: Switch to a better error type
    pub fn eq_matches(&self, eq: &Equation) -> Result<(), NerdleError> {
        let eq_str = eq.to_string();
        let eq_bytes = eq_str.as_bytes();

        // First check counts.  This is the unique thing that we do that contraints cannot.
        let mut char_counts = HashMap::new();
        for &ch in eq_bytes.iter() {
            let counter = char_counts.entry(ch).or_insert(0);
            *counter += 1;
        }

        for(ch, info) in self.char_info.iter() {
            let count = char_counts.get(&ch).unwrap_or(&0);
            if count < &info.min_count {
                return Err(NerdleError { message: format!("Not enough of character '{}' ({} < {})", *ch as char, count, info.min_count) })
            }
            if count > &info.max_count {
                return Err(NerdleError { message: format!("Too many of character '{}' ({} > {})", *ch as char, count, info.max_count) })
            }
        }

        // Check characters in positions
        for pos in 0..(NERDLE_CHARACTERS as usize) {
            let guess_ch = eq_bytes[pos];
            match (guess_ch, self.positions[pos].get(&guess_ch)) {
                (0..=9, Some(false)) => {
                    return Err(NerdleError { message: format!("Position {} cannot be {}", pos, guess_ch as char)})
                    // For numbers, this should have been caught by earlier checks.  Uncomment the below line to fail if not, for testing.
                    // panic!("Position {} cannot be {} in {}", pos, guess_ch as char, &eq),
                },
                (_, Some(false)) => return Err(NerdleError { message: format!("Position {} cannot be {}", pos, guess_ch as char)}),
                _ => { }
            }
        }

        Ok(())
    }

    pub fn describe_counts(&self) -> String {
        let mut description = "digit counts: ".to_string();

        for(ch, info) in self.char_info.iter() {
            description.push_str(&format!("{}[{}-{}], ", *ch as char, info.min_count, info.max_count));
        }

        // Remove the extra comma and space we added above
        description.pop();
        description.pop();

        description
    }

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
            let new_char_pos_info: NerdleIsChar;
            // First handle general-purpose logic
            match result.positions[i] {
                NerdlePositionResult::Green => {
                    self.positions[i].insert(guess_ch, true);
                    for ch in NERDLE_VALID_CHAR_BYTES.iter() {
                        if *ch != guess_ch {
                            self.positions[i].insert(*ch, false);
                        }
                    }
                    new_char_pos_info = NerdleIsChar::Definitely;
                    *counter += 1
                },
                NerdlePositionResult::Yellow => {
                    self.positions[i].insert(guess_ch, false);
                    new_char_pos_info = NerdleIsChar::DefinitelyNot;
                    *counter += 1;
                },
                NerdlePositionResult::Gray => {
                    self.positions[i].insert(guess_ch, false);
                    new_char_pos_info = NerdleIsChar::DefinitelyNot;
                    found_max.insert(guess_ch, true);
                }
            }

            let char_info = self.char_info.entry(guess_ch).or_insert(NerdleCharInfo::new());
            (*char_info).positions[i] = new_char_pos_info;

            // Special handling for equal sign and operators
            match guess_ch as char {
                '=' => match result.positions[i] {
                    NerdlePositionResult::Green => {
                        self.equal_pos = Some(i);
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

        // If we have eliminated all other possibilities, explicitly set remaining item to find, to simplify logic later
        for pos in 0..(NERDLE_CHARACTERS as usize) {
            let poss = self.possibilities_for_pos(pos);
            if poss.len() == 1 {
                let ch = poss.iter().next().unwrap();

                let mut char_info = self.char_info.entry(*ch).or_insert(NerdleCharInfo::new());
                (*char_info).min_count = max((*char_info).min_count, 1);
                (*char_info).positions[pos] = NerdleIsChar::Definitely;

                let pos_data = &mut self.positions[pos];
                pos_data.insert(*ch, true);
            }
        }
    }

    pub fn possibilities_for_pos(&self, pos: usize) -> HashSet<u8> {
        // TODO: Move to state or something?  Or maybe this should be done up update() to pre-calculate all this?
        let mut known_pos: HashMap<u8, u32> = HashMap::new();
        for (key, val) in self.char_info.iter() {
            known_pos.insert(*key, val.positions.iter().fold(0, |sum, status| sum + match status {
                NerdleIsChar::Definitely => 1,
                _ => 0
            }));
        }

        let mut ret = HashSet::new();
        match self.positions[pos].iter().find_map(|(key, value)| if *value { Some(key) } else { None }) {
            Some(known) => { ret.insert(*known); },
            None => {
                for ch in NERDLE_VALID_CHAR_BYTES.iter() {
                    let info: Option<&NerdleCharInfo> = self.char_info.get(&ch);
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
                            '=' => self.equal_pos == None,
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
}

#[derive(Clone, Debug)]
pub struct NerdleCharInfo {
    pub min_count: u32,
    pub max_count: u32,
    pub positions: [NerdleIsChar; NERDLE_CHARACTERS as usize]
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

#[derive(Clone, Copy, Debug)]
pub enum NerdleIsChar {
    Definitely,
    DefinitelyNot,
    Maybe,
}
