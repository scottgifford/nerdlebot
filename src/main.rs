use std::str::FromStr;
use std::io;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::{Instant};

use colored::*;

mod eq;
mod expr;
mod eqgen;
mod constraint;
mod nerdle;
mod nerdsolver;
mod util;

use crate::eq::Equation;
use crate::expr::Expression;
use crate::eqgen::eqgen;
use crate::nerdsolver::NerdleSolver;
use crate::nerdle::{NERDLE_CHARACTERS};

#[derive(Clone)]
pub struct CommandLineError {
    message: String,
}

impl fmt::Display for CommandLineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CommandLineError
    : {}", self.message)
    }
}

impl fmt::Debug for CommandLineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "CommandLineError
    : {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}

fn prettylen(len: usize) -> String {
    format!("{} ({})", "-".repeat(len), len)
}


fn pretty_print_result(guess: &str, res: &nerdle::NerdleResult) {
    let guess = guess.as_bytes();
    for pos in 0..(nerdle::NERDLE_CHARACTERS as usize) {
        let chs = String::from(guess[pos] as char);
        let color_chs = match res.positions[pos] {
            nerdle::NerdlePositionResult::Yellow => chs.black().on_yellow(),
            nerdle::NerdlePositionResult::Green => chs.black().on_green(),
            nerdle::NerdlePositionResult::Gray => chs.black().on_white(),
        };
        print!("{}", color_chs);
    }
    println!("");
}

macro_rules! skip_fail {
    ($res:expr, $message:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("{} (Error {})", $message, e);
                continue;
            }
        }
    };
}

fn main() -> Result<(), CommandLineError> {
    let cmd = std::env::args().nth(1);
    match cmd.as_deref() {
        Some("expr") => {
            let expr = std::env::args().nth(2)
                .expect("no expr given");
            let expr = Expression::from_str(&expr)
                .expect("Failed to parse expression");
            println!("Expression: {}", &expr);
            println!("    Length: {}", prettylen(expr.len()));
            let res = expr.calculate()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },

        Some("eq") => {
            let eq = std::env::args().nth(2)
                .expect("no expr given");
            let eq = Equation::from_str(&eq)
                .expect("Failed to parse equation");
            println!("Equation: {}", &eq);
            println!("  Length: {}", prettylen(eq.len()));
            let res = eq.computes()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },

        Some("gen") => {
            let eq = eqgen()
                .expect("Failed to generate equation");
            println!("Equation: {}", &eq);
            println!("  Length: {}", prettylen(eq.len()));
            let res = eq.computes()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },

        Some("gen3") => {
            let eq = eqgen::eqgen()
                .expect("Failed to generate equation");
            println!("Equation: {}", &eq);
            println!("  Length: {}", prettylen(eq.len()));
            let res = eq.computes()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },

        Some("eval") => {
            let answer = std::env::args().nth(2)
                .expect("no expr given in arg 2");
            let answer = Equation::from_str(&answer)
                .expect("Failed to parse equation in arg 2");
            println!("Answer: {}", &answer);

            let guess = std::env::args().nth(3)
                .expect("no expr given in arg 3");
            let guess = Equation::from_str(&guess)
                .expect("Failed to parse equation in arg 3");
            println!(" Guess: {}", &guess);

            let res = nerdle::nerdle(&guess, &answer)
                .expect("Failed to nerdle");

            println!("Result: {}", res);
            Ok(())
        },

        Some("play") => {
            let answer = eqgen()
                .expect("Failed to generate equation");
            let mut won = false;

            for turn in 1..=nerdle::NERDLE_TURNS {
                let mut guess;
                let res;
                loop {
                    println!("Turn {} Enter Guess:", turn);
                    let mut input = String::new();
                    skip_fail!(io::stdin().read_line(&mut input), "Read error, try again");
                    let cleanput = input.trim_end();
                    println!("Read: {}", cleanput);
                    guess = skip_fail!(Equation::from_str(&cleanput), "Invalid equation, try again");
                    res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed try again");
                    break;
                }

                println!("Turn {} Result: {}", turn, res);
                pretty_print_result(&guess.to_string(), &res);
                if res.won() {
                    won = true;
                    println!("You won in {} turns!", turn);
                    break;
                }
            }
            println!("Answer: {}", &answer);
            if !won {
                println!("You lost");
            }
            Ok(())
        },

        // TODO: Lots of copypasta from "play"
        Some("play_assist") => {
            let mut solver = NerdleSolver::new();
            let answer = eqgen()
                .expect("Failed to generate equation");
            let mut won = false;

            for turn in 1..=nerdle::NERDLE_TURNS {
                let mut guess;
                let res;
                loop {
                    match solver.take_guess() {
                        Ok(bot_guess) => println!("Bot guess: {}", bot_guess),
                        Err(err) => println!("Bot could not come up with guess: {}", err),
                    }
                    println!("Turn {} Enter Guess:", turn);
                    let mut input = String::new();
                    skip_fail!(io::stdin().read_line(&mut input), "Read error, try again");
                    let cleanput = input.trim_end();
                    println!("Read: {}", cleanput);
                    guess = skip_fail!(Equation::from_str(&cleanput), "Invalid equation, try again");
                    match solver.eq_matches(&guess) {
                        Ok(()) => { },
                        Err(why) => println!("Equation is impossible because {}", why)
                    }
                    res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed try again");
                    break;
                }

                println!("Turn {} Result: {}", turn, res);
                pretty_print_result(&guess.to_string(), &res);
                if res.won() {
                    won = true;
                    println!("You won in {} turns!", turn);
                    break;
                }
                solver.update(&guess, &res);
                solver.print_hint();
            }
            println!("Answer: {}", &answer);
            if !won {
                println!("You lost");
            }
            Ok(())
        },

        // TODO: Lots of duplicated code
        Some("solve_random") => {
            let count = std::env::args().nth(2).map(|x| i32::from_str(&x).expect("Invalid number of turns")).unwrap_or(1);
            let mut wins = 0;
            let mut losses = 0;
            let mut win_turn_hist = [0; nerdle::NERDLE_TURNS as usize];

            for i in 0..count {
                println!("=== Playing game {} / {}", i, count);

                let mut solver = NerdleSolver::new();
                let answer = skip_fail!(eqgen(), "Failed to generate equation");
                println!("Answer: {}", &answer);

                let mut won = false;
                for turn in 1..=nerdle::NERDLE_TURNS {
                    let mut guess;
                    let res;
                    loop {
                        guess = skip_fail!(solver.take_guess(), "No valid guess was generating, trying again");
                        println!("Turn {}  Guess: {}", turn, guess);
                        match solver.eq_matches(&guess) {
                            Ok(()) => { },
                            Err(why) => println!("Equation is impossible because {}", why)
                        }
                        res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed, trying again");
                        break;
                    }

                    println!("Turn {} Result: {}", turn, res);
                    pretty_print_result(&guess.to_string(), &res);
                    if res.won() {
                        won = true;
                        println!("I won in {} turns!", turn);
                        wins += 1;
                        win_turn_hist[turn as usize - 1] += 1;
                        break;
                    }
                    solver.update(&guess, &res);
                    solver.print_hint();
                }
                if !won {
                    losses += 1;
                    println!("I lost");
                }
            }
            println!("Played {} games", count);
            println!("       {} failures", count - wins - losses);
            println!("       {} wins", wins);
            println!("       {} losses", losses);
            println!("       {} win rate",(wins as f64) / (count as f64));
            println!("");
            for i in 0..nerdle::NERDLE_TURNS as usize {
                println!(" Turn {} wins {}", i+1, win_turn_hist[i]);
            }
            Ok(())
        },

        // TODO: Lots of duplicated code
        Some("solve") => {
            let answer = std::env::args().nth(2)
                .expect("no expr given");
            let answer = Equation::from_str(&answer)
                .expect("Failed to parse equation");

            if answer.len() != NERDLE_CHARACTERS as usize {
                return Err(CommandLineError { message: format!("Equation '{}' is wrong length ({} chars != {})", answer, answer.len(), NERDLE_CHARACTERS) } );
            }
            if !answer.computes().unwrap_or(false) {
                return Err(CommandLineError { message: format!("Equation unexpectedly did not compute: {}", answer) } );
            }
        
            let mut solver = NerdleSolver::new();
            println!("Answer: {}", &answer);

            let mut won = false;
            for turn in 1..=nerdle::NERDLE_TURNS {
                let mut guess;
                let res;
                loop {
                    let constraint = solver.constraint();
                    match constraint.accept(&answer) {
                        Err(err) =>  return Err(CommandLineError { message: format!("Solver constraint {} rejects answer: {}", constraint, err) } ),
                        Ok(()) => { }
                    }
                    guess = match std::env::args().nth(2 + turn as usize) {
                        Some(guess) => match Equation::from_str(&guess) {
                            Ok(guess) => guess,
                            Err(err) => return Err(CommandLineError { message: format!("Invalid guess equation in command-line arg {} '{}': {}", 2+turn, guess, err) } )
                        }
                        None => skip_fail!(solver.take_guess(), "No valid guess was generating, trying again")
                    };
                    println!("Turn {}  Guess: {}", turn, guess);
                    match solver.eq_matches(&guess) {
                        Ok(()) => { },
                        Err(why) => println!("Equation is impossible because {}", why)
                    }
                    res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed, trying again");
                    break;
                }

                println!("Turn {} Result: {}", turn, res);
                pretty_print_result(&guess.to_string(), &res);
                if res.won() {
                    won = true;
                    println!("I won in {} turns!", turn);
                    break;
                }
                solver.update(&guess, &res);
                solver.print_hint();
            }
            if !won {
                println!("I lost");
            }

            Ok(())
        },

        // TODO: Lots of duplicated code
        Some("solve_file") => {
            let file_name = std::env::args().nth(2)
                .expect("Expected file name in arg 2");
            let file = File::open(&file_name)
                .expect(&format!("Error opening file '{}'", &file_name));
            let buf_reader = BufReader::new(file);

            let mut wins = 0;
            let mut losses = 0;
            let mut win_turn_hist = [0; nerdle::NERDLE_TURNS as usize];

            let mut i = 0;

            for line in buf_reader.lines() {
                let line = line.expect(&format!("Error readline line from file '{}'", &file_name));
                let line = line.trim();

                match line.chars().next() {
                    None => continue, // Empty line (after removing whitespace)
                    Some('#') => continue, // Comment line
                    _ => { }
                };

                println!("=== Playing game {}", i);
                let start_time = Instant::now();

                let mut solver = NerdleSolver::new();
                let answer = Equation::from_str(&line)
                    .expect("Failed to parse equation");

                if answer.len() != NERDLE_CHARACTERS as usize {
                    return Err(CommandLineError { message: format!("Equation '{}' is wrong length ({} chars != {})", answer, answer.len(), NERDLE_CHARACTERS) } );
                }
                if !answer.computes().unwrap_or(false) {
                    return Err(CommandLineError { message: format!("Equation unexpectedly did not compute: {}", answer) } );
                }

                println!("Answer: {}", &answer);

                let mut won = false;
                for turn in 1..=nerdle::NERDLE_TURNS {
                    let mut guess;
                    let res;
                    loop {
                        let constraint = solver.constraint();
                        match constraint.accept(&answer) {
                            Err(err) =>  return Err(CommandLineError { message: format!("Solver constraint {} rejects answer: {}", constraint, err) } ),
                            Ok(()) => { }
                        }
                        guess = skip_fail!(solver.take_guess(), "No valid guess was generating, trying again");
                        println!("Turn {}  Guess: {}", turn, guess);
                        match solver.eq_matches(&guess) {
                            Ok(()) => { },
                            Err(why) => println!("Equation is impossible because {}", why)
                        }
                        res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed, trying again");
                        break;
                    }

                    println!("Turn {} Result: {}", turn, res);
                    pretty_print_result(&guess.to_string(), &res);
                    if res.won() {
                        won = true;
                        println!("I won in {} turns!", turn);
                        wins += 1;
                        win_turn_hist[turn as usize - 1] += 1;
                        break;
                    }
                    solver.update(&guess, &res);
                    solver.print_hint();
                }
                if !won {
                    losses += 1;
                    println!("I lost");
                }
                println!("Game {} completed in {:?}", i, start_time.elapsed());
                i += 1;
            }
            let count = i;
            println!("Played {} games", count);
            println!("       {} failures", count - wins - losses);
            println!("       {} wins", wins);
            println!("       {} losses", losses);
            println!("       {} win rate",(wins as f64) / (count as f64));
            println!("");
            for i in 0..nerdle::NERDLE_TURNS as usize {
                println!(" Turn {} wins {}", i+1, win_turn_hist[i]);
            }
            Ok(())
        },

        Some(oops) => Err(CommandLineError { message: format!("Unrecognized command '{}'", oops) } ),

        None => Err(CommandLineError { message: format!("Missing command line flag") } ),
    }
}
