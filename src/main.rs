use std::str::FromStr;
use std::io;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::{Instant};
use std::panic;

use colored::*;

mod eq;
mod expr;
mod eqgen;
mod constraint;
mod nerdle;
mod strategy;
mod nerdsolver;
mod nerdledata;
mod util;

use crate::eq::Equation;
use crate::expr::Expression;
use crate::eqgen::eqgen;
use crate::strategy::{Strategy, StrategyEnum};
use crate::nerdle::{NerdleResult, NERDLE_CHARACTERS};

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

fn prettylen<E>(len: Result<usize, E>) -> String
    where E: fmt::Display
{
    match len {
        Ok(len) => format!("{} ({})", "-".repeat(len), len),
        Err(err) => format!("Invalid length: {}", err)
    }
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
            let mut solver = StrategyEnum::by_name("first_possible")
                .expect("Failed to find named strategy");
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
                    match solver.answer_ok(&guess) {
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
            let count = std::env::args().nth(2).map(|x| i32::from_str(&x).expect("Invalid number of games")).unwrap_or(1);
            let mut wins = 0;
            let mut losses = 0;
            let mut win_turn_hist = [0; nerdle::NERDLE_TURNS as usize];

            for i in 0..count {
                println!("=== Playing game {} / {}", i, count);
                let start_time = Instant::now();

                let result = panic::catch_unwind(|| {
                    let mut solver = StrategyEnum::by_name("first_possible")
                        .expect("Failed to find named strategy");
                    let answer = eqgen().expect("Failed to generate equation");
                    println!("Answer: {}", &answer);

                    let mut turn: u32 = 0;
                    loop {
                        turn += 1;
                        if turn > nerdle::NERDLE_TURNS {
                            break GameResult::Loss();
                        }

                        let mut guess;
                        let res;
                        loop {
                            guess = skip_fail!(solver.take_guess(), "No valid guess was generating, trying again");
                            println!("Turn {}  Guess: {}", turn, guess);
                            match solver.answer_ok(&guess) {
                                Ok(()) => { },
                                Err(why) => println!("Equation is impossible because {}", why)
                            }
                            res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed, trying again");
                            break;
                        }

                        println!("Turn {} Result: {}", turn, res);
                        pretty_print_result(&guess.to_string(), &res);
                        if res.won() {
                            break GameResult::Win(turn);
                        }
                        solver.update(&guess, &res);
                        solver.print_hint();
                    }
                });
                match result {
                    Ok(GameResult::Win(turn)) => {
                        println!("I won in {} turns!", turn);
                        wins += 1;
                        win_turn_hist[turn as usize - 1] += 1;
                    },
                    Ok(GameResult::Loss()) => {
                        losses += 1;
                        println!("I lost");
                    },
                    Err(err) => println!("Game failed: {:?}", err)
                }
                println!("Game {} completed in {:?}", i, start_time.elapsed());
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
            let result = panic::catch_unwind(|| {
                let answer = std::env::args().nth(2)
                    .expect("no expr given");
                let answer = Equation::from_str(&answer)
                    .expect("Failed to parse equation");

                match answer.len() {
                    Ok(len) => if len != NERDLE_CHARACTERS as usize {
                        return Err(CommandLineError { message: format!("Equation '{}' is wrong length ({} chars != {})", answer, len, NERDLE_CHARACTERS) } );
                    },
                    Err(err) => return Err(CommandLineError { message: format!("Equation '{}' has invalid length: {})", answer, err) } )
                }
                if !answer.computes().unwrap_or(false) {
                    return Err(CommandLineError { message: format!("Equation unexpectedly did not compute: {}", answer) } );
                }
        
                let mut solver = StrategyEnum::by_name("first_possible")
                    .expect("Failed to find named strategy");

                println!("Answer: {}", &answer);

                let mut won = false;
                for turn in 1..=nerdle::NERDLE_TURNS {
                    let mut guess;
                    let res;
                    loop {
                        match solver.answer_ok(&answer) {
                            Err(err) =>  return Err(CommandLineError { message: format!("Solver {} rejects answer: {}", solver, err) } ),
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
                        match solver.answer_ok(&guess) {
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
            });
            match result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(CommandLineError { message: format!("Failed: {:?}", err) }),
                Err(err) => Err(CommandLineError { message: format!("Panicked: {:?}", err) })
            }
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

                let mut solver = StrategyEnum::by_name("first_possible")
                    .expect("Failed to find named strategy");
                let answer = Equation::from_str(&line)
                    .expect("Failed to parse equation");

                match answer.len() {
                    Ok(len) => if len != NERDLE_CHARACTERS as usize {
                        return Err(CommandLineError { message: format!("Equation '{}' is wrong length ({} chars != {})", answer, len, NERDLE_CHARACTERS) } );
                    },
                    Err(err) => return Err(CommandLineError { message: format!("Equation '{}' had invalid length: {}", answer, err)})
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
                        match solver.answer_ok(&answer) {
                            Err(err) =>  return Err(CommandLineError { message: format!("Solver {} rejects answer: {}", solver, err) } ),
                            Ok(()) => { }
                        }
                        guess = skip_fail!(solver.take_guess(), "No valid guess was generating, trying again");
                        println!("Turn {}  Guess: {}", turn, guess);
                        match solver.answer_ok(&guess) {
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

        // TODO: Lots of duplicated code
        Some("interactive") => {        
            let mut solver = StrategyEnum::by_name("first_possible")
                .expect("Failed to find named strategy");


            let mut won = false;
            for turn in 1..=nerdle::NERDLE_TURNS {
                // No idea why res should be mut but not guess?..
                let guess;
                let mut res;
                loop {
                    guess = match std::env::args().nth(1 + turn as usize) {
                        Some(guess) => match Equation::from_str(&guess) {
                            Ok(guess) => guess,
                            Err(err) => return Err(CommandLineError { message: format!("Invalid guess equation in command-line arg {} '{}': {}", 2+turn, guess, err) } )
                        }
                        None => skip_fail!(solver.take_guess(), "No valid guess was generating, trying again")
                    };
                    println!("Turn {}  Guess: {}", turn, &guess);
                    match solver.answer_ok(&guess) {
                        Ok(()) => { },
                        Err(why) => println!("Equation is impossible because {}", why)
                    }
                    res = loop {
                        println!("Turn {} Enter Result:", turn);
                        let mut input = String::new();
                        skip_fail!(io::stdin().read_line(&mut input), "Read error, try again");
                        let cleanput = input.trim();
                        res = skip_fail!(NerdleResult::from_str(cleanput), "Invalid entry");
                        break res;
                    };
                    break;
                }

                println!("Turn {} Result: {}", turn, &res);
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
        Some(oops) => Err(CommandLineError { message: format!("Unrecognized command '{}'", oops) } ),

        None => Err(CommandLineError { message: format!("Missing command line flag") } ),
    }
}

enum GameResult {
    Win(u32),
    Loss(),
}
