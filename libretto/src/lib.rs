#![allow(dead_code)]
use pest::Parser;
use pest_derive::*;
use unescape;


#[grammar = "../grammar.pest"]
#[derive(Parser)]
struct MainParser;

mod ast {

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

    use super::*;
    use pest::iterators::{Pair, Pairs};
    use pest::Parser;
    use pest_derive::*;

    enum ParseError {
        Invalid,
    }

    type ParseResult<T> = Result<T, ParseError>;

    fn unescape(string: &str) -> String {
        if &string[0..1] == "\"" {
            unescape::unescape(&string[1..string.len()-1]).unwrap()
        } else {
            for i in 0..string.len() {
                if &string[i..i+1] == "\"" {
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
            Rule::string => Expr::String(unescape(pair.as_str())),
            _ => {
                panic!(format!("Unreachable const {}, {:?}", pair.as_str(), pair.as_rule()));
            }
        }
    }

    pub fn parse_op_item(pair: Pair<Rule>) -> Expr {
        match pair.as_rule() {
            Rule::object => Expr::Object(pair.into_inner().map(|pair| match pair.as_rule() {
                Rule::pair => {
                    let mut children = pair.into_inner();
                    let key = children.next().unwrap();
                    let v = children.next().unwrap();
                    (
                        match key.as_rule() {
                            Rule::string => unescape(key.as_str()),
                            Rule::ident => key.as_str().to_string(),
                            _ => unreachable!()
                        },
                        parse_expr(v)
                    )
                }
                _ => unreachable!()

            }).collect()),
            Rule::array => Expr::Array(pair.into_inner().map(parse_expr).collect()),
            Rule::const_ => parse_const(pair.into_inner().next().unwrap()),
            Rule::option => Expr::Option(Box::new(pair.into_inner().next().map(parse_expr))),
            Rule::ident => Expr::Ident(pair.as_str().to_string()),
            Rule::value => parse_expr(pair),
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
        if input.1.len() > 0 {
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
            .into_iter()
            .map(|rule| {
                let mut items = rule.into_inner();
                let op = items.next().unwrap().as_str();
                let v = parse_op_item(items.next().unwrap());
                (op, v)
            })
            .collect();
        make_op_tree((first, rest))
    }

    // pub fn parse_value(pair: Pair<Rule>) -> Expr {
    //     unimplemented!("Ok");
    // }
    pub fn process(text: &str) -> Result<Expr, pest::error::Error<Rule>> {
        match MainParser::parse(Rule::file, text) {
            Ok(v) => Ok(ast::parse(v)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{Expr};

    #[test]
    fn array() {
        assert_eq!(
            ast::process("[1,2,3]"),
            Ok(Expr::Array(vec![
                Expr::Int(1),
                Expr::Int(2),
                Expr::Int(3),
            ]))
        );
    }

    #[test]
    fn plus_minus() {
        assert_eq!(
            ast::process("1 + 2 - 3"),
            Ok(Expr::Minus(
                Box::new(Expr::Plus(
                    Box::new(Expr::Int(1)),
                    Box::new(Expr::Int(2)),
                )),
                Box::new(Expr::Int(3)),
            ))
        );

        assert_eq!(
            ast::process("1 - 2 + 3"),
            Ok(Expr::Plus(
                Box::new(Expr::Minus(
                    Box::new(Expr::Int(1)),
                    Box::new(Expr::Int(2)),
                )),
                Box::new(Expr::Int(3)),
            ))
        );

        assert_eq!(
            ast::process(r##"["o\nne", r#"t"w\no"#, 'a', '\n', "😅"]"##),
            Ok(Expr::Array(
                vec![
                    Expr::String("o\nne".to_string()),
                    Expr::String("t\"w\\no".to_string()),
                    Expr::Char('a'),
                    Expr::Char('\n'),
                    Expr::String("😅".to_string()),
                ]
            ))
        );
    }

    #[test]
    fn many_ops() {
        assert_eq!(
            ast::process("1 - 2 * 3 + 5 == 4"),
            Ok(Expr::Eq(
                Box::new(Expr::Plus(
                    Box::new(Expr::Minus(
                        Box::new(Expr::Int(1)),
                        Box::new(Expr::Times(
                            Box::new(Expr::Int(2)),
                            Box::new(Expr::Int(3))
                        ))
                    )),
                    Box::new(Expr::Int(5))
                )),
                Box::new(Expr::Int(4))
            ))
        );
    }

    #[test]
    fn complex() {
        ast::process(r###"
{
    one: 1,
    "two": 2,
    three_four: [3, 3 + 1],
    five: None,
    six: Some(6 - (3 - 2)),
    "7": true != false
}
        "###).unwrap();
    }
}
