use clap::Args;
use clap::Parser;
use clap::Subcommand;
use pinbreak::roll::DicePool;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rolls some dice
    Roll(RollArgs),
}

#[derive(Args)]
struct RollArgs {
    /// The size of the dice pool
    amount: u32,
    /// The type of dice
    sides: u32,
    /// The amount of times to repeat the roll
    times: u32,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Roll(args) => {
            let pool = DicePool {
                amount: args.amount,
                sides: args.sides,
            };
            println!(
                "results of rolling {}d{} {} times: {}",
                pool.amount,
                pool.sides,
                args.times,
                pool.roll_mult(args.times)
            );
            println!(
                "average of rolling {}d{} {} times: {}",
                pool.amount,
                pool.sides,
                args.times,
                pool.average_over(args.times)
            );
        }
    }
}
