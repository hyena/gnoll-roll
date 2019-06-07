use rand::prelude::*;
use std::cell::Cell;

use pest::Parser;
use pest::error::Error;

#[derive(Parser)]
#[grammar = "gnoll_roll.pest"]
struct GnollRollParser;

/// Keeps track of the status of individual rolls for their display
enum RollEntry {
    Normal(i64),
    Failure(i64),
    Success(i64),
    Reroll(i64),
}

/// Rolls a single die term, e.g. 3d20 or 5d10k3
/// TODO: Support more operands.
fn roll_die(term: pest::iterators::Pair<Rule>) -> (String, i64) {
    let mut rng = thread_rng();
    println!("Die Roll {:?}", term.as_str());

    let mut inner_rules = term.into_inner();  // { number ~ "d" ~ number }
    let count: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();
    // TODO: if count > BIG_NUMBER return Error
    let size: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();

    // Helper fn to roll one die.
    let mut roll_fn = || {
        if size > 0 {
            rng.gen_range(1, size)
        } else {
            0
        }
    };

    // What we do from here is based on our 'mode' given by an optional suffix.
    if let Some(rule) = inner_rules.next() {
        match rule.as_rule() {
            Rule::keep => {
                let keep_low = rule.as_str().starts_with("kl");
                let rolls: Vec<u64> = (0..count).map(|_| { roll_fn() }).collect();
                
            }
        }
    }
    let rolls: Vec<u64> = (0..count).map(|_| { if size > 0 { rng.gen_range(1, size) } else { 0 }}).collect();

    
    let total: u64 = rolls.iter().sum();
    let roll_string: String = format!("({})", rolls.into_iter().map(|i| i.to_string()).collect::<Vec<String>>().join("+"));
    (roll_string, total as i64)
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
}
