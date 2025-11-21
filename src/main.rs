use rand::Rng;

struct DicePool {
    amount: u32,
    sides: u32,
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
    fn roll_mult(&self, times: u32) -> u32 {
        let mut result = 0;
        for _ in 0..times {
            result += self.roll();
        }
        result
    }
    fn average_over(&self, times: u32) -> u32 {
        self.roll_mult(times) / times
    }
}

fn main() {
    let pool = DicePool {
        amount: 10,
        sides: 6,
    };
    let times = 1000;
    println!("results of rolling {}d{} {} times: {}", pool.amount, pool.sides, times, pool.roll_mult(times));
    println!("average of rolling {}d{} {} times: {}", pool.amount, pool.sides, times, pool.average_over(times));
}
