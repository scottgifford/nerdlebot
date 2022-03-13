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
        let mut state = 0; // 0=expect num, 1=expect op; TODO: Use enum?

        let mut cur: Option<ExpressionNumber> = None;
        let mut op: Option<&Box<dyn ExpressionPart>> = None;

        for part in &self.parts {
            if state == 0 {
                cur = Some(part.as_expression_number().clone());
                state = 1;
            } else if state == 1 {
                op = Some(part);
                state = 2;
            } else if state == 2 {
                match op {
                    Some(op2) => match cur {
                        Some(cur2) => cur = Some(op2.operate(cur2.as_expression_number(), part.as_expression_number())),
                        None => return Err(InvalidExpressionError { message: String::from("Operator missing first operand") }),
                    }
                    None => return Err(InvalidExpressionError { message: String::from("Expected operator") }),
                }
                state = 1;
            }
        }

        match cur {
            Some(ret) => return Ok(ret),
            None => return Err(InvalidExpressionError { message: String::from("No values found") }),
        }
    }
}

pub struct Expression {
    parts: Vec<Box<dyn ExpressionPart>>,
}


pub trait ExpressionPart {
    // TODO: Something better than String?
    fn to_string(&self) -> String;
    // TODO: Change to u32
    fn len(&self) -> usize;

    // TODO: This is kind of hacky?..
    fn get_value(&self) -> u32;
    fn as_expression_number(&self) -> &ExpressionNumber;

    fn operate(&self, a: &ExpressionNumber , b: &ExpressionNumber) -> ExpressionNumber;
}

#[derive(Clone)]
pub struct ExpressionNumber {
    value: u32,
}

impl ExpressionPart for ExpressionNumber {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    // TODO: Inefficient
    fn len(&self) -> usize {
        return self.to_string().len();
    }

    fn get_value(&self) -> u32 {
        return self.value;
    }

    fn as_expression_number(&self) -> &ExpressionNumber {
        return self;
    }

    fn operate(&self, _a: &ExpressionNumber, _b: &ExpressionNumber) -> ExpressionNumber {
        panic!("ExpressionNumber cannot perform an operation");
    }
}

pub trait ExpressionOperator: ExpressionPart {
}

pub struct ExpressionOperatorPlus {

}

impl ExpressionPart for ExpressionOperatorPlus {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "+".to_string();
    }

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn as_expression_number(&self) -> &ExpressionNumber {
        panic!("Operator does not have a value");
    }

    fn get_value(&self) -> u32 {
        panic!("Operator does not have a value");
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value + b.value,
        };
    }
}

pub struct ExpressionOperatorMinus {

}

impl ExpressionPart for ExpressionOperatorMinus {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "-".to_string();
    }

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn as_expression_number(&self) -> &ExpressionNumber {
        panic!("Operator does not have a value");
    }

    fn get_value(&self) -> u32 {
        panic!("Operator does not have a value");
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value - b.value,
        };
    }
}

pub struct ExpressionOperatorTimes {

}

impl ExpressionPart for ExpressionOperatorTimes {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "*".to_string();
    }

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn as_expression_number(&self) -> &ExpressionNumber {
        panic!("Operator does not have a value");
    }

    fn get_value(&self) -> u32 {
        panic!("Operator does not have a value");
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value * b.value,
        };
    }
}

pub struct ExpressionOperatorDivide {

}

impl ExpressionPart for ExpressionOperatorDivide {
    fn to_string(&self) -> String {
        // TODO: Why is this return needed?
        return "/".to_string();
    }

    fn len(&self) -> usize {
        // TODO: Why is this return needed?
        return 1;
    }

    fn as_expression_number(&self) -> &ExpressionNumber {
        panic!("Operator does not have a value");
    }

    fn get_value(&self) -> u32 {
        panic!("Operator does not have a value");
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value / b.value,
        };
    }
}


// TODO: Make a constructor method?
fn parse_expression(input: &str) -> Result<Expression,InvalidExpressionError> {
    let mut parts: Vec<Box<dyn ExpressionPart>> = Vec::new();
    let mut in_num: bool = false;
    let mut accum: u32 = 0;

    for (_i, &item) in input.as_bytes().iter().enumerate() {
        if item >= b'0' && item <= b'9' {
            in_num = true;
            accum *= 10;
            accum += (item - b'0') as u32;
        } else {
            if in_num {
                parts.push(Box::new(ExpressionNumber {
                    value: accum,
                }));
            }
            accum = 0;
            in_num = false;
            if item == b' ' || item == b'\n' {
                // No-Op (but end number)
            } else if item == b'+' {
                parts.push(Box::new(ExpressionOperatorPlus {
                }));
            } else if item == b'-' {
                parts.push(Box::new(ExpressionOperatorMinus {
                }));
            } else if item == b'*' {
                parts.push(Box::new(ExpressionOperatorTimes {
                }));
            } else if item == b'/' {
                parts.push(Box::new(ExpressionOperatorDivide {
                }));
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
