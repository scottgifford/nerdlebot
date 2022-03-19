use std::str::FromStr;
use std::io;
use std::fmt;

mod eq;
mod expr;
mod eqgen;
mod constraint;
mod nerdle;

use crate::eq::Equation;
use crate::expr::Expression;
use crate::eqgen::eqgen;

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

            for turn in 1..=nerdle::NERDLE_TURNS {
                let res;
                loop {
                    println!("Turn {} Enter Guess:", turn);
                    let mut input = String::new();
                    skip_fail!(io::stdin().read_line(&mut input), "Read error, try again");
                    let cleanput = input.trim_end();
                    println!("Read: {}", cleanput);
                    let guess = skip_fail!(Equation::from_str(&cleanput), "Invalid equation, try again");
                    res = skip_fail!(nerdle::nerdle(&guess, &answer), "Nerdling failed try again");
                    break;
                }
                // TODO: Stop if we have won!
                println!("Turn {} Result: {}", turn, res);
            }
            println!("Answer: {}", &answer);

            Ok(())
        },

        Some(oops) => Err(CommandLineError { message: format!("Unrecognized command '{}'", oops) } ),

        None => Err(CommandLineError { message: format!("Missing command line flag") } ),
    }
}
