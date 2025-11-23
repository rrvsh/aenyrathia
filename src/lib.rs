use rand::Rng;

pub struct DicePool {
    pub amount: u32,
    pub sides: u32,
}

impl DicePool {
    fn roll(&self) -> u32 {
        let mut rng = rand::rng();
        let mut result = 0;
        for _ in 0..self.amount {
            result += rng.random_range(1..=self.sides);
        }
        result
    }
    pub fn roll_mult(&self, times: u32) -> u32 {
        let mut result = 0;
        for _ in 0..times {
            result += self.roll();
        }
        result
    }
    pub fn average_over(&self, times: u32) -> u32 {
        self.roll_mult(times) / times
    }
}
