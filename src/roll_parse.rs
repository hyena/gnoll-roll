use itertools::Itertools;
use rand::prelude::*;
use std::cell::Cell;
use std::collections::HashSet;

use pest::Parser;
use pest::error::Error;

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

/// Rolls a single die term, e.g. 3d20 or 5d10k3
/// TODO: Support more operands.
/// TODO: Return parse errors.
fn roll_die(term: pest::iterators::Pair<Rule>) -> (String, i64) {
    let mut rng = thread_rng();
    println!("Die Roll {:?}", term.as_str());

    let mut inner_rules = term.into_inner();  // { number ~ "d" ~ number }
    let count: i64 = inner_rules.next().unwrap().as_str().parse().unwrap();
    // TODO: if count > BIG_NUMBER return Error
    let size: i64 = inner_rules.next().unwrap().as_str().parse().unwrap();

    // Helper fn to roll one die.
    let mut roll_fn = || {
        if size > 1 {
            rng.gen_range(1, size)
        } else {
            size
        }
    };

    // Figure out the result rolls.
    // What we do from here is based on our 'mode' given by an optional suffix.
    let rolls: Vec<RollEntry> = if let Some(suffix) = inner_rules.next() {
        match suffix.as_rule() {
            Rule::keep => {
                let keep_low = suffix.as_str().starts_with("kl");
                // Grab the count of the dice to keep.
                let keep_count: usize = suffix.into_inner().next().unwrap().as_str().parse().unwrap();
                let rolls: Vec<i64> = (0..count).map(|_| { roll_fn() }).collect();

                // Find the k smallest or largest elements to keep.
                let keepers: HashSet<usize> =
                    rolls.iter()
                    .enumerate()
                    .sorted_by(|a, b| { 
                        if keep_low { Ord::cmp(&a.1, &b.1) } else { Ord::cmp(&a.1, &b.1).reverse() }
                    })
                    .map(|a| a.0)
                    .take(keep_count)
                    .collect();
        
                rolls.into_iter().enumerate().map(|(index, value)| {
                    if keepers.contains(&index) { 
                        RollEntry::Normal(value) 
                    } else {
                        RollEntry::Discard(value) 
                    }
                }).collect()
            }
            // Future affixes wll go here.
            _ => unreachable!()
        }
    } else {
        // Normall die roll.
        (0..count).map(|_| { RollEntry::Normal(roll_fn()) }).collect()
    };

    // Convert the vector of rolls into a string and a total.
    let total = rolls.iter().fold(0, |total: i64, roll: &RollEntry| {
        match roll {
            RollEntry::Normal(value) => total + value,
            RollEntry::Discard(_) => total,
        }
    });
    let roll_string: String = rolls.into_iter()
        .map(|roll: RollEntry| {
            match roll {
                RollEntry::Normal(value) => value.to_string(),
                RollEntry::Discard(value) => format!("~~{}~~", value.to_string())
            }
        })
        .join("+");
    (format!("({})", roll_string), total)
}

pub fn parse_roll(file: &str) -> Result<String, Error<Rule>> {
    let roll = GnollRollParser::parse(Rule::roll, file)?.next().unwrap();
    println!("Roll: {:?}", roll.as_str());

    let mut result_string = String::new();
    let mut result_total: i64 = 0;
    let mode: Cell<Option<Rule>> = Cell::new(None);
    let mut comment: Option<&str> = None;

    let do_math = |a: i64, b: i64| {
        match mode.get() {
            None => {
                assert_eq!(a, 0);
                b
            }
            Some(Rule::add) => a + b,
            Some(Rule::subtract) => a - b,
            Some(Rule::multiply) => a * b,
            Some(Rule::divide) => a / b,
            _ => unreachable!()
        }
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
            _ => unreachable!()
        }
    };
    result_string.push_str(&format!(" = {}", result_total));
    if let Some(c) = comment {
        result_string.push_str("  # ");
        result_string.push_str(c);
    }
    println!("{}", result_string);
    Ok(result_string)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_roll_0() {
        parse_roll("1d0").unwrap();
//        assert_eq!(0, )
    }
    #[test]
    fn test_add() {
        parse_roll("5 + 7").unwrap();
    }

    #[test]
    fn test_simple_roll() {
        parse_roll("5d6").unwrap();
    }

    #[test]
    fn test_roll_with_math() {
        parse_roll("1d6 - 7").unwrap();
    }

    #[test]
    fn test_comment() {
        parse_roll("5d20 + 17  # Seduce the dragon").unwrap();
    }

    #[test]
    fn test_leading_ws_comment() {
        parse_roll("1d20 + 1#     Seduce the dragon").unwrap();
    }

    #[test]
    fn test_bad_parse() {
        assert!(parse_roll("spaghetti").is_err());
    }

    #[test]
    fn test_keep_high() {
        parse_roll("2d20k1").unwrap();
    }

    #[test]
    fn test_keep_low() {
        parse_roll("2d20 kl 1").unwrap();
    }
}
