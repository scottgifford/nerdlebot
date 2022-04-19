use std::fmt;
use std::str::FromStr;

use crate::expr::Expression;
use crate::expr::ExpressionPart;
use crate::expr::ExpressionNumber;
use crate::expr::InvalidExpressionError;

pub struct Equation {
    pub expr: Expression,
    pub res: ExpressionNumber,
}

impl Equation {
    pub fn computes(&self) -> Result<bool, InvalidExpressionError> {
        let calc = self.expr.calculate()?;
        Ok(calc == self.res)
    }

    pub fn len(&self) -> Result<usize, InvalidEquationError> {
        Ok(self.expr.len()? + self.res.len()? + 1)
    }
}

impl FromStr for Equation {
    type Err = InvalidEquationError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut split = input.split("=");
        let expr = split.next().ok_or_else(|| InvalidEquationError { message: format!("Could not find equal sign in '{}'", input) } )?;
        let expr = Expression::from_str(&expr)?;
        let res = split.next().ok_or_else(|| InvalidEquationError { message: format!("Could not find value after equal sign '{}'", input) } )?;
        let res = Expression::from_str(&res)?;
        if res.parts.len() != 1 {
            return Err(InvalidEquationError { message: format!("RHS must be a simple number in '{}'", input) } );
        }
        let res = match &res.parts[0] {
            ExpressionPart::Number(n) => n.clone(),
            _ => return Err(InvalidEquationError { message: format!("RHS must be a simple number in '{}'", input) } )
        };
        if split.next() != None {
            return Err(InvalidEquationError {message: format!("Too many equal signs in '{}'", input) } );
        }
        Ok(Equation {
            expr,
            res,
        })
    }
}

impl fmt::Display for Equation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}={}",self.expr,self.res)
    }
}

#[derive(Clone)]
pub struct InvalidEquationError {
    message: String,
}

impl fmt::Display for InvalidEquationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidEquationError: {}", self.message)
    }
}

impl fmt::Debug for InvalidEquationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Line and file are this one, not caller?!
        write!(f, "InvalidEquationError: {} at {{ file: {}, line: {} }}", self.message, file!(), line!()) // programmer-facing output
    }
}

impl From<InvalidExpressionError> for InvalidEquationError {
    fn from(error: InvalidExpressionError) -> Self {
        InvalidEquationError { message : error.to_string() }
    }
}
