use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::cmp::{min, max};
use regex::Regex;

use crate::strategy::Strategy;
use crate::eq::Equation;
use crate::nerdle::{NerdleResult, NerdleError, NERDLE_CHARACTERS, NERDLE_VALID_CHAR_BYTES, NERDLE_OPERAND_MAX_DIGITS, NERDLE_MAX_OPS};
use crate::eqgen::{eqgen_constrained};
use crate::constraint::{EquationConstraint, ExpressionNumberConstraint, NoMatchFound, range_for_digits, range_for_digits_or_less};
use crate::expr::{ExpressionNumber};
use crate::nerdledata::{NerdleData, NerdleCharInfo, NerdleIsChar};

const OPERATOR_STR: &str = "-+*/";

pub struct NerdleSolver {
    data: Rc<RefCell<NerdleData>>,
}

impl Strategy for NerdleSolver {
    fn take_guess(&self) -> Result<Equation, NoMatchFound> {
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

    fn update(&mut self, guess: &Equation, result: &NerdleResult) {
        let mut data = self.data.borrow_mut();
        data.update(guess, result);
    }

    fn print_hint(&self) {
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

    fn answer_ok(&self, eq: &Equation) -> Result<(), NerdleError> {
        self.data.borrow().eq_matches(eq)?;
        match self.constraint().accept(eq) {
            Err(err) => return Err(NerdleError { message: format!("Constraint {} failed: {}", self.constraint(), err)}),
            Ok(()) => { }
        }

        Ok(())
    }
}

impl NerdleSolver {
    pub fn new() -> NerdleSolver {
        NerdleSolver {
            data: Rc::new(RefCell::new(NerdleData::default())),
        }
    }

    pub fn constraint(&self) -> EquationConstraint {
        let mut constraint = EquationConstraint {
            accept: {
                let data: Rc<RefCell<NerdleData>> = self.data.clone();
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
            accept_description: self.data.borrow().describe_counts(),
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
        constraint.num_ops = max(min_ops, 1)..=min(max_ops, NERDLE_MAX_OPS);

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

        let is_op_at = |pos: usize| -> bool {
            data.possibilities_for_pos(pos)
                .iter()
                .all(|ch| NerdleSolver::is_op_char(*ch as char))
        };

        let op1_pos_opt = (0..NERDLE_CHARACTERS as usize).find(|i| is_op_at(*i as usize));
        let op2_pos_opt = match op1_pos_opt {
            Some(op1_pos) => ((op1_pos+1)..NERDLE_CHARACTERS as usize).find(|i| is_op_at(*i as usize)),
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
            (Some(op1_pos), _, _) => {
                let a_must_be_before_op1 = op1_pos < 3;
                // println!("Pattern 4: op1_pos={}, a_must_be_before_op1={}", op1_pos, a_must_be_before_op1);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, op1_pos, !a_must_be_before_op1, false, "a");
                constraint.b_constraint = self.constraint_for_digits(max_digits, None, true, false, "b");
                constraint.b2_constraint = self.constraint_for_digits(max_digits, None, true, false, "b2");
                constraint.c_constraint = self.constraint_for_digits(max_digits, None, true, true, "c");
            },
            (_, _, Some(equal_pos)) => {
                // println!("Pattern 90: equal_pos = {}", equal_pos);
                constraint.a_constraint = self.constraint_for_digits_start_end(0, min(equal_pos, max_digits), true, false, "a");
                constraint.b_constraint = self.constraint_for_digits(max_digits, None, true, false, "b");
                constraint.b2_constraint = self.constraint_for_digits(max_digits, None, true, false, "b2");
                constraint.c_constraint = self.constraint_for_digits_start_end(equal_pos, NERDLE_CHARACTERS as usize, false, true, "c");
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

    fn is_op_char(ch: char) -> bool {
        match ch {
            '+' | '-' | '/' | '*' => true,
            _ => false
        }
    }

    fn possibilities_for_pos(&self, pos: usize) -> HashSet<u8> {
        self.data.borrow().possibilities_for_pos(pos)
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
        let accept = Rc::new(move |n: &ExpressionNumber| {
            // println!("Checking {} against regex {}", &n, regex);
            regex.is_match(&n.to_string())
        });
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
            if min && pos != start {
                regex.push_str("(?:|");
            }
            regex.push_str("[");
            for byte in self.possibilities_for_pos(pos).iter() {
                let chr = *byte as char;
                match chr {
                    '0'..='9' => regex.push(chr),
                    _ => { }
                }
            }
            regex.push_str("]");
        }

        if min {
            for _pos in (start+1)..(start+digits) {
                regex.push_str(")");
            }
        }
        regex.push_str("$");

        // TODO: Better error handling?
        Regex::new(&regex).unwrap()
    }

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

        for ch in NERDLE_VALID_CHAR_BYTES.iter() {
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

#[cfg(test)]
use std::str::FromStr;

#[test]
fn regression_test_1() {
    let mut solver = NerdleSolver::new();
    solver.update(&Equation::from_str("104-9=95").unwrap(), &NerdleResult::from_str("--Y--G-Y").unwrap());
    solver.update(&Equation::from_str("385/5=77").unwrap(), &NerdleResult::from_str("--Y--GYY").unwrap());
    let constraint = solver.constraint();
    println!("Eq Constraint: {}", constraint);
    assert!(solver.constraint().accept(&Equation::from_str("42+24=66").unwrap()).is_err());
}

#[test]
fn regex_for_digits_test_1() {
    let mut solver = NerdleSolver::new();
    solver.update(&Equation::from_str("62+28=90").unwrap(), &NerdleResult::from_str("YG-YYY--").unwrap());
    let regex = solver.regex_for_digits_at(0, 4, true);
    println!("Regex: {}", regex);
    assert!(!regex.is_match("23"));
    assert!(!regex.is_match("6"));
    assert!(!regex.is_match("41"));

    assert!(regex.is_match("3"));
    assert!(regex.is_match("32"));
    assert!(regex.is_match("321"));
    assert!(regex.is_match("3217"));
}
