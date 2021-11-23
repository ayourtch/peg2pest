use clap::Parser as ClapParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use pest::Parser;

use pest::prec_climber::PrecClimber;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MyParser;

/// This program converts the .peg grammars as used in https://github.com/pointlander/peg
/// into the .pest files as used by https://github.com/pest-parser/pest
///
/// Actions are not supported, all generated rules are compound-atomic
/// (you most probably will need to tweak that).
#[derive(Debug, Clone, ClapParser, Serialize, Deserialize)]
#[clap(version = env!("GIT_VERSION"), author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
struct Opts {
    /// Override options from this yaml/json file
    #[clap(short, long)]
    options_override: Option<String>,

    /// input filename to read
    #[clap(short, long)]
    input_filename: Option<String>,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn convert_identifier_name(ident: pest::iterators::Pair<Rule>) -> String {
    let mut acc: String = "".to_string();
    acc.push_str(&format!(" {}", ident.as_str()));
    acc
}

fn convert_class(class: pest::iterators::Pair<Rule>) -> String {
    let mut full_acc: String = "".to_string();
    let mut inner_pairs = class.into_inner();
    let mut suppress_bar = true;
    let mut negate_class = false;

    for inner_pair in inner_pairs {
        let mut acc: String = "".to_string();
        match &inner_pair.as_rule() {
            Rule::NegateClass => {
                negate_class = true;
                continue;
            }
            Rule::Range => {
                let s = inner_pair.as_str();
                if s.len() == 3 {
                    let parts: Vec<&str> = s.split("-").collect();
                    assert!(parts.len() == 2);
                    acc.push_str(&format!(" '{}'..'{}'", &parts[0], &parts[1]));
                } else {
                    match s {
                        "\"" => acc.push_str(&format!(" \"\\{}\"", s)),
                        "\\-" => acc.push_str(&format!(" \"-\"")),
                        x => acc.push_str(&format!(" \"{}\"", s)),
                    }
                }
            }
            Rule::DoubleRange => {
                let s = inner_pair.as_str();
                if s.len() == 3 {
                    let parts: Vec<&str> = s.split("-").collect();
                    assert!(parts.len() == 2);
                    if parts[0].to_lowercase() != parts[0].to_uppercase() {
                        acc.push_str(&format!(
                            " '{}'..'{}' | '{}'..'{}'",
                            &parts[0].to_lowercase(),
                            &parts[1].to_lowercase(),
                            &parts[0].to_uppercase(),
                            &parts[1].to_uppercase()
                        ));
                    } else {
                        acc.push_str(&format!(" '{}'..'{}'", &parts[0], &parts[1]));
                    }
                } else {
                    match s {
                        "\"" => acc.push_str(&format!(" \"\\{}\"", s)),
                        "\\-" => acc.push_str(&format!(" \"-\"")),
                        x => acc.push_str(&format!(" \"{}\"", s)),
                    }
                }
            }
            x => acc.push_str(&format!(" CRULE({:#?})", x)),
        }

        if suppress_bar {
            suppress_bar = false;
        } else {
            full_acc.push_str(" |");
        }

        full_acc.push_str(&acc);
    }
    if negate_class {
        format!("!({} ) ~ ANY", &full_acc)
    } else {
        full_acc
    }
}

fn convert_sequence(seq: pest::iterators::Pair<Rule>) -> String {
    let mut full_acc: String = "".to_string();
    let mut inner_pairs = seq.into_inner();
    let mut suppress_tilde = true;
    let mut suppress_next_tilde = false;
    let mut rule_not = false;
    let mut current_rule_not = false;

    for inner_pair in inner_pairs {
        let mut acc: String = "".to_string();
        if suppress_next_tilde {
            suppress_tilde = true;
            suppress_next_tilde = false;
        }
        current_rule_not = (&inner_pair.as_rule() == &Rule::Not);
        match &inner_pair.as_rule() {
            Rule::IdentifierName => {
                acc.push_str(&convert_identifier_name(inner_pair));
            }
            Rule::Open => {
                acc.push_str(" (");
            }
            Rule::Close => {
                acc.push_str(" )");
                suppress_tilde = true;
            }
            Rule::And => {
                acc.push_str(" &");
                suppress_next_tilde = true;
            }
            Rule::Not => {
                rule_not = true;
                suppress_next_tilde = true;
            }
            Rule::Star => {
                acc.push_str("*");
                suppress_tilde = true;
            }
            Rule::Plus => {
                acc.push_str("+");
                suppress_tilde = true;
            }
            Rule::Question => {
                acc.push_str("?");
                suppress_tilde = true;
            }
            Rule::Dot => {
                if rule_not {
                    acc.push_str(" EOI");
                    rule_not = false;
                } else {
                    acc.push_str(" ANY");
                }
            }
            Rule::SingleQLiteral => acc.push_str(&format!(
                " \"{}\"",
                inner_pair.as_str().replace("\"", "\\\"")
            )),
            Rule::DoubleQLiteral => acc.push_str(&format!(
                " ^\"{}\"",
                inner_pair.as_str().replace("\"", "\\\"")
            )),
            Rule::Action => {
                /* Actions are not supported */
                suppress_tilde = true;
            }
            Rule::Begin => { /* nothing */ }
            Rule::End => {
                /* nothing */
                suppress_tilde = true;
            }
            Rule::Class => {
                acc.push_str(" (");
                acc.push_str(&convert_class(inner_pair));
                acc.push_str(")");
            }
            Rule::Expression => {
                acc.push_str(&convert_expression(inner_pair));
                suppress_tilde = true;
            }
            x => acc.push_str(&format!(" RULE({:#?})", x)),
        }

        if !suppress_tilde {
            full_acc.push_str(" ~");
        }
        if rule_not && !current_rule_not {
            full_acc.push_str(" !");
            rule_not = false;
        }
        full_acc.push_str(&acc);
        if suppress_tilde {
            suppress_tilde = suppress_next_tilde;
        }
    }
    full_acc
}

// fn convert_expression<R: std::fmt::Debug + std::marker::Copy + std::cmp::Ord + std::hash::Hash>(expr: pest::iterators::Pair<R>) -> String {
fn convert_expression(expr: pest::iterators::Pair<Rule>) -> String {
    let mut acc: String = "".to_string();
    let mut inner_pairs = expr.into_inner();
    let mut need_bar = false;
    let mut empty_match = false;
    for inner_pair in inner_pairs {
        // acc.push_str(&format!("\n    {:#?}", &inner_pair));
        match inner_pair.as_rule() {
            Rule::Slash => need_bar = true,
            Rule::Sequence => {
                let seq_str = convert_sequence(inner_pair);
                if &seq_str != "" {
                    if need_bar {
                        acc.push_str(" |");
                    }
                    acc.push_str(&seq_str);
                } else {
                    /* there was an empty branch, we should generate code for an empty match */
                    empty_match = true;
                }
            }
            Rule::TrailingSlash => {
                empty_match = true;
            }
            _ => unreachable!(),
        }
    }
    if empty_match {
        format!(" ({} )?", &acc)
    } else {
        acc
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    // allow to load the options, so far there is no good built-in way
    let opts = if let Some(fname) = &opts.options_override {
        if let Ok(data) = std::fs::read_to_string(&fname) {
            let res = serde_json::from_str(&data);
            if res.is_ok() {
                res.unwrap()
            } else {
                serde_yaml::from_str(&data).unwrap()
            }
        } else {
            opts
        }
    } else {
        opts
    };

    if opts.verbose > 4 {
        let data = serde_json::to_string_pretty(&opts).unwrap();
        println!("{}", data);
        println!("===========");
        let data = serde_yaml::to_string(&opts).unwrap();
        println!("{}", data);
    }
    if let Some(fname) = &opts.input_filename {
        if let Ok(data) = std::fs::read_to_string(&fname) {
            match MyParser::parse(Rule::Grammar, &data) {
                Ok(okparse) => {
                    // println!("Parse: {:?}", &okparse);
                    // println!("Eval: {:?}", eval(okparse));
                    // println!("{}: Parse is ok", &fname);
                    // println!("data:\n{:#?}", okparse);
                    for pair in okparse {
                        match pair.as_rule() {
                            Rule::Definition => {
                                let mut inner_pairs = pair.into_inner();
                                let ident = inner_pairs.next().unwrap();
                                let left_arrow = inner_pairs.next().unwrap();
                                assert!(left_arrow.as_rule() == Rule::LeftArrow);
                                let expression = inner_pairs.next().unwrap();
                                println!(
                                    "{} = {}{}{}",
                                    ident.as_str(),
                                    " ${",
                                    convert_expression(expression),
                                    " }"
                                );
                            }
                            _ => { /* do nothing */ }
                        }
                    }
                    // println!("data:\n{}", okparse.as_str());
                }
                Err(e) => {
                    println!("{}: Error: {:?}", &fname, &e);
                }
            }
        }
    }
}
