use std::fmt;
use std::str::FromStr;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

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

enum ExpressionCalculateState {
    ExpectOperator,
    ExpectNumber,
}

pub struct Expression {
    pub parts: Vec<ExpressionPart>,
}

impl Expression {
    pub fn calculate(&self) -> Result<ExpressionNumber, InvalidExpressionError> {
        // TODO: Is this efficient?
        let first_parts = &self.parts;
        let mut next_parts: Vec<ExpressionPart> = Vec::new();
        let mut next_parts_2: Vec<ExpressionPart> = Vec::new();


        // println!("parts: {:?}", first_parts);
        Expression::calculate_for_precedence(first_parts, 0, &mut next_parts)?;
        // println!("next_parts: {:?}", next_parts);
        Expression::calculate_for_precedence(&next_parts, 1, &mut next_parts_2)?;
        // println!("next_parts_2: {:?}", next_parts_2);

        if next_parts_2.len() != 1 {
            return Err(InvalidExpressionError { message: format!("Final expression didn't contained {} elements instead of 1", next_parts_2.len()) })
        }

        match &next_parts_2[0] {
            ExpressionPart::Number(num) => Ok(num.clone()),
            _ => Err(InvalidExpressionError { message: format!("next_parts_2 element 1 is not a number!") }),
        }
    }

    pub fn calculate_for_precedence(parts: &Vec<ExpressionPart>, precedence: u8, next_parts: &mut Vec<ExpressionPart>) -> Result<(), InvalidExpressionError> {
        let mut state = ExpressionCalculateState::ExpectNumber;
        let mut cur: Option<ExpressionNumber> = None;
        let mut op: Option<&Box<dyn ExpressionOperator>> = None;

        for part in parts {
            match state {
                ExpressionCalculateState::ExpectNumber => match part {
                    ExpressionPart::Number(num) => {
                        cur = match op {
                            Some(op2) => {
                                match cur {
                                    Some(cur2) => {
                                        op = None;
                                        Some(op2.operate(&cur2, num)?)
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
                        if op2.precedence() == precedence {
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
        }

        match state {
            ExpressionCalculateState::ExpectNumber => return Err(InvalidExpressionError { message: String::from("Expected number but expression ended") }),
            _ => {},
        };

        match cur {
            Some(cur2) => {
                next_parts.push(ExpressionPart::Number(cur2.clone()));
            }
            None => { }
        };

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.parts.iter().fold(0, |sum, part| sum + ExpressionPart::len(&part))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first = true;
        for part in &self.parts {
            if is_first {
                is_first = false;
            } else {
                // Uncomment for more pretty but harder-to-eyeball length
                // write!(f, " ")?
            }
            write!(f, "{}", &part.to_string())?
        }
        Ok(())
    }
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

impl ExpressionPart {
    pub fn len(&self) -> usize {
        match self {
            ExpressionPart::Number(num) => num.len(),
            ExpressionPart::Operator(op) => op.len(),
        }
    }

    pub fn from_char_byte(char_byte: &u8) -> Result<ExpressionPart, InvalidExpressionError> {
        match char_byte {
            b'+' => Ok(ExpressionPart::Operator(Box::new(ExpressionOperatorPlus { }))),
            b'-' => Ok(ExpressionPart::Operator(Box::new(ExpressionOperatorMinus { }))),
            b'*' => Ok(ExpressionPart::Operator(Box::new(ExpressionOperatorTimes { }))),
            b'/' => Ok(ExpressionPart::Operator(Box::new(ExpressionOperatorDivide { }))),
            _ => Err(InvalidExpressionError { message: format!("Cannot parse unrecognized operator character '{}'", *char_byte as char) })
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionNumber {
    pub value: i32,
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

// TODO: This should be merged into ExpressionOperator, possibly replace it
#[derive(Debug)]
pub enum ExpressionOperatorEnum {
    Plus,
    Minus,
    Times,
    Divide,
}

impl ExpressionOperatorEnum {
    pub fn from_char_byte(char_byte: &u8) -> Result<ExpressionOperatorEnum, InvalidExpressionError> {
        match char_byte {
            b'+' => Ok(ExpressionOperatorEnum::Plus),
            b'-' => Ok(ExpressionOperatorEnum::Minus),
            b'*' => Ok(ExpressionOperatorEnum::Times),
            b'/' => Ok(ExpressionOperatorEnum::Divide),
            _ => Err(InvalidExpressionError { message: format!("Cannot parse unrecognized operator character '{}'", *char_byte as char) })
        }
    }
    pub fn to_char_byte(&self) -> u8 {
        self.to_char() as u8
    }

    pub fn to_char(&self) -> char {
        match self {
            ExpressionOperatorEnum::Plus => '+',
            ExpressionOperatorEnum::Minus => '-',
            ExpressionOperatorEnum::Times => '*',
            ExpressionOperatorEnum::Divide => '/',
        }
    }
}

impl Distribution<ExpressionOperatorEnum> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ExpressionOperatorEnum {
        match rng.gen_range(0..4) {
            0 => ExpressionOperatorEnum::Plus,
            1 => ExpressionOperatorEnum::Minus,
            2 => ExpressionOperatorEnum::Times,
            3 => ExpressionOperatorEnum::Divide,
            _ => panic!("Out-of-range random number chosen!")
        }
    }
}

impl fmt::Display for ExpressionOperatorEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}


pub trait ExpressionOperator: ExpressionOperatorClone + fmt::Display + fmt::Debug {
    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError>;
    fn len(&self) -> usize {
        1
    }
    fn precedence(&self) -> u8;
    fn as_char(&self) -> char {
        // TODO: Invert this implementation
        self.to_string().chars().next().expect("Operator string had no chars?!")
    }
    fn as_char_byte(&self) -> u8 {
        self.as_char() as u8
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



#[derive(Clone, Debug)]
pub struct ExpressionOperatorPlus {
}

impl fmt::Display for ExpressionOperatorPlus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "+")
    }
}


impl ExpressionOperator for ExpressionOperatorPlus {
    fn precedence(&self) -> u8 {
        1
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        let value = a.value.checked_add(b.value).ok_or(InvalidExpressionError { message: format!("Could not compute {} + {}", a, b)} )?;
        Ok(ExpressionNumber {
            value,
        })
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
    fn precedence(&self) -> u8 {
        1
    }


    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        let value = a.value.checked_sub(b.value).ok_or(InvalidExpressionError { message: format!("Could not compute {} - {}", a, b)} )?;
        Ok(ExpressionNumber {
            value,
        })
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
    fn precedence(&self) -> u8 {
        0
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        let value = a.value.checked_mul(b.value).ok_or(InvalidExpressionError { message: format!("Could not compute {} * {}", a, b)} )?;
        Ok(ExpressionNumber {
            value,
        })
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
    fn precedence(&self) -> u8 {
        0
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        let rem = a.value.checked_rem(b.value).ok_or(InvalidExpressionError { message: format!("Could not compute {} / {}", a, b)} )?;
        if rem != 0 {
            return Err(InvalidExpressionError { message: format!("Found remainder in {} / {}", a, b)})
        }
        let value = a.value.checked_div(b.value).ok_or(InvalidExpressionError { message: format!("Could not compute {} / {}", a, b)} )?;
        Ok(ExpressionNumber {
            value,
        })
    }
}

impl FromStr for Expression {
    type Err = InvalidExpressionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts: Vec<ExpressionPart> = Vec::new();
        let mut in_num: bool = false;
        let mut accum: i32 = 0;

        // Simulate an extra space on the end so we get the last number
        let iter = input.as_bytes().iter().chain(" ".as_bytes().iter());
        for &item in iter {
            match item {
                b'0'..=b'9' => {
                    in_num = true;
                    accum *= 10;
                    accum += (item - b'0') as i32;
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
                        ch => parts.push(ExpressionPart::from_char_byte(&ch)?),
                    }
                }
            }
        }

        Ok(Expression {
            parts: parts,
        })
    }
}


pub fn mknum(x:i32) -> ExpressionNumber {
    ExpressionNumber {
        value: x
    }
}

// pub fn mknump(x:i32) -> ExpressionPart {
//     ExpressionPart::Number(mknum(x))
// }
