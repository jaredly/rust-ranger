#![allow(dead_code)]

mod ast;
mod de;
mod error;
mod parser;
mod scope;
mod ser;

pub use ast::{Expr, ExprDesc, Pos};
pub use de::from_expr;
pub use error::Error;
pub use parser::{process_expr, process_file};
pub use scope::Scope;
pub use ser::to_expr;

pub fn eval_expr(input: &str) -> Result<Expr, error::EvalError> {
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
    use ast::ExprDesc;

    #[test]
    fn array() {
        assert_eq!(
            parser::process_expr("vec![1,2,3]")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::array(vec![1.into(), 2.into(), 3.into()]))
        );
    }

    #[test]
    fn struct_() {
        assert_eq!(
            parser::process_expr("Hello { one: 2 }")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(ExprDesc::Struct("Hello".to_string(), vec![("one".to_string(), 2.into()),]).into())
        );
    }

    #[test]
    fn named_tuple() {
        assert_eq!(
            parser::process_expr("Hello ( 2, 3 )")
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(ExprDesc::NamedTuple("Hello".to_string(), vec![2.into(), 3.into(),]).into())
        );
    }

    #[test]
    fn plus_minus() {
        assert_eq!(
            parser::process_expr("1 + 2 - 3"),
            Ok(ExprDesc::Block(
                vec![],
                Box::new(
                    ExprDesc::Minus(
                        Box::new(ExprDesc::Plus(Box::new(1.into()), Box::new(2.into()),).into()),
                        Box::new(3.into()),
                    )
                    .into()
                )
            )
            .into())
        );

        assert_eq!(
            parser::process_expr("1 - 2 + 3"),
            Ok(ExprDesc::Block(
                vec![],
                Box::new(
                    ExprDesc::Plus(
                        Box::new(ExprDesc::Minus(Box::new(1.into()), Box::new(2.into()),).into()),
                        Box::new(3.into()),
                    )
                    .into()
                )
            )
            .into())
        );

        assert_eq!(
            parser::process_expr(r##"vec!["o\nne", r#"t"w\no"#, 'a', '\n', "😅"]"##)
                .unwrap()
                .into_eval(&mut Scope::new()),
            Ok(Expr::array(vec![
                ("o\nne".to_string()).into(),
                ("t\"w\\no".to_string()).into(),
                ('a').into(),
                ('\n').into(),
                ("😅".to_string()).into(),
            ]))
        );
    }

    #[test]
    fn struct__() {
        assert_eq!(
            parser::process_expr(r##" Point {x: 3, y: 5, name: "awesome"} "##),
            Ok(ExprDesc::Block(
                vec![],
                Box::new(
                    ExprDesc::Struct(
                        "Point".into(),
                        vec![
                            ("x".into(), (3).into()),
                            ("y".into(), (5).into()),
                            ("name".into(), ("awesome".to_owned()).into()),
                        ]
                    )
                    .into()
                )
            )
            .into())
        );
    }

    #[test]
    fn many_ops() {
        assert_eq!(
            parser::process_expr("1 - 2 * 3 + 5 == 4"),
            Ok(ExprDesc::Block(
                vec![],
                Box::new(
                    ExprDesc::Eq(
                        Box::new(
                            ExprDesc::Plus(
                                Box::new(
                                    ExprDesc::Minus(
                                        Box::new((1).into()),
                                        Box::new(
                                            ExprDesc::Times(
                                                Box::new((2).into()),
                                                Box::new((3).into())
                                            )
                                            .into()
                                        )
                                    )
                                    .into()
                                ),
                                Box::new((5).into())
                            )
                            .into()
                        ),
                        Box::new((4).into())
                    )
                    .into()
                )
            )
            .into())
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
