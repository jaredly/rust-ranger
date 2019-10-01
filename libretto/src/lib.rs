#![allow(dead_code)]

mod ast;
mod de;
mod error;
mod parser;
mod scope;
mod ser;

pub use ast::Expr;
pub use de::from_expr;
pub use error::Error;
pub use parser::{process_expr, process_file};
pub use scope::Scope;
pub use ser::to_expr;

pub fn eval_expr(input: &str) -> Result<Expr, ast::EvalError> {
    process_expr(input).unwrap().into_eval(&mut Scope::new())
}

pub fn eval_file(input: &str) -> Result<Scope, error::Error> {
    let mut scope = Scope::new();
    for stmt in process_file(input)? {
        stmt.eval(&mut scope)?;
    }
    Ok(scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Expr;

    #[test]
    fn array() {
        assert_eq!(
            parser::process_expr("vec![1,2,3]")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::Array(vec![Expr::Int(1).into(), Expr::Int(2).into(), Expr::Int(3).into(),]).into())
        );
    }

    #[test]
    fn struct_() {
        assert_eq!(
            parser::process_expr("Hello { one: 2 }")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::Struct(
                "Hello".to_string(),
                vec![("one".to_string(), Expr::Int(2).into()),]
            ).into())
        );
    }

    #[test]
    fn named_tuple() {
        assert_eq!(
            parser::process_expr("Hello ( 2, 3 )")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::NamedTuple(
                "Hello".to_string(),
                vec![Expr::Int(2).into(), Expr::Int(3).into(),]
            ))
        );
    }

    #[test]
    fn plus_minus() {
        assert_eq!(
            parser::process_expr("1 + 2 - 3"),
            Ok(Expr::Block(
                vec![],
                Box::new(Expr::Minus(
                    Box::new(Expr::Plus(Box::new(Expr::Int(1).into()), Box::new(Expr::Int(2).into()),).into()),
                    Box::new(Expr::Int(3).into()),
                ).into())
            ))
        );

        assert_eq!(
            parser::process_expr("1 - 2 + 3"),
            Ok(Expr::Block(
                vec![],
                Box::new(Expr::Plus(
                    Box::new(Expr::Minus(Box::new(Expr::Int(1).into()), Box::new(Expr::Int(2).into()),).into()),
                    Box::new(Expr::Int(3).into()),
                ).into())
            ))
        );

        assert_eq!(
            parser::process_expr(r##"vec!["o\nne", r#"t"w\no"#, 'a', '\n', "😅"]"##)
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::Array(vec![
                Expr::String("o\nne".to_string()).into(),
                Expr::String("t\"w\\no".to_string()).into(),
                Expr::Char('a').into(),
                Expr::Char('\n').into(),
                Expr::String("😅".to_string()).into(),
            ]))
        );
    }

    #[test]
    fn struct__() {
        assert_eq!(
            parser::process_expr(r##" Point {x: 3, y: 5, name: "awesome"} "##),
            Ok(Expr::Block(
                vec![],
                Box::new(Expr::Struct(
                    "Point".into(),
                    vec![
                        ("x".into(), Expr::Int(3).into()),
                        ("y".into(), Expr::Int(5).into()),
                        ("name".into(), Expr::String("awesome".into()).into()),
                    ]
                ).into())
            ).into())
        );
    }

    #[test]
    fn many_ops() {
        assert_eq!(
            parser::process_expr("1 - 2 * 3 + 5 == 4"),
            Ok(Expr::Block(
                vec![],
                Box::new(Expr::Eq(
                    Box::new(Expr::Plus(
                        Box::new(Expr::Minus(
                            Box::new(Expr::Int(1).into()),
                            Box::new(Expr::Times(Box::new(Expr::Int(2).into()), Box::new(Expr::Int(3).into())).into())
                        ).into()),
                        Box::new(Expr::Int(5).into())
                    ).into()),
                    Box::new(Expr::Int(4).into())
                ).into())
            ).into())
        );
    }

    #[test]
    fn complex() {
        parser::process_expr(
            r###"
{
    one: 1,
    "two": 2,
    three_four: vec![3, 3 + 1],
    five: None,
    six: Some(6 - (3 - 2)),
    "7": true != false
}
        "###,
        )
        .unwrap();
    }
}
