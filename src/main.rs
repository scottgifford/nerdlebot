use std::str::FromStr;
use std::fmt;

mod eq;
mod expr;
mod eqgen;

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


fn main() -> Result<(), CommandLineError> {
    let cmd = std::env::args().nth(1);
    match cmd.as_deref() {
        Some("expr") => {
            let expr = std::env::args().nth(2)
                .expect("no expr given");
            let expr = Expression::from_str(&expr)
                .expect("Failed to parse expression");
            println!("Expression: {}", &expr);
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
            let res = eq.computes()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },

        Some("gen") => {
            let eq = eqgen()
                .expect("Failed to generate equation");
            println!("Equation: {}", &eq);
            let res = eq.computes()
                .expect("Failed to compute expression");
            println!("Equation Computes: {}", res);
            Ok(())
        },


        Some(oops) => Err(CommandLineError { message: format!("Unrecognized command '{}'", oops) } ),

        None => Err(CommandLineError { message: format!("Missing command line flag") } ),
    }
}
