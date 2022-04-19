use std::fmt;
use std::str::FromStr;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

use crate::util::num_digits;

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

    pub fn len(&self) -> Result<usize, InvalidExpressionError> {
        self.parts.iter().fold(Ok(0), |sum, part| -> Result<usize, InvalidExpressionError> {
            match sum {
                Ok(sum) => Ok(sum + ExpressionPart::len(&part)?),
                Err(err) => Err(err)
            }
        })
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
            ExpressionPart::Number(num) => write!(f, "{}", num),
            ExpressionPart::Operator(op) => write!(f, "{}", op),
        }
    }
}

impl ExpressionPart {
    pub fn len(&self) -> Result<usize, InvalidExpressionError> {
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
     numerator: i32,
     denominator: i32,
}

impl fmt::Display for ExpressionNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_int() {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl Default for ExpressionNumber {
    fn default() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }
}

impl ExpressionNumber {
    pub fn new(value: i32) -> ExpressionNumber {
        ExpressionNumber {
            numerator: value,
            ..Default::default()
        }
    }

    pub fn len(&self) -> Result<usize, InvalidExpressionError> {
        if self.is_int() {
            Ok(num_digits(self.numerator) as usize)
        } else {
            Err(InvalidExpressionError { message: format!("Cannot get length of non-integer value {}/{}", self.numerator, self.denominator) })
        }
    }

    // TODO: Not really a great error type
    pub fn int_value(&self) -> Result<i32, InvalidExpressionError> {
        if self.is_int() {
            Ok(self.numerator)
        } else {
            Err(InvalidExpressionError { message: format!("Cannot get integer value of {}/{}", self.numerator, self.denominator) })
        }
    }

    pub fn is_int(&self) -> bool {
        self.denominator == 1
    }

    pub fn simplify(self) -> ExpressionNumber {
        if (self.numerator % self.denominator) == 0 {
            ExpressionNumber::new(self.numerator / self.denominator)
        } else {
            // Cannot be simplified to int, just leave as-is.
            self
        }
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


pub trait ExpressionOperator: ExpressionOperatorClone + fmt::Debug {
    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError>;

    fn len(&self) -> Result<usize, InvalidExpressionError> {
        Ok(1)
    }

    fn precedence(&self) -> u8;

    fn as_char(&self) -> char;

    fn as_char_byte(&self) -> u8 {
        self.as_char() as u8
    }
}

impl fmt::Display for dyn ExpressionOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_char())
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

impl ExpressionOperator for ExpressionOperatorPlus {
    fn precedence(&self) -> u8 {
        1
    }

    fn as_char(&self) -> char {
        '+'
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        match (a.int_value(), b.int_value()) {
            (Ok(a_value), Ok(b_value)) => {
                let value = a_value.checked_add(b_value).ok_or(InvalidExpressionError { message: format!("Could not compute {} + {}", a, b)} )?;
                Ok(ExpressionNumber::new(value))
            }
            (_, _) => Err(InvalidExpressionError { message: format!("Could not add {} + {}: one or both values are not integers", a, b)})
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionOperatorMinus {

}

impl ExpressionOperator for ExpressionOperatorMinus {
    fn precedence(&self) -> u8 {
        1
    }

    fn as_char(&self) -> char {
        '-'
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        match (a.int_value(), b.int_value()) {
            (Ok(a_value), Ok(b_value)) => {
                let value = a_value.checked_sub(b_value).ok_or(InvalidExpressionError { message: format!("Could not compute {} - {}", a, b)} )?;
                Ok(ExpressionNumber::new(value))
            }
            (_, _) => Err(InvalidExpressionError { message: format!("Could not add {} + {}: one or both values are not integers", a, b)})
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionOperatorTimes {
}

impl ExpressionOperator for ExpressionOperatorTimes {
    fn precedence(&self) -> u8 {
        0
    }

    fn as_char(&self) -> char {
        '*'
    }

    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        let numerator = a.numerator.checked_mul(b.numerator).ok_or(InvalidExpressionError { message: format!("Could not compute numerator for {} * {}", a, b)} )?;
        let denominator = a.denominator.checked_mul(b.denominator).ok_or(InvalidExpressionError { message: format!("Could not compute numerator for {} * {}", a, b)} )?;
        Ok(ExpressionNumber {
            numerator,
            denominator,
        }.simplify())
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionOperatorDivide {

}

impl ExpressionOperator for ExpressionOperatorDivide {
    fn precedence(&self) -> u8 {
        0
    }

    fn as_char(&self) -> char {
        '/'
    }


    fn operate(&self, a: &ExpressionNumber, b: &ExpressionNumber) -> Result<ExpressionNumber, InvalidExpressionError> {
        // flip numerator and denominator of b
        let numerator = a.numerator.checked_mul(b.denominator).ok_or(InvalidExpressionError { message: format!("Could not compute numerator for {} * {}", a, b)} )?;
        let denominator = a.denominator.checked_mul(b.numerator).ok_or(InvalidExpressionError { message: format!("Could not compute numerator for {} * {}", a, b)} )?;
        Ok(ExpressionNumber {
            numerator,
            denominator,
        }.simplify())
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
                        parts.push(ExpressionPart::Number(ExpressionNumber::new(accum)));
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
    ExpressionNumber::new(x)
}

// pub fn mknump(x:i32) -> ExpressionPart {
//     ExpressionPart::Number(mknum(x))
// }
#[cfg(test)]
#[test]
fn simple_int_test() {
    {
        let a = ExpressionNumber::new(10);
        assert!(a.is_int());
        assert_eq!(a.int_value().unwrap(), 10);
        assert_eq!(a.len().unwrap(), 2);
    }
    {
        let a = ExpressionNumber::new(1);
        assert!(a.is_int());
        assert_eq!(a.int_value().unwrap(), 1);
        assert_eq!(a.len().unwrap(), 1);
    }

    {
        let a = ExpressionNumber::new(0);
        assert!(a.is_int());
        assert_eq!(a.int_value().unwrap(), 0);
        assert_eq!(a.len().unwrap(), 1);
    }

    {
        let a = ExpressionNumber::new(-1);
        assert!(a.is_int());
        assert_eq!(a.int_value().unwrap(), -1);
        assert_eq!(a.len().unwrap(), 2);
    }

    {
        let a = ExpressionNumber::new(-10);
        assert!(a.is_int());
        assert_eq!(a.int_value().unwrap(), -10);
        assert_eq!(a.len().unwrap(), 3);
    }
}

#[test]
fn simple_frac_test() {
    {
        let a = ExpressionNumber {
            numerator: 1,
            denominator: 2,
        };
        assert!(!a.is_int());
        assert!(a.int_value().is_err());
        assert!(a.len().is_err());
    }
}

#[test]
fn simplify_test() {
    {
        let a = ExpressionNumber {
            numerator: 144,
            denominator: 12,
        };
        assert!(!a.is_int());
        assert!(a.int_value().is_err());
        assert!(a.len().is_err());

        let s = a.simplify();
        assert!(s.is_int());
        assert_eq!(s.int_value().unwrap(), 12);
        assert_eq!(s.len().unwrap(), 2);
    }
}

#[test]
fn add_int_test() {
    let plus = ExpressionOperatorPlus { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber::new(2);
        let c = plus.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 12);
    }
}

#[test]
fn add_frac_test() {
    let plus = ExpressionOperatorPlus { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber {
            numerator: 1,
            denominator: 2,
        };
        assert!(plus.operate(&a, &b).is_err());
    }
}

#[test]
fn sub_int_test() {
    let minus = ExpressionOperatorMinus { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber::new(2);
        let c = minus.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 8);
    }
}

#[test]
fn sub_frac_test() {
    let minus = ExpressionOperatorMinus { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber {
            numerator: 1,
            denominator: 2,
        };
        assert!(minus.operate(&a, &b).is_err());
    }
}

#[test]
fn mul_int_test() {
    let times = ExpressionOperatorTimes { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber::new(2);
        let c = times.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 20);
    }
}

#[test]
fn mul_frac_test() {
    let times = ExpressionOperatorTimes { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber {
            numerator: 1,
            denominator: 2,
        };
        let c = times.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 5);
    }
}


#[test]
fn div_int_test() {
    let divide = ExpressionOperatorDivide { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber::new(2);
        let c = divide.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 5);
    }
}

#[test]
fn div_frac_test() {
    let divide = ExpressionOperatorDivide { };
    {
        let a = ExpressionNumber::new(10);
        let b = ExpressionNumber {
            numerator: 1,
            denominator: 2,
        };
        let c = divide.operate(&a, &b).unwrap();
        assert_eq!(c.int_value().unwrap(), 20);
    }

    {
        let a = ExpressionNumber::new(1);
        let b = ExpressionNumber::new(2);
        let c = divide.operate(&a, &b).unwrap();
        assert_eq!(c.numerator, 1);
        assert_eq!(c.denominator, 2);
        assert!(!c.is_int());
        assert!(c.int_value().is_err());

        let d = divide.operate(&c, &ExpressionNumber {
            numerator: 1,
            denominator: 4,
        }).unwrap();
        assert!(d.is_int());
        assert_eq!(d.int_value().unwrap(), 2);
    }
}
