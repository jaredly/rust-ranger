use pest::iterators::Pair;
use pest::Parser;
use pest_derive::*;

use unescape;

use crate::ast::{Const, Expr, IfCond, Pattern, Statement, Type};

#[grammar = "../grammar.pest"]
#[derive(Parser)]
pub struct MainParser;

pub enum ParseError {
    Invalid,
}

pub type ParseResult<T> = Result<T, ParseError>;

fn unescape_string(string: &str) -> String {
    if &string[0..1] == "\"" {
        unescape::unescape(&string[1..string.len() - 1]).unwrap()
    } else {
        for i in 0..string.len() {
            if &string[i..=i] == "\"" {
                return string[i + 1..string.len() - i].to_string();
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
            Expr::Char(
                unescape::unescape(&str[1..str.len() - 1])
                    .unwrap()
                    .parse::<char>()
                    .unwrap(),
            )
        }
        Rule::string => Expr::String(unescape_string(pair.as_str())),
        _ => {
            panic!(format!(
                "Unreachable const {}, {:?}",
                pair.as_str(),
                pair.as_rule()
            ));
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
            _ => unreachable!(),
        },
        parse_expr(v),
    )
}

fn parse_const_const(pair: Pair<Rule>) -> Const {
    match pair.as_rule() {
        Rule::const_ => parse_const_const(pair.into_inner().next().unwrap()),
        Rule::float => Const::Float(pair.as_str().parse::<f32>().unwrap()),
        Rule::signed_int => Const::Int(pair.as_str().parse::<i32>().unwrap()),
        Rule::bool => Const::Bool(pair.as_str().parse::<bool>().unwrap()),
        Rule::char => {
            let str = pair.as_str();
            Const::Char(
                unescape::unescape(&str[1..str.len() - 1])
                    .unwrap()
                    .parse::<char>()
                    .unwrap(),
            )
        }
        Rule::string => Const::String(unescape_string(pair.as_str())),
        _ => {
            panic!(format!(
                "Unreachable const {}, {:?}",
                pair.as_str(),
                pair.as_rule()
            ));
        }
    }
}

fn parse_pattern(pattern: Pair<Rule>) -> Pattern {
    let pattern = match pattern.into_inner().next() {
        None => return Pattern::Any,
        Some(item) => item,
    };
    match pattern.as_rule() {
        Rule::const_ => Pattern::Const(parse_const_const(pattern)),
        Rule::ident => Pattern::Ident(pattern.as_str().to_owned()),
        Rule::tuple_pattern => {
            let mut inner = pattern.into_inner();
            let first = inner.next().unwrap();
            Pattern::Tuple(
                first.as_str().to_owned(),
                inner.map(parse_pattern).collect(),
            )
        }
        Rule::struct_pattern => {
            let mut inner = pattern.into_inner();
            let first = inner.next().unwrap();
            let mut items = vec![];
            loop {
                if let None = inner.peek() {
                    break;
                }
                let ident = inner.next().unwrap().as_str().to_owned();
                let pattern = if let Some(pattern) = inner.peek() {
                    if pattern.as_rule() == Rule::pattern {
                        parse_pattern(inner.next().unwrap())
                    } else {
                        Pattern::Ident(ident.clone())
                    }
                } else {
                    Pattern::Ident(ident.clone())
                };
                items.push((ident, pattern))
            }
            Pattern::Struct(first.as_str().to_owned(), items)
        }
        _ => unreachable!(),
    }
}

fn parse_if_cond(pair: Pair<Rule>) -> IfCond {
    let mut cond = pair.into_inner();
    let first = cond.next().unwrap();
    match first.as_rule() {
        Rule::value => IfCond::Value(parse_expr(first)),
        Rule::pattern => IfCond::IfLet(parse_pattern(first), parse_expr(cond.next().unwrap())),
        _ => unreachable!(),
    }
}

pub fn parse_op_item(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::cast => {
            let mut items = pair.into_inner();
            let first = parse_op_item(items.next().unwrap());
            match items.next() {
                None => first,
                Some(pair) => Expr::Cast(
                    Box::new(first),
                    match pair.as_str() {
                        "i32" => Type::I32,
                        "f32" => Type::F32,
                        _ => unreachable!(),
                    },
                ),
            }
        }

        Rule::if_chain => {
            let mut items = pair.into_inner();
            let first_cond = parse_if_cond(items.next().unwrap());
            let block = parse_expr(items.next().unwrap());
            let mut middles = vec![(first_cond, block)];
            loop {
                let first = match items.next() {
                    None => break,
                    Some(pair) => pair,
                };
                match first.as_rule() {
                    Rule::block => {
                        return Expr::IfChain(middles, Some(Box::new(parse_expr(first))))
                    }
                    Rule::if_cond => {
                        middles.push((parse_if_cond(first), parse_expr(items.next().unwrap())))
                    }
                    _ => unreachable!(),
                }
            }
            Expr::IfChain(middles, None)
        }

        Rule::match_ => {
            let mut items = pair.into_inner();
            let value = parse_expr(items.next().unwrap());
            let mut cases = vec![];
            loop {
                if let None = items.peek() {
                    break;
                }
                let pattern = parse_pattern(items.next().unwrap());
                let body = parse_expr(items.next().unwrap());
                cases.push((pattern, body))
            }
            Expr::Match(Box::new(value), cases)
        }

        Rule::subject => {
            let mut items = pair.into_inner();
            let first = parse_op_item(items.next().unwrap());
            let access: Vec<(String, Option<Vec<Expr>>)> = items
                .into_iter()
                .map(|pair| match pair.as_rule() {
                    Rule::fncall => {
                        let mut items = pair.into_inner();
                        let name = items.next().unwrap().as_str().to_string();
                        let args = items.map(parse_expr).collect();
                        (name, Some(args))
                    }
                    _ => (pair.as_str().to_owned(), None),
                })
                .collect();
            if access.is_empty() {
                first
            } else {
                Expr::MemberAccess(Box::new(first), access)
            }
        }
        Rule::object => Expr::Object(pair.into_inner().map(parse_pair).collect()),
        Rule::array => Expr::Array(pair.into_inner().map(parse_expr).collect()),
        Rule::tuple => {
            let mut items: Vec<Expr> = pair.into_inner().map(parse_expr).collect();
            if items.len() == 1 {
                items.remove(0)
            } else {
                Expr::Tuple(items)
            }
        },
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
        Rule::fncall => {
            let mut items = pair.into_inner();
            let key = items.next().unwrap().as_str().to_string();
            Expr::FnCall(key, items.map(parse_expr).collect())
        }
        _ => {
            panic!(format!(
                "Unreachable op item {}, {:?}",
                pair.as_str(),
                pair.as_rule()
            ));
        }
    }
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

pub fn parse_expr(pair: Pair<Rule>) -> Expr {
    if pair.as_rule() == Rule::block {
        return parse_block(pair);
    }
    if pair.as_rule() != Rule::value {
        panic!(
            "Invalid use of parse_expr. Must be a 'value' : {} {:?}",
            pair.as_str(),
            pair
        );
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

pub fn parse_stmt(pair: Pair<Rule>) -> Statement {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::const_binding |
        Rule::let_binding => {
            let mut items = pair.into_inner();
            let ident = items.next().unwrap().as_str().to_owned();
            let value = parse_expr(items.next().unwrap());
            Statement::Let(ident, value)
        }
        Rule::value => Statement::Expr(parse_expr(pair)),
        Rule::fndefn => {
            let mut items = pair.into_inner();
            let ident = items.next().unwrap().as_str().to_owned();
            let args = items
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| pair.as_str().to_owned())
                .collect();
            let value = parse_block(items.next().unwrap());
            Statement::FnDefn(ident, args, value)
        }
        _ => {
            panic!(format!(
                "Unreachable stmt {}, {:?}",
                pair.as_str(),
                pair.as_rule()
            ));
        }
    }
}

pub fn process_file(text: &str) -> Result<Vec<Statement>, pest::error::Error<Rule>> {
    match MainParser::parse(Rule::file, text) {
        Ok(v) => {
            let mut stmts = vec![];
            for pair in v {
                if let Rule::toplevel_statement = pair.as_rule() {
                    stmts.push(parse_stmt(pair))
                }
            }
            Ok(stmts)
        }
        Err(e) => Err(e),
    }
}

pub fn parse_block(pair: Pair<Rule>) -> Expr {
    let mut items = vec![];
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::statement => items.push(parse_stmt(item)),
            Rule::value => return Expr::Block(items, Box::new(parse_expr(item))),
            _ => (),
        }
    }
    unreachable!()
}

pub fn process_expr(text: &str) -> Result<Expr, pest::error::Error<Rule>> {
    match MainParser::parse(Rule::expr, text) {
        Ok(v) => {
            let mut items = vec![];
            for item in v {
                match item.as_rule() {
                    Rule::statement => items.push(parse_stmt(item)),
                    Rule::value => return Ok(Expr::Block(items, Box::new(parse_expr(item)))),
                    _ => (),
                }
            }
            // let mut v: Vec<Pair<_>> = v.into_iter().collect();
            // let last = v.pop().unwrap();
            // println!("Number {} {:?}", v.len(), last);
            // for item in v {
            //     items.push(parse_stmt(item));
            // }
            // Ok(Expr::Block(items, Box::new(parse_expr(last))))
            unreachable!()
        }
        Err(e) => Err(e),
    }
}
