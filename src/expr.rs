use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct InvalidExpressionError {
    message: String,
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidExpressionError: {}", self.message)
    }
}

impl fmt::Debug for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "InvalidExpressionError: {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
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
    // TODO: Use formatter instead or something
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for part in &self.parts {
            s.push_str(&part.to_string());
            s.push_str(" ");
        }
        if s.len() > 0 {
            // Get rid of the extra trailing space we added
            s.truncate(s.len()-1);
        }
        return s;
    }

    pub fn calculate(&self) -> Result<ExpressionNumber, InvalidExpressionError> {
        // TODO: Does not correctly implement order of expressions
        let mut state = ExpressionCalculateState::ExpectNumber;
        let mut cur: Option<ExpressionNumber> = None;
        let mut op: Option<&Box<dyn ExpressionOperator>> = None;
        // TODO: Is this efficient?
        let mut next_parts: Vec<ExpressionPart> = Vec::new();
        let mut parts = &self.parts;


        // TODO: If we e.g. unshifted from the vec would that make ownership simpler?

        // First pass for order of operations: Multiplication, Division
        // TODO: Fix code duplication (maybe loop over precedence list)
        // println!("parts: {:?}", parts);
        for part in parts {
            match state {
                ExpressionCalculateState::ExpectNumber => match part {
                    ExpressionPart::Number(num) => {
                        cur = match op {
                            Some(op2) => {
                                match cur {
                                    Some(cur2) => {
                                        op = None;
                                        Some(op2.operate(&cur2, num))
                                    }
                                    None => return Err(InvalidExpressionError { message: format!("Operator missing first operand somehow L67") }),
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
                        if op2.precedence() == 0 {
                            op = Some(op2);
                        } else {
                            match cur {
                                Some(cur2) => {
                                    next_parts.push(ExpressionPart::Number(cur2));
                                    next_parts.push(ExpressionPart::Operator(op2.clone()));
                                    cur = None;
                                },
                                None => return Err(InvalidExpressionError { message: format!("Operator missing first operand somehow") }),
                            }
                        }
                        state = ExpressionCalculateState::ExpectNumber;
                    },
                    ExpressionPart::Number(num) => return Err(InvalidExpressionError { message: format!("Expected Operator but got {}", num) })
                }
            }
        };

        match cur {
            Some(cur2) => {
                next_parts.push(ExpressionPart::Number(cur2.clone()));
                cur = None;         
            }
            None => { }
        };

        // Now loop again for lower-priority operators

        state = ExpressionCalculateState::ExpectNumber;
        parts = &next_parts;
        // println!("parts: {:?}", parts);
        for part in parts {
            match state {
                ExpressionCalculateState::ExpectNumber => match part {
                    ExpressionPart::Number(num) => {
                        cur = match op {
                            Some(op2) => {
                                match cur {
                                    Some(cur2) => Some(op2.operate(&cur2, &num)),
                                    None => return Err(InvalidExpressionError { message: format!("Operator missing first operand somehow L122") }),
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
                        op = Some(&op2);
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

#[derive(Debug)]
pub enum ExpressionPart {
    Number(ExpressionNumber),
    Operator(Box<dyn ExpressionOperator>),
}

impl fmt::Display for ExpressionPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // TODO: Can this be simplified?
            ExpressionPart::Number(num) => write!(f, "{}", num),
            ExpressionPart::Operator(op) => write!(f, "{}", op),
        }
    }
}


#[derive(Clone, Debug)]
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
    pub fn len(&self) -> usize {
        return self.to_string().len();
    }
}

pub trait ExpressionOperatorClone {
    fn clone_box(&self) -> Box<dyn ExpressionOperator>;
}

impl<T> ExpressionOperatorClone for T
where
    T: 'static + ExpressionOperator + Clone,
{
    fn clone_box(&self) -> Box<dyn ExpressionOperator> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn ExpressionOperator> {
    fn clone(&self) -> Box<dyn ExpressionOperator> {
        self.clone_box()
    }
}

pub trait ExpressionOperator: ExpressionOperatorClone + fmt::Display + fmt::Debug {
    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber;
    fn len(&self) -> usize;
    fn precedence(&self) -> u8;
}


#[derive(Clone, Debug)]
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

    fn precedence(&self) -> u8 {
        1
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value + b.value,
        };
    }
}

#[derive(Clone, Debug)]
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

    fn precedence(&self) -> u8 {
        1
    }


    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value - b.value,
        };
    }
}

#[derive(Clone, Debug)]
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

    fn precedence(&self) -> u8 {
        0
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value * b.value,
        };
    }
}

#[derive(Clone, Debug)]
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

    fn precedence(&self) -> u8 {
        0
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> ExpressionNumber {
        return ExpressionNumber {
            value: a.value / b.value,
        };
    }
}

impl FromStr for Expression {
    type Err = InvalidExpressionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

        Ok(Expression {
            parts: parts,
        })
    }
}
