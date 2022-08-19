use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("An error occurred while parsing the input. {0}")]
    ParseError(String),
    #[error("An invalid number was entered.")]
    InvalidNumberInput(#[from] ParseIntError),
    #[error("An unknown error occurred.")]
    Unknown,
}
