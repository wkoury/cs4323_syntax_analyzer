#![warn(clippy::all)]
// Keeping track of a few types of errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    InvalidSymbol,
    ConstantHasTooManyPeriods,
    IdentifierBeginsWithNumber,
    // Just a general placeholder syntax error
    // SyntaxError,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub(crate) error_type: ErrorType,
    pub(crate) token: String,
}
