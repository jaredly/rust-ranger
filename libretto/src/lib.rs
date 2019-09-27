use pest::Parser;
use pest_derive::*;

#[grammar = "../grammar.pest"]
#[derive(Parser)]
struct MainParser;

mod ast {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Value<Expr> {
        Float(f32),
        Int(i32),
        Bool(bool),
        Char(char),
        String(String),

        Array(Vec<Expr>),
        Object(Vec<(String, Expr)>),
        Option(Option<Box<Expr>>),
    }
    #[derive(PartialEq, Debug, Clone)]
    pub enum Expr {
        Value(Value<Expr>),

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

    pub fn parse_const(pair: Pair<Rule>) -> Value<Expr> {
        match pair.as_rule() {
            Rule::float => Value::Float(pair.as_str().parse::<f32>().unwrap()),
            Rule::signed_int => Value::Int(pair.as_str().parse::<i32>().unwrap()),
            _ => unimplemented!(),
        }
    }

    pub fn parse_op_item(pair: Pair<Rule>) -> Expr {
        match pair.as_rule() {
            Rule::object => unimplemented!(),
            Rule::array => Expr::Value(Value::Array(pair.into_inner().map(parse_expr).collect())),
            Rule::const_ => Expr::Value(parse_const(pair.into_inner().next().unwrap())),
            Rule::option => unimplemented!(),
            Rule::ident => unimplemented!(),
            _ => {
                panic!(format!("{}, {:?}", pair.as_str(), pair.as_rule()));
            }
        }
    }

    pub fn parse(mut pairs: Pairs<Rule>) -> Expr {
        parse_expr(pairs.next().unwrap())
    }

    fn make_op_4(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
        if input.1.len() > 0 {
            panic!("Invalid binop tree, there are none left");
        }
        input.0
    }

    fn make_op_3(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
        let (first, items) = input;
        // let mut left = vec![];
        // let mut pos = None;
        for i in (0..items.len()) {
            if items[i].0 == "==" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Eq(
                    Box::new(make_op_4((first, left.to_vec()))),
                    Box::new(make_op_3((expr, right.to_vec()))),
                );
            }
            if items[i].0 == "!=" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Neq(
                    Box::new(make_op_4((first, left.to_vec()))),
                    Box::new(make_op_3((expr, right.to_vec()))),
                );
            }
            if items[i].0 == "<" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Lt(
                    Box::new(make_op_4((first, left.to_vec()))),
                    Box::new(make_op_3((expr, right.to_vec()))),
                );
            }
            if items[i].0 == ">" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Gt(
                    Box::new(make_op_4((first, left.to_vec()))),
                    Box::new(make_op_3((expr, right.to_vec()))),
                );
            }
        }
        make_op_4((first, items))
    }

    fn make_op_2(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
        let (first, items) = input;
        // let mut left = vec![];
        // let mut pos = None;
        for i in (0..items.len()) {
            if items[i].0 == "-" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Minus(
                    Box::new(make_op_3((first, left.to_vec()))),
                    Box::new(make_op_2((expr, right.to_vec()))),
                );
            }
            if items[i].0 == "+" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Plus(
                    Box::new(make_op_3((first, left.to_vec()))),
                    Box::new(make_op_2((expr, right.to_vec()))),
                );
            }
        }
        make_op_3((first, items))
    }

    fn make_op_tree(mut input: (Expr, Vec<(&str, Expr)>)) -> Expr {
        let (first, items) = input;
        // let mut left = vec![];
        // let mut pos = None;
        for i in (0..items.len()) {
            if items[i].0 == "/" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Divide(
                    Box::new(make_op_2((first, left.to_vec()))),
                    Box::new(make_op_tree((expr, right.to_vec()))),
                );
            }
            if items[i].0 == "*" {
                let (left, right) = items.split_at(i);
                let mut right = right.to_vec();
                let (op, expr) = right.remove(0);
                return Expr::Times(
                    Box::new(make_op_2((first, left.to_vec()))),
                    Box::new(make_op_tree((expr, right.to_vec()))),
                );
            }
        }
        make_op_2((first, items))
    }

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

    pub fn parse_value(pair: Pair<Rule>) -> Value<Expr> {
        unimplemented!("Ok");
    }
    pub fn process(text: &str) -> Result<Expr, pest::error::Error<Rule>> {
        match MainParser::parse(Rule::value, text) {
            Ok(v) => Ok(ast::parse(v)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        use ast::{Expr, Value};

        assert_eq!(
            ast::process("[1,2,3]"),
            Ok(Expr::Value(Value::Array(vec![
                Expr::Value(Value::Int(1)),
                Expr::Value(Value::Int(2)),
                Expr::Value(Value::Int(3)),
            ])))
        );

        assert_eq!(
            ast::process("1 + 2"),
            Ok(Expr::Plus(
                Box::new(Expr::Value(Value::Int(1))),
                Box::new(Expr::Value(Value::Int(2)))
            ))
        );

        assert_eq!(
            ast::process("1 + 2 * 3 == 4"),
            Ok(Expr::Eq(
                Box::new(Expr::Plus(
                    Box::new(Expr::Value(Value::Int(1))),
                    Box::new(Expr::Times(
                        Box::new(Expr::Value(Value::Int(2))),
                        Box::new(Expr::Value(Value::Int(3)))
                    ))
                )),
                Box::new(Expr::Value(Value::Int(4)))
            ))
        );

        // if let Ok(result) = ast::parse("[1,2,3]") {
        //     use ast::{Expr, Value};
        //     if ast::parse(result) == Expr::Value(Value::Array(vec![
        //         Expr::Value(Value::Int(1)),
        //         Expr::Value(Value::Int(2)),
        //         Expr::Value(Value::Int(3)),
        //     ])) {
        //         assert!(true);
        //     } else {
        //         assert!(false);
        //     }
        // } else {
        //     assert!(false);
        // }
    }
}
