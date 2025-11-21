use pinbreak::roll::DicePool;

fn main() {
    let pool = DicePool {
        amount: 10,
        sides: 6,
    };
    let times = 1000;
    println!("results of rolling {}d{} {} times: {}", pool.amount, pool.sides, times, pool.roll_mult(times));
    println!("average of rolling {}d{} {} times: {}", pool.amount, pool.sides, times, pool.average_over(times));
}
