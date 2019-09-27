
use std::collections::HashMap;

use pest::Parser;

#[grammar = "../scripting.pest"]
#[derive(Parser)]
struct ScriptingParser;


pub enum Value<T> {
    // Json values
    Null,
    Bool(bool),
    Int(isize),
    Float(f32),
    String(String),
    Array(Vec<T>),
    Object(HashMap<String, T>),
    // Pseudo-json
    Opt(Option<T>),
    // Scripting values
    FnRef(String),
    Closure(Fn),
    // Ident(String),
}

pub struct Fn {
    name: String,
    args: Vec<(String, Typ)>,
    body: Vec<Expr>,
    scope: HashMap<String, Value<Expr>>
}

pub struct Block {
    fns: HashMap<String, Fn>,
    exprs: Vec<Expr>,
}

pub enum Typ {
    Int, String, Float, Bool, Json, Option(Box<Typ>),
}

pub enum Stmt {
    Expr(Expr),
    FnDefn(Fn)
}

pub enum Pattern {
    String(String),
    Bool(bool),
    Int(isize),
    Float(f32)
}

pub enum Expr {
    Value(Box<Value<Expr>>),
    Assignment(String, Box<Value<Expr>>),
    If(Box<Expr>, Block, Option<Block>),
    Match(Box<Expr>, Vec<(Pattern, Block)>),
    FnCall(String, Vec<Expr>),
    Lambda(Fn),
    Vbl(String),
}


use pest::iterators::Pair;

// fn parse_stmt(pair: Pair<Rule>) -> Block {
//     match pair.as_rule() {
//         Rule::fn_defn => 
//     }
// }

fn parse_file(pair: Pair<Rule>) -> Block {
    match pair.as_rule() {
        Rule::block => {
            let mut fns = HashMap::new();
            let mut exprs = vec![];

            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::statement => match pair.into_inner().next().unwrap() {
                        Rule::fn_defn => (),
                        Rule::assignment => (),
                    },
                    Rule::value => Stmt::Expr(parse_expr(pair))
                }
            }

            // pair.into_inner()
            // .map(|pair| {
            //     match pair.as_rule() {
            //         Rule::statement => parse_stmt(pair.into_inner().next().unwrap()),
            //         Rule::value => Stmt::Expr(parse_expr(pair))
            //     }
            // })
        },
        _ => unreachable!()
    }
}

// fn parse_value(pair: Pair<Rule>) -> JSONValue {
//     match pair.as_rule() {
//         Rule::object => JSONValue::Object(
//             pair.into_inner()
//                 .map(|pair| {
//                     let mut inner_rules = pair.into_inner();
//                     let name = inner_rules
//                         .next()
//                         .unwrap()
//                         .into_inner()
//                         .next()
//                         .unwrap()
//                         .as_str();
//                     let value = parse_value(inner_rules.next().unwrap());
//                     (name, value)
//                 })
//                 .collect(),
//         ),
//         Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
//         Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
//         Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
//         Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
//         Rule::null => JSONValue::Null,
//         Rule::json
//         | Rule::EOI
//         | Rule::pair
//         | Rule::value
//         | Rule::inner
//         | Rule::char
//         | Rule::WHITESPACE => unreachable!(),
//     }
// }



// fn parse(raw: &str) -> Result<JSONValue, Error<Rule>> {

//     Ok(parse_value(raw))
// }
