use pest::Parser;
use pest_derive::*;
use unescape;
use serde_json;

use pest::iterators::{Pair, Pairs};

#[grammar = "../grammar.pest"]
#[derive(Parser)]
pub struct MainParser;

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Float(f32),
    Int(i32),
    Bool(bool),
    Char(char),
    String(String),

    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Option(Box<Option<Expr>>),
    Ident(String),

    Unit,
    Struct(String, Vec<(String, Expr)>),
    NamedTuple(String, Vec<Expr>),

    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Times(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    // todo fncall, etc.
}

type Scope = std::collections::HashMap<String, Expr>;

enum EvalError {
    InvalidType
}

impl Expr {
    fn eval(self, scope: &Scope) -> Result<serde_json::Value, EvalError> {
        use serde_json::Value;
        match self {
            Expr::Float(f) => Ok(Value::Number(serde_json::Number::from_f64(f64::from(f)).unwrap())),
            Expr::Int(f) => Ok(Value::Number(f.into())),
            Expr::Bool(f) => Ok(Value::Bool(f)),
            Expr::String(f) => Ok(Value::String(f)),
            Expr::Char(c) => Ok(Value::String(c.to_string())),
            Expr::Array(items) => {
                let mut res = vec![];
                for item in items {
                    res.push(item.eval(scope)?);
                }
                Ok(Value::Array(res))
            },
            Expr::Object(items) => {
                let mut res = serde_json::Map::new();
                for (key, value) in items {
                    res.insert(key, value.eval(scope)?);
                }
                Ok(Value::Object(res))
            },
            _ => unimplemented!()
        }
    }
}

enum ParseError {
    Invalid,
}

type ParseResult<T> = Result<T, ParseError>;

fn unescape_string(string: &str) -> String {
    if &string[0..1] == "\"" {
        unescape::unescape(&string[1..string.len()-1]).unwrap()
    } else {
        for i in 0..string.len() {
            if &string[i..=i] == "\"" {
                return string[i+1..string.len() - i].to_string();
            }
        }
        panic!("Unterminated raw string")
    }
}

pub fn parse_const(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::float => Expr::Float(pair.as_str().parse::<f32>().unwrap()),
        Rule::signed_int => Expr::Int(pair.as_str().parse::<i32>().unwrap()),
        Rule::bool => Expr::Bool(pair.as_str().parse::<bool>().unwrap()),
        Rule::char => {
            let str = pair.as_str();
            Expr::Char(unescape::unescape(&str[1..str.len()-1]).unwrap().parse::<char>().unwrap())
        },
        Rule::string => Expr::String(unescape_string(pair.as_str())),
        _ => {
            panic!(format!("Unreachable const {}, {:?}", pair.as_str(), pair.as_rule()));
        }
    }
}

fn parse_pair(pair: Pair<Rule>) -> (String, Expr) {
    let mut children = pair.into_inner();
    let key = children.next().unwrap();
    let v = children.next().unwrap();
    (
        match key.as_rule() {
            Rule::string => unescape_string(key.as_str()),
            Rule::ident => key.as_str().to_string(),
            _ => unreachable!()
        },
        parse_expr(v)
    )
}

pub fn parse_op_item(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::object => Expr::Object(pair.into_inner().map(parse_pair).collect()),
        Rule::array => Expr::Array(pair.into_inner().map(parse_expr).collect()),
        Rule::const_ => parse_const(pair.into_inner().next().unwrap()),
        Rule::option => Expr::Option(Box::new(pair.into_inner().next().map(parse_expr))),
        Rule::ident => Expr::Ident(pair.as_str().to_string()),
        Rule::upper_ident => Expr::NamedTuple(pair.as_str().to_string(), vec![]),
        Rule::value => parse_expr(pair),
        Rule::unit => Expr::Unit,
        Rule::struct_ => {
            let mut items = pair.into_inner();
            let key = items.next().unwrap().as_str().to_string();
            Expr::Struct(key, items.map(parse_pair).collect())
        }
        Rule::named_tuple => {
            let mut items = pair.into_inner();
            let key = items.next().unwrap().as_str().to_string();
            Expr::NamedTuple(key, items.map(parse_expr).collect())
        }
        _ => {
            panic!(format!("Unreachable op item {}, {:?}", pair.as_str(), pair.as_rule()));
        }
    }
}

pub fn parse(mut pairs: Pairs<Rule>) -> Expr {
    parse_expr(pairs.next().unwrap())
}

macro_rules! make_ops {
    ($current: ident, $next: ident; $( $op: expr, $constr: path );*) => {
        fn $current(input: (Expr, Vec<(&str, Expr)>)) -> Expr {
            let (first, items) = input;
            let ln = items.len();
            for i in 0..ln {
                let i = ln-1-i;
                $(
                    if items[i].0 == $op {
                        let (left, right) = items.split_at(i);
                        let mut right = right.to_vec();
                        let (_op, expr) = right.remove(0);
                        return $constr(
                            Box::new($current((first, left.to_vec()))),
                            Box::new($next((expr, right.to_vec()))),
                        );
                    }
                )*
            }
            $next((first, items))
        }

    };
}

make_ops!(make_op_tree, make_op_2; "==", Expr::Eq; "!=", Expr::Neq; "<", Expr::Lt; ">", Expr::Gt);
make_ops!(make_op_2, make_op_3; "-", Expr::Minus; "+", Expr::Plus);
make_ops!(make_op_3, make_op_4; "*", Expr::Times; "/", Expr::Divide);

fn make_op_4(input: (Expr, Vec<(&str, Expr)>)) -> Expr {
    if !input.1.is_empty() {
        panic!("Invalid binop tree, there are none left");
    }
    input.0
}

// fn make_op_tree(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
//     let (first, items) = input;
//     // let mut left = vec![];
//     // let mut pos = None;
//     for i in (0..items.len()) {
//         if items[i].0 == "==" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Eq(
//                 Box::new(make_op_2((first, left.to_vec()))),
//                 Box::new(make_op_tree((expr, right.to_vec()))),
//             );
//         }
//         if items[i].0 == "!=" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Neq(
//                 Box::new(make_op_2((first, left.to_vec()))),
//                 Box::new(make_op_tree((expr, right.to_vec()))),
//             );
//         }
//         if items[i].0 == "<" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Lt(
//                 Box::new(make_op_2((first, left.to_vec()))),
//                 Box::new(make_op_tree((expr, right.to_vec()))),
//             );
//         }
//         if items[i].0 == ">" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Gt(
//                 Box::new(make_op_2((first, left.to_vec()))),
//                 Box::new(make_op_tree((expr, right.to_vec()))),
//             );
//         }
//     }
//     make_op_2((first, items))
// }

// fn make_op_2(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
//     let (first, items) = input;
//     // let mut left = vec![];
//     // let mut pos = None;
//     for i in (0..items.len()) {
//         if items[i].0 == "-" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Minus(
//                 Box::new(make_op_3((first, left.to_vec()))),
//                 Box::new(make_op_2((expr, right.to_vec()))),
//             );
//         }
//         if items[i].0 == "+" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Plus(
//                 Box::new(make_op_3((first, left.to_vec()))),
//                 Box::new(make_op_2((expr, right.to_vec()))),
//             );
//         }
//     }
//     make_op_3((first, items))
// }

// fn make_op_3(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
//     let (first, items) = input;
//     // let mut left = vec![];
//     // let mut pos = None;
//     for i in (0..items.len()) {
//         if items[i].0 == "/" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Divide(
//                 Box::new(make_op_4((first, left.to_vec()))),
//                 Box::new(make_op_3((expr, right.to_vec()))),
//             );
//         }
//         if items[i].0 == "*" {
//             let (left, right) = items.split_at(i);
//             let mut right = right.to_vec();
//             let (op, expr) = right.remove(0);
//             return Expr::Times(
//                 Box::new(make_op_4((first, left.to_vec()))),
//                 Box::new(make_op_3((expr, right.to_vec()))),
//             );
//         }
//     }
//     make_op_4((first, items))
// }

pub fn parse_expr(pair: Pair<Rule>) -> Expr {
    if pair.as_rule() != Rule::value {
        panic!("Invalid use of parse_expr. Must be a 'value'");
    }
    let mut items = pair.into_inner();
    let first = parse_op_item(items.next().unwrap());
    let rest = items
        .map(|rule| {
            let mut items = rule.into_inner();
            let op = items.next().unwrap().as_str();
            let v = parse_op_item(items.next().unwrap());
            (op, v)
        })
        .collect();
    make_op_tree((first, rest))
}

pub fn process(text: &str) -> Result<Expr, pest::error::Error<Rule>> {
    match MainParser::parse(Rule::file, text) {
        Ok(v) => Ok(parse(v)),
        Err(e) => Err(e),
    }
}
