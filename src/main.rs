use std::io;
use std::fmt;

#[derive(Clone,Debug)]
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
    ExpectOperator,
    ExpectNumber,
}

impl Expression {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for part in &self.parts {
            s.push_str(&part.to_string());
            s.push_str(" ");
        }
        if (s.len() > 0) {
            // Get rid of the extra trailing space we added
            s.truncate(s.len()-1);
        }
        return s;
    }

    fn calculate(&self) -> Result<ExpressionNumber, InvalidExpressionError> {
        // TODO: Does not correctly implement order of expressions
        let mut state = ExpressionCalculateState::ExpectNumber;
        let mut cur: Option<ExpressionNumber> = None;
        let mut op: Option<&Box<dyn ExpressionOperator>> = None;

        for part in &self.parts {
            match state {
                ExpressionCalculateState::ExpectNumber => match part {
                    ExpressionPart::Number(num) => {
                        cur = match op {
                            Some(op2) => {
                                match cur {
                                    Some(cur2) => Some(op2.operate(&cur2, num)),
                                    None => return Err(InvalidExpressionError { message: format!("Operator missing first operand somehow") }),
                                }
                            },
                            None => Some(num.clone()),
                        };
                        state = ExpressionCalculateState::ExpectOperator;
                    },
                    ExpressionPart::Operator(op) => return Err(InvalidExpressionError { message: format!("Expected Number but got {}", op) }),
                },

                ExpressionCalculateState::ExpectOperator => match part {
                    ExpressionPart::Operator(op2) => {
                        op = Some(op2);
                        state = ExpressionCalculateState::ExpectNumber;
                    },
                    ExpressionPart::Number(num) => return Err(InvalidExpressionError { message: format!("Expected Operator but got {}", num) })
                }
            }
        }

        match state {
            ExpressionCalculateState::ExpectNumber => return Err(InvalidExpressionError { message: String::from("Expected number but string ended") }),
            _ => {},
        };

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

impl fmt::Display for ExpressionPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpressionPart::Number(num) => write!(f, "{}", num),
            ExpressionPart::Operator(op) => write!(f, "{}", op),
        }
    }
}


#[derive(Clone)]
pub struct ExpressionNumber {
    value: u32,
}

impl fmt::Display for ExpressionNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ExpressionNumber {
    // TODO: Inefficient
    fn len(&self) -> usize {
        return self.to_string().len();
    }
}

pub trait ExpressionOperator: fmt::Display {
    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber;
    fn len(&self) -> usize;
}

pub struct ExpressionOperatorPlus {

}

impl fmt::Display for ExpressionOperatorPlus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "+")
    }
}


impl ExpressionOperator for ExpressionOperatorPlus {
    fn len(&self) -> usize {
        1
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value + b.value,
        };
    }
}

pub struct ExpressionOperatorMinus {

}

impl fmt::Display for ExpressionOperatorMinus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-")
    }
}

impl ExpressionOperator for ExpressionOperatorMinus {
    fn len(&self) -> usize {
        1
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value - b.value,
        };
    }
}

pub struct ExpressionOperatorTimes {

}

impl fmt::Display for ExpressionOperatorTimes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "*")
    }
}

impl ExpressionOperator for ExpressionOperatorTimes {
    fn len(&self) -> usize {
        1
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value * b.value,
        };
    }
}

pub struct ExpressionOperatorDivide {

}

impl fmt::Display for ExpressionOperatorDivide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/")
    }
}

impl ExpressionOperator for ExpressionOperatorDivide {
    fn len(&self) -> usize {
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
        match item {
            b'0'..=b'9' => {
                in_num = true;
                accum *= 10;
                accum += (item - b'0') as u32;    
            },
            _ => {
                if in_num {
                    parts.push(ExpressionPart::Number(ExpressionNumber {
                        value: accum,
                    }));
                }
                accum = 0;
                in_num = false;
                match item {
                    b' ' | b'\n' | b'\r' => { } // No-op (but already ended number)
                    b'+' => parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorPlus { }))),
                    b'-' => parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorMinus { }))),
                    b'*' => parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorTimes { }))),
                    b'/' => parts.push(ExpressionPart::Operator(Box::new(ExpressionOperatorDivide { }))),
                    _ =>  return Err(InvalidExpressionError { message: format!("Cannot parse unrecognized character '{}'", item as char) }),
                }
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

    let ex = parse_expression(&input)
        .expect("Failed to parse expression");
    println!("Expression: {}", ex.to_string());

    let res = ex.calculate()
        .expect("Failed to calculate expression");
    println!("Calculation: {}", res.to_string());
}
