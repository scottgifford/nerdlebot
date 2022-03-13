use std::io;
use std::fmt;

pub struct InvalidExpressionError {
    message: String,
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidExpressionError: {}", self.message)
    }
}

pub struct Equation {
    expr: Expression,
    res: ExpressionNumber,
}

enum ExpressionCalculateState {
    FirstNumber,
    ExpectOperator,
    ExpectOperand,
}


impl Expression {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for part in &self.parts {
            s.push_str(&part.to_string());
            s.push_str(" ");
        }
        s.truncate(s.len()-1);
        return s;
    }

    fn calculate(&self) -> Result<ExpressionNumber, InvalidExpressionError> {
        // TODO: Does not correctly implement order of expressions
        let mut state = ExpressionCalculateState::FirstNumber;
        let mut cur: Option<ExpressionNumber> = None;
        let mut op: Option<&Box<dyn ExpressionOperator>> = None;

        for part in &self.parts {
            match part {
                ExpressionPart::Number(num) => {
                    match state {
                        ExpressionCalculateState::FirstNumber => {
                            cur = Some(num.clone());
                            state = ExpressionCalculateState::ExpectOperator;
                        },
                        ExpressionCalculateState::ExpectOperand => match op {
                                Some(op2) => match cur {
                                    Some(cur2) => {
                                        cur = Some(op2.operate(&cur2, num));
                                        state = ExpressionCalculateState::ExpectOperator;
                                    },
                                    None => return Err(InvalidExpressionError { message: String::from("Operator missing first operand") }),
                                }
                                None => return Err(InvalidExpressionError { message: String::from("Expected operator") }),
                        },
                        ExpressionCalculateState::ExpectOperator => return Err(InvalidExpressionError { message: String::from("Expected operator but got a number") }),
                    }
                },
                ExpressionPart::Operator(op2) => {
                    match state {
                        ExpressionCalculateState::ExpectOperator => {
                            op = Some(op2);
                            state = ExpressionCalculateState::ExpectOperand;
                        },
                        ExpressionCalculateState::FirstNumber | ExpressionCalculateState::ExpectOperand => return Err(InvalidExpressionError { message: String::from("Expected number but got an operator") }),
                    }
                },
            }
        }

        match cur {
            Some(ret) => return Ok(ret),
            None => return Err(InvalidExpressionError { message: String::from("No values found") }),
        }
    }
}

pub struct Expression {
    parts: Vec<ExpressionPart>,
}


pub enum ExpressionPart {
    Number(ExpressionNumber),
    Operator(Box<dyn ExpressionOperator>),
}

impl ToString for ExpressionPart {
    fn to_string(&self) -> String {
        match self {
            ExpressionPart::Number(num) => num.to_string(),
            ExpressionPart::Operator(op) => op.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ExpressionNumber {
    value: u32,
}

impl ToString for ExpressionNumber {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }
}

impl ExpressionNumber {
    // TODO: Inefficient
    fn len(&self) -> usize {
        return self.to_string().len();
    }
}

pub trait ExpressionOperator: ToString {
    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber;
    fn len(&self) -> usize;
}

pub struct ExpressionOperatorPlus {

}

impl ToString for ExpressionOperatorPlus {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "+".to_string();
    }
}

impl ExpressionOperator for ExpressionOperatorPlus {

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value + b.value,
        };
    }
}

pub struct ExpressionOperatorMinus {

}

impl ToString for ExpressionOperatorMinus {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "-".to_string();
    }
}

impl ExpressionOperator for ExpressionOperatorMinus {
    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value - b.value,
        };
    }
}

pub struct ExpressionOperatorTimes {

}

impl ToString for ExpressionOperatorTimes {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "*".to_string();
    }
}
impl ExpressionOperator for ExpressionOperatorTimes {

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value * b.value,
        };
    }
}

pub struct ExpressionOperatorDivide {

}

impl ToString for ExpressionOperatorDivide {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "/".to_string();
    }
}

impl ExpressionOperator for ExpressionOperatorDivide {
    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value / b.value,
        };
    }
}


// TODO: Make a constructor method?
fn parse_expression(input: &str) -> Result<Expression,InvalidExpressionError> {
    let mut parts: Vec<ExpressionPart> = Vec::new();
    let mut in_num: bool = false;
    let mut accum: u32 = 0;

    for (_i, &item) in input.as_bytes().iter().enumerate() {
        if item >= b'0' && item <= b'9' {
            in_num = true;
            accum *= 10;
            accum += (item - b'0') as u32;
        } else {
            if in_num {
                parts.push(ExpressionPart::Number(ExpressionNumber {
                    value: accum,
                }));
            }
            accum = 0;
            in_num = false;
            if item == b' ' || item == b'\n' {
                // No-Op (but end number)
            } else if item == b'+' {
                parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorPlus {
                })));
            } else if item == b'-' {
                parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorMinus {
                })));
            } else if item == b'*' {
                parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorTimes {
                })));
            } else if item == b'/' {
                parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorDivide {
                })));
            } else {
                return Err(InvalidExpressionError { message: format!("Cannot parse unrecognized character {}", item) });
            }
        }
    }

    return Ok(Expression {
        parts: parts,
    });
}

fn main() {
    println!("Enter an Expression to parse");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("You inputed: {}", input);

    let ex = match parse_expression(&input) {
        Ok(ex2) => ex2,
        Err(err) => panic!("{}", err),
    };
    println!("Expression: {}", ex.to_string());
    let res = match ex.calculate() {
        Ok(res2) => res2,
        Err(err) => panic!("{}", err),
    };
    println!("Calculation: {}", res.to_string());
}
