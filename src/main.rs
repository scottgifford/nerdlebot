use std::io;

mod expr;
use crate::expr::parse_expression;

fn main() {
    println!("Enter an Expression to parse");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("You inputed: {}", input);

    let ex = parse_expression(&input)
        .expect("Failed to parse expression");
    println!("Expression: {}", ex.to_string());

    let res = ex.calculate()
        .expect("Failed to calculate expression");
    println!("Calculation: {}", res.to_string());
}
