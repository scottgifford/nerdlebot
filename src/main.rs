use std::io;
use std::str::FromStr;

mod eq;
mod expr;

use crate::eq::Equation;

fn main() {
    println!("Enter an Equation to parse");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("You inputed: {}", input);

    let eq = Equation::from_str(&input)
        .expect("Failed to parse equation");
    println!("Expression: {}", eq.to_string());

    let res = eq.computes()
        .expect("Failed to compute expression");
    println!("Equation Computes: {}", res.to_string());
}
