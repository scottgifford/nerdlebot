use std::fmt;
use std::collections::{HashMap};

use crate::eq::Equation;
use crate::nerdle::{NerdleError, NERDLE_CHARACTERS};

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
