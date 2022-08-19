use crate::dice_roll::{DiceRoll, Operation};

#[derive(Clone, Debug, PartialEq)]
pub struct DiceRollWithOp {
    pub dice_roll: DiceRoll,
    pub operation: Operation,
}

impl DiceRollWithOp {
    #[must_use]
    pub fn new(dice_roll: DiceRoll, operation: Operation) -> Self {
        DiceRollWithOp {
            dice_roll,
            operation,
        }
    }
}
