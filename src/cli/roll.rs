use clap::Args;
use pinbreak::domain::dice::DicePool;

#[derive(Args)]
pub struct RollArgs {
    /// The size of the dice pool
    pub amount: u32,
    /// The type of dice
    pub sides: u32,
    /// The amount of times to repeat the roll
    pub times: u32,
}

impl RollArgs {
    pub fn run(&self) {
        let pool = DicePool {
            amount: self.amount,
            sides: self.sides,
        };
        println!(
            "results of rolling {}d{} {} times: {}",
            pool.amount,
            pool.sides,
            self.times,
            pool.roll_mult(self.times)
        );
        println!(
            "average of rolling {}d{} {} times: {}",
            pool.amount,
            pool.sides,
            self.times,
            pool.average_over(self.times)
        );
    }
}
