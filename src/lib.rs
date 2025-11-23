use markdown::to_html;
use rand::Rng;
use std::fs;
use std::io;
use std::path::Path;

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
    #[must_use]
    pub fn roll_mult(&self, times: u32) -> u32 {
        let mut result = 0;
        for _ in 0..times {
            result += self.roll();
        }
        result
    }
    #[must_use]
    pub fn average_over(&self, times: u32) -> u32 {
        self.roll_mult(times) / times
    }
}

pub struct Markdown {
    content: String,
}

impl Markdown {
    #[must_use]
    pub fn as_html(&self) -> String {
        to_html(&self.content)
    }
    /// # Errors
    /// Will return Err if the file content cannot be read.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self { content })
    }
}
