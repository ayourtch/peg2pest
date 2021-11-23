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

/// This program does something useful, but its author needs to edit this.
/// Else it will be just hanging around forever
#[derive(Debug, Clone, ClapParser, Serialize, Deserialize)]
#[clap(version = env!("GIT_VERSION"), author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
struct Opts {
    /// Target hostname to do things on
    #[clap(short, long, default_value = "localhost")]
    target_host: String,

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

fn convert_sequence(seq: pest::iterators::Pair<Rule>) -> String {
    let mut full_acc: String = "".to_string();
    let mut inner_pairs = seq.into_inner();
    let mut suppress_tilde = true;
    let mut suppress_next_tilde = false;
    let mut rule_not = false;

    for inner_pair in inner_pairs {
        let mut acc: String = "".to_string();
        if suppress_next_tilde {
            suppress_tilde = true;
            suppress_next_tilde = false;
        }
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
                    acc.push_str("EOI");
                    rule_not = false;
                } else {
                    acc.push_str(".");
                }
            }
            Rule::SingleQLiteral => acc.push_str(&format!(" \"{}\"", inner_pair.as_str())),
            Rule::DoubleQLiteral => acc.push_str(&format!(" ^\"{}\"", inner_pair.as_str())),
            Rule::Action => {
                /* Actions are not supported */
                suppress_tilde = true;
            }
            Rule::Begin => { /* nothing */ }
            Rule::End => {
                /* nothing */
                suppress_tilde = true;
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
        if rule_not {
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
    for inner_pair in inner_pairs {
        // acc.push_str(&format!("\n    {:#?}", &inner_pair));
        match inner_pair.as_rule() {
            Rule::Slash => acc.push_str(" |"),
            Rule::Sequence => acc.push_str(&convert_sequence(inner_pair)),
            Rule::TrailingSlash => acc.push_str(" | EOI "),
            _ => unreachable!(),
        }
    }
    acc
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
                    println!("{}: Parse is ok", &fname);
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
