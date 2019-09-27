#![allow(dead_code)]
use pest::Parser;
use pest_derive::*;
use unescape;
use serde_json;

mod ast;
mod de;

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
    fn struct_() {
        assert_eq!(
            ast::process("Hello { one: 2 }"),
            Ok(Expr::Struct("Hello".to_string(), vec![
                ("one".to_string(), Expr::Int(2)),
            ]))
        );
    }

    #[test]
    fn named_tuple() {
        assert_eq!(
            ast::process("Hello ( 2, 3 )"),
            Ok(Expr::NamedTuple("Hello".to_string(), vec![
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
