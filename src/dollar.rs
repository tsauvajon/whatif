//! Human readable dollar amounts
use std::fmt::Display;

#[derive(Copy, Clone)]
pub struct DollarAmount {
    dollars: u64,
}

impl DollarAmount {
    pub fn dollars(self) -> u64 {
        self.dollars
    }
}

impl From<i32> for DollarAmount {
    fn from(dollars: i32) -> Self {
        DollarAmount {
            dollars: dollars as u64,
        }
    }
}

impl From<u32> for DollarAmount {
    fn from(dollars: u32) -> Self {
        DollarAmount {
            dollars: dollars as u64,
        }
    }
}

impl From<u64> for DollarAmount {
    fn from(dollars: u64) -> Self {
        DollarAmount { dollars }
    }
}

impl From<f64> for DollarAmount {
    fn from(dollars: f64) -> Self {
        DollarAmount {
            dollars: dollars as u64,
        }
    }
}

fn separate_thousands(amount: u64) -> Vec<u64> {
    let mut remainder = amount;
    let mut acc = vec![];
    while remainder > 0 {
        acc.push(remainder % 1000);
        remainder = remainder / 1000;
    }

    acc.reverse();
    acc
}

impl Display for DollarAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = separate_thousands(self.dollars)
            .iter()
            .map(|v| format!("{v:03}"))
            .collect::<Vec<String>>()
            .join(",");
        let dollars = dollars.trim_start_matches("0");

        write!(f, "${dollars}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_thousands() {
        assert_eq!(
            separate_thousands(123_456_789_000),
            vec![123, 456, 789, 000]
        );

        assert_eq!(separate_thousands(123_456_789), vec![123, 456, 789]);
        assert_eq!(separate_thousands(123_456), vec![123, 456]);
        assert_eq!(separate_thousands(12_345), vec![12, 345]);
        assert_eq!(separate_thousands(1_234), vec![1, 234]);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            DollarAmount::from(123_456_789_000u64).to_string(),
            "$123,456,789,000"
        );
        assert_eq!(
            DollarAmount::from(1_100_000_000).to_string(),
            "$1,100,000,000"
        );
        assert_eq!(DollarAmount::from(123_456_789).to_string(), "$123,456,789");
        assert_eq!(DollarAmount::from(100_000_000).to_string(), "$100,000,000");
        assert_eq!(DollarAmount::from(10_000_000).to_string(), "$10,000,000");
        assert_eq!(DollarAmount::from(1_000_000).to_string(), "$1,000,000");
        assert_eq!(DollarAmount::from(999_999).to_string(), "$999,999");
        assert_eq!(DollarAmount::from(100_000).to_string(), "$100,000");
        assert_eq!(DollarAmount::from(10_000).to_string(), "$10,000");
        assert_eq!(DollarAmount::from(2_000).to_string(), "$2,000");
        assert_eq!(DollarAmount::from(100).to_string(), "$100");
        assert_eq!(DollarAmount::from(10).to_string(), "$10");
        assert_eq!(DollarAmount::from(1).to_string(), "$1");
    }
}
