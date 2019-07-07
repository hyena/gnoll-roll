use itertools::Itertools;
use rand::prelude::*;
use std::cell::Cell;
use std::collections::HashSet;

use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "gnoll_roll.pest"]
struct GnollRollParser;

/// Keeps track of the status of individual rolls for their display and calculation.
enum RollEntry {
    Normal(i64),
    Discard(i64),
    // TODO: Pending implementation of other affixes.
    //    Failure(i64),
    //    Reroll(i64),
}

/// Trait we use for rolling dice based on an RNG so we can keep it testable
trait Roller {
    fn roll(&mut self, sides: u64) -> u64;
}

#[cfg(not(test))]
impl Roller for ThreadRng {
    fn roll(&mut self, sides: u64) -> u64 {
        if sides > 1 {
            self.gen_range(1, sides) as u64
        } else {
            sides
        }
    }
}

/// Rolls a single die term, e.g. 3d20 or 5d10k3
/// TODO: Support more operands.
/// TODO: Return parse errors.
fn roll_die(term: pest::iterators::Pair<Rule>) -> (String, i64) {
    println!("Die Roll {:?}", term.as_str());

    let mut rng = thread_rng();
    let mut inner_rules = term.into_inner(); // { number ~ "d" ~ number }
    let count: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();
    // TODO: if count > BIG_NUMBER return Error
    let size: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();

    // Figure out the result rolls.
    // What we do from here is based on our 'mode' given by an optional suffix.
    let rolls: Vec<RollEntry> = if let Some(suffix) = inner_rules.next() {
        match suffix.as_rule() {
            Rule::keep => {
                let keep_low = suffix.as_str().starts_with("kl");
                // Grab the count of the dice to keep.
                let keep_count: usize = suffix
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
                let rolls: Vec<i64> = (0..count).map(|_| rng.roll(size) as i64).collect();

                // Find the k smallest or largest elements to keep.
                let keepers: HashSet<usize> = rolls
                    .iter()
                    .enumerate()
                    .sorted_by(|a, b| {
                        if keep_low {
                            Ord::cmp(&a.1, &b.1)
                        } else {
                            Ord::cmp(&a.1, &b.1).reverse()
                        }
                    })
                    .map(|a| a.0)
                    .take(keep_count)
                    .collect();

                rolls
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        if keepers.contains(&index) {
                            RollEntry::Normal(value)
                        } else {
                            RollEntry::Discard(value)
                        }
                    })
                    .collect()
            }
            Rule::reroll => {
                let reroll_number: i64 = suffix
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();

                (0..count).flat_map(|_| {
                    let v = rng.roll(size) as i64;
                    match v {
                        _ if v == reroll_number => vec![RollEntry::Discard(reroll_number as i64), RollEntry::Normal(rng.roll(size) as i64)],
                        _ => vec![RollEntry::Normal(v)]
                    }
                }).collect()
            }
            // Future affixes will go here.
            _ => unreachable!(),
        }
    } else {
        // Normall die roll.
        (0..count).map(|_| RollEntry::Normal(rng.roll(size) as i64)).collect()
    };

    // Convert the vector of rolls into a string and a total.
    let total = rolls
        .iter()
        .fold(0, |total: i64, roll: &RollEntry| match roll {
            RollEntry::Normal(value) => total + value,
            RollEntry::Discard(_) => total,
        });
    let roll_string: String = rolls
        .into_iter()
        .map(|roll: RollEntry| match roll {
            RollEntry::Normal(value) => value.to_string(),
            RollEntry::Discard(value) => format!("~~{}~~", value.to_string()),
        })
        .join("+");
    (format!("({})", roll_string), total)
}

pub fn parse_roll(die_str: &str) -> Result<(String, i64), Error<Rule>> {
    let roll = GnollRollParser::parse(Rule::roll, die_str)?.next().unwrap();
    println!("Roll: {:?}", roll.as_str());

    let mut result_string = String::new();
    let mut result_total: i64 = 0;
    let mode: Cell<Option<Rule>> = Cell::new(None);
    let mut comment: Option<&str> = None;

    let do_math = |a: i64, b: i64| match mode.get() {
        None => {
            assert_eq!(a, 0);
            b
        }
        Some(Rule::add) => a + b,
        Some(Rule::subtract) => a - b,
        Some(Rule::multiply) => a * b,
        Some(Rule::divide) => a / b,
        _ => unreachable!(),
    };

    for part in roll.into_inner() {
        match part.as_rule() {
            Rule::comment => {
                comment = Some(part.into_inner().next().unwrap().as_str());
            }
            Rule::number => {
                result_string.push_str(part.as_str());
                result_total = do_math(result_total, part.as_str().parse().unwrap());
            }
            Rule::die_roll => {
                let value = roll_die(part);
                result_string.push_str(&value.0);
                result_total = do_math(result_total, value.1);
            }
            Rule::add | Rule::subtract | Rule::multiply | Rule::divide => {
                result_string.push_str(" ");
                result_string.push_str(part.as_str());
                result_string.push_str(" ");

                mode.set(Some(part.as_rule()));
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    result_string.push_str(&format!(" = {}", result_total));
    if let Some(c) = comment {
        result_string.push_str("  # ");
        result_string.push_str(c);
    }
    println!("{}", result_string);
    Ok((result_string, result_total))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::cell::RefCell;

    // Thread-local space to store dice rolls and the rule for returning them.
    thread_local! {
        static FUTURE_ROLLS: RefCell<Vec<u64>> = RefCell::new(vec![]);
    }

    impl Roller for ThreadRng {
        fn roll(&mut self, _: u64) -> u64 {
            FUTURE_ROLLS.with(|v| {
                v.borrow_mut().pop().unwrap()
            })
        }
    }

    /// Convenience function that rolls parses a string, rolls it with the given values, and returns the value.
    fn roll(die_str: &str, dice_rolls: &[u64]) -> i64 {
        FUTURE_ROLLS.with(|v| {
            v.replace(dice_rolls.to_vec())
        });
        parse_roll(die_str).unwrap().1
    }

    #[test]
    fn test_roll_0() {
        assert_eq!(roll("1d0", &[0]), 0);
    }

    #[test]
    fn test_add() {
        assert_eq!(roll("5 + 7", &[]), 12);
    }

    #[test]
    fn test_simple_roll() {
        assert_eq!(roll("5d6", &[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_roll_with_math() {
        assert_eq!(roll("1d6 - 7", &[1]), -6);
    }

    #[test]
    fn test_comment() {
        assert_eq!(roll("1d20 + 17  # Seduce the dragon", &[3]), 20);
    }

    #[test]
    fn test_leading_ws_comment() {
        assert_eq!(roll("1d20 +17#     Seduce the dragon", &[3]), 20);
    }

    #[test]
    fn test_bad_parse() {
        assert!(parse_roll("spaghetti").is_err());
    }

    #[test]
    fn test_keep_high() {
        assert_eq!(roll("2d20k1", &[17, 5]), 17);
    }

    #[test]
    fn test_keep_low() {
        assert_eq!(roll("2d20 kl 1", &[17, 5]), 5);
    }

    #[test]
    fn test_reroll() {
        assert_eq!(roll("3d10r3", &[7, 1, 3, 5]), 13);
    }
}
