#[derive(Clone, Debug, PartialEq)]
pub struct DiceRoll {
    pub number_of_dice_to_roll: u32,
    pub dice_sides: u32,
    pub modifier: Option<i32>,
    pub roll_type: RollType,
}

impl DiceRoll {
    #[must_use]
    pub fn new(dice_sides: u32, modifier: Option<i32>, number_of_dice_to_roll: u32, roll_type: RollType) -> Self {
        DiceRoll {
            dice_sides,
            modifier,
            number_of_dice_to_roll,
            roll_type,
        }
    }
    
    #[must_use]
    pub fn new_regular_roll(dice_sides: u32, modifier: Option<i32>, number_of_dice_to_roll: u32) -> Self {
        DiceRoll {
            dice_sides,
            modifier,
            number_of_dice_to_roll,
            roll_type: RollType::Regular,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RollType {
    WithAdvantage,
    WithDisadvantage,
    Regular,
}

impl Default for RollType {
    fn default() -> Self {
        Self::Regular
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Addition,
    Subtraction,
}
