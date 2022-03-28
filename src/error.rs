#![warn(clippy::all)]
// Keeping track of a few types of errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    InvalidSymbol,
    ConstantHasTooManyPeriods,
    IdentifierBeginsWithNumber,
    SyntaxError, // Just a general placeholder syntax error
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub(crate) error_type: ErrorType,
    pub(crate) token: String,
}
