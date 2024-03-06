use std::fmt::Display;

/// Minimum amount of sats to display the ₿ symbol
const B_DISPLAY_THRESHOLD: u64 = 1_000_000;
const SATS_IN_BTC: u64 = 100_000_000;

pub struct BitcoinAmount {
    sats: u64,
}

// TODO:
// https://bitcoin.design/guide/designing-products/units-and-symbols/
// Separate alternate coloring, use a monospace font
impl BitcoinAmount {
    pub fn from(sats: u64) -> Self {
        Self { sats }
    }

    pub fn sats(&self) -> u64 {
        self.sats
    }

    fn split_btc_sats(&self) -> (u64, u64) {
        let sats = self.sats % SATS_IN_BTC;
        let remaining_btc = self.sats / SATS_IN_BTC;

        (remaining_btc, sats)
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

/// Under 1M sats -> sats only.
/// Else: ₿ with sats thousands.
impl Display for BitcoinAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sats < B_DISPLAY_THRESHOLD {
            let sats = separate_thousands(self.sats)
                .iter()
                .map(|v| format!("{v:03}"))
                .collect::<Vec<String>>()
                .join(" ");
            let sats = sats.trim_start_matches("0");

            write!(f, "{sats} sats")
        } else {
            let (btc, sats) = self.split_btc_sats();

            let btc_thousands = separate_thousands(btc);
            let btc = match btc_thousands.len() {
                0 => 0.to_string(),
                1 => btc_thousands[0].to_string(),
                _ => {
                    // Don't pad the first thousands block
                    let (head, tail) = btc_thousands.split_at(1);
                    head.iter()
                        .map(ToString::to_string)
                        .chain(tail.iter().map(|thousand| format!("{thousand:03}")))
                        .collect::<Vec<String>>()
                        .join(" ")
                }
            };

            // Example: 10 000 000
            let mut sats = format!("{sats:08}");
            sats.insert(5, ' ');
            sats.insert(2, ' ');
            write!(f, "₿{btc}.{sats}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btc_sats_split() {
        // Both
        let bitcoin_amount = BitcoinAmount {
            sats: 123_456_789_000,
        };

        let (btc, sats) = bitcoin_amount.split_btc_sats();
        assert_eq!(btc, 1_234);
        assert_eq!(sats, 56_789_000);

        // Only sats
        let bitcoin_amount = BitcoinAmount { sats: 123_456 };

        let (btc, sats) = bitcoin_amount.split_btc_sats();
        assert_eq!(btc, 0);
        assert_eq!(sats, 123_456);

        // Only BTC
        let bitcoin_amount = BitcoinAmount {
            sats: 123_456_000_000_000,
        };

        let (btc, sats) = bitcoin_amount.split_btc_sats();
        assert_eq!(btc, 1_234_560);
        assert_eq!(sats, 0);
    }

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
            BitcoinAmount::from(123_456_789_000).to_string(),
            "₿1 234.56 789 000"
        );
        assert_eq!(
            BitcoinAmount::from(123_456_789).to_string(),
            "₿1.23 456 789"
        );
        assert_eq!(
            BitcoinAmount::from(1_100_000_000).to_string(),
            "₿11.00 000 000"
        );
        assert_eq!(
            BitcoinAmount::from(100_000_000).to_string(),
            "₿1.00 000 000"
        );

        assert_eq!(BitcoinAmount::from(10_000_000).to_string(), "₿0.10 000 000");
        assert_eq!(BitcoinAmount::from(1_000_000).to_string(), "₿0.01 000 000");
        assert_eq!(BitcoinAmount::from(999_999).to_string(), "999 999 sats");
        assert_eq!(BitcoinAmount::from(100_000).to_string(), "100 000 sats");
        assert_eq!(BitcoinAmount::from(10_000).to_string(), "10 000 sats");
        assert_eq!(BitcoinAmount::from(2_000).to_string(), "2 000 sats");
    }
}
