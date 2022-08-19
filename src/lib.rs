#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::pedantic::module_name_repetitions)]

use nom::{branch, bytes, character, combinator, sequence, Err};

pub mod dice_roll;
pub mod error;
pub mod dice_roll_with_op;
mod parse;

use crate::dice_roll::{DiceRoll, Operation, RollType};
use crate::dice_roll_with_op::DiceRollWithOp;
use crate::error::ParserError;
use crate::parse::terminated_spare;

fn parse_end_of_input_or_modifier(i: &str) -> nom::IResult<&str, &str> {
    branch::alt((bytes::complete::tag("+"), bytes::complete::tag("-"), bytes::complete::tag(","), combinator::eof))(i)
}

fn parse_operator_as_value(i: &str) -> nom::IResult<&str, Operation> {
    branch::alt((combinator::value(Operation::Addition, character::complete::char('+')), combinator::value(Operation::Subtraction, character::complete::char('-'))))(i)
}

fn parse_roll_type(i: &str) -> nom::IResult<&str, RollType> {
    let result = combinator::opt(branch::alt((combinator::value(RollType::WithAdvantage, terminated_spare(bytes::complete::tag_no_case("a"), parse_end_of_input_or_modifier)), combinator::value(RollType::WithDisadvantage, terminated_spare(bytes::complete::tag_no_case("d"), parse_end_of_input_or_modifier)))))(i);
    match result {
        Ok((i, None)) => Ok((i, RollType::Regular)),
        Ok((i, Some(roll_type))) => Ok((i, roll_type)),
        Err(e) => Err(e),
    }
}

fn parse_dice_parts(i: &str) -> nom::IResult<&str, (Option<&str>, &str, &str)> {
    sequence::tuple((combinator::opt(character::complete::digit1), bytes::complete::tag_no_case("d"), character::complete::digit1))(i)
}

fn parse_roll_as_value(i: &str) -> nom::IResult<&str, DiceRoll> {
    branch::alt((parse_dice_with_operator, parse_dice_without_operator))(i)
}

fn parse_dice_with_operator(i: &str) -> nom::IResult<&str, DiceRoll> {
    let result = sequence::tuple((parse_dice_parts, parse_operator_as_value, terminated_spare(character::complete::digit1, combinator::not(sequence::tuple((bytes::complete::tag_no_case("d"), character::complete::digit1)))), parse_roll_type))(i);
    match result {
        Ok((remaining, ((number_of_dice, _, dice_sides), operation, modifier, roll_type))) => Ok((remaining, dice_roll_from_parsed_items(number_of_dice, dice_sides, Some(operation), Some(modifier), roll_type))),
        Err(e) => Err(e),
    }
}

fn parse_dice_without_operator(i: &str) -> nom::IResult<&str, DiceRoll> {
    let result = sequence::tuple((parse_dice_parts, parse_roll_type))(i);
    match result {
        Ok((remaining, ((number_of_dice, _, dice_sides), roll_type))) => Ok((remaining, dice_roll_from_parsed_items(number_of_dice, dice_sides, None, None, roll_type))),
        Err(e) => Err(e),
    }
}

fn parse_statement_with_leading_op(i: &str) -> nom::IResult<&str, Vec<DiceRollWithOp>> {
    let (remaining, (operation, roll, later_rolls)) = sequence::tuple((parse_operator_as_value, parse_roll_as_value, branch::alt((parse_statement_with_leading_op, combinator::value(Vec::new(), character::complete::space0)))))(i)?;
    let mut rolls = Vec::with_capacity(later_rolls.len() + 1);
    rolls.push(DiceRollWithOp::new(roll, operation));
    for roll in later_rolls {
        rolls.push(roll);
    }
    Ok((remaining, rolls))
}

fn parse_initial_statement(i: &str) -> nom::IResult<&str, DiceRollWithOp> {
    let (remaining, (operator, roll)) = sequence::tuple((combinator::opt(parse_operator_as_value), parse_roll_as_value))(i)?;
    Ok((remaining, DiceRollWithOp::new(roll, operator.unwrap_or(Operation::Addition))))
}

fn parse_statements(i: &str) -> nom::IResult<&str, Vec<DiceRollWithOp>> {
    let (remaining, (operation, roll, later_rolls)) = sequence::tuple((parse_operator_as_value, parse_roll_as_value, branch::alt((parse_statement_with_leading_op, combinator::value(Vec::new(), character::complete::space0)))))(i)?;
    let mut rolls = Vec::with_capacity(later_rolls.len() + 1);
    rolls.push(DiceRollWithOp::new(roll, operation));
    for roll in later_rolls {
        rolls.push(roll);
    }
    Ok((remaining, rolls))
}

fn parse_group(i: &str) -> nom::IResult<&str, Vec<DiceRollWithOp>> {
    let (remaining, (initial_roll, additional_rolls)) = sequence::tuple((parse_initial_statement, branch::alt((parse_statements, combinator::value(Vec::new(), character::complete::space0)))))(i)?;

    let mut rolls = Vec::with_capacity(additional_rolls.len() + 1);
    rolls.push(initial_roll);
    for roll in additional_rolls {
        rolls.push(roll);
    }
    Ok((remaining, rolls))
}

fn parse_groups(i: &str) -> nom::IResult<&str, Vec<Vec<DiceRollWithOp>>> {
    let (remaining, (group_rolls, other_groups)) = sequence::tuple((parse_group, combinator::opt(sequence::tuple((character::complete::char(','), parse_groups)))))(i)?;

    let other_groups_size = match &other_groups {
        Some((_, rolls)) => rolls.len(),
        None => 0,
    };

    let mut rolls: Vec<Vec<DiceRollWithOp>> = Vec::with_capacity(other_groups_size + 1);
    rolls.push(group_rolls);
    if other_groups.is_some() {
        let (_, other_groups_rolls) = other_groups.unwrap();
        rolls.extend(other_groups_rolls);
    }
    Ok((remaining, rolls))
}

fn dice_roll_from_parsed_items(number_of_dice: Option<&str>, dice_sides: &str, modifier_operation: Option<Operation>, modifier_value: Option<&str>, roll_type: RollType) -> DiceRoll {
    let number_of_dice: u32 = number_of_dice.map_or(Ok(1), str::parse).unwrap();
    let dice_sides: u32 = dice_sides.parse().unwrap();
    if modifier_operation.is_some() && modifier_value.is_some() {
        let modifier_operation = modifier_operation.unwrap();
        let modifier_value: i32 = modifier_value.unwrap().parse().unwrap();
        match modifier_operation {
            Operation::Addition => {
                return DiceRoll::new(dice_sides, Some(modifier_value), number_of_dice, roll_type)
            }
            Operation::Subtraction => {
                let modifier = Some(-modifier_value);
                return DiceRoll::new(dice_sides, modifier, number_of_dice, roll_type)
            }
        }
    }
    DiceRoll::new(dice_sides, None, number_of_dice, roll_type)
}

pub fn parse_line(i: &str) -> Result<Vec<Vec<DiceRollWithOp>>, ParserError> {
    let whitespaceless: String = i.replace(" ", "");

    match parse_groups(&whitespaceless) {
        Ok((remaining, dice_rolls)) => {
            if !remaining.trim().is_empty() {
                return Err(ParserError::ParseError(format!("Expected remaining input to be empty, found: {0}", remaining)));
            }
            return Ok(dice_rolls);
        }
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            return Err(ParserError::ParseError(format!("{0}", e)));
        }
        Err(Err::Incomplete(_)) => {
            return Err(ParserError::Unknown);
        }
    }
}
