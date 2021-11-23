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

fn convert_sequence(seq: pest::iterators::Pair<Rule>) -> String {
    let mut acc: String = "".to_string();
    let mut inner_pairs = seq.into_inner();
    for inner_pair in inner_pairs {
        acc.push_str(&format!(" {:#?}", &inner_pair.as_rule()));
    }
    acc
}

// fn convert_expression<R: std::fmt::Debug + std::marker::Copy + std::cmp::Ord + std::hash::Hash>(expr: pest::iterators::Pair<R>) -> String {
fn convert_expression(expr: pest::iterators::Pair<Rule>) -> String {
    let mut acc: String = "".to_string();
    let mut inner_pairs = expr.into_inner();
    for inner_pair in inner_pairs {
        // acc.push_str(&format!("\n    {:#?}", &inner_pair));
        match inner_pair.as_rule() {
            Rule::Slash => acc.push_str(" | "),
            Rule::Sequence => acc.push_str(&convert_sequence(inner_pair)),
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
                                    "${ ",
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
