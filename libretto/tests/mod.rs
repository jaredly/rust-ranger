use libretto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Point {
  x: i32,
  y: i32,
  name: String,
}

#[test]
fn example() {
  let expr = libretto::process(r##" Point {x: 3, y: 5, name: "awesome"} "##).unwrap();
  assert_eq!(
    Ok(Point {
      x: 3,
      y: 5,
      name: "awesome".to_string()
    }),
    libretto::from_expr(&expr)
  );
}

#[test]
fn example_uneval() {
  let expr = libretto::process(r##" Point {x: 3 + 4, y: 5, name: "awesome"} "##).unwrap();
  assert_eq!(
    Err(libretto::Error::Unevaluated),
    libretto::from_expr::<Point>(&expr)
  );
}

#[test]
fn example_eval() {
  let expr = libretto::process(r##" Point {x: 3 + 4, y: 5, name: "awesome"} "##)
    .unwrap()
    .eval(&libretto::Scope::empty())
    .ok()
    .unwrap();
  assert_eq!(
    Ok(Point {
      x: 7,
      y: 5,
      name: "awesome".to_string()
    }),
    libretto::from_expr::<Point>(&expr)
  );
}

#[test]
fn example_eval_pass() {
  let mut scope = libretto::Scope::empty();
  scope.set("heads", 5).unwrap();
  let expr = libretto::process(r##" Point {x: 3 + 4 + heads, y: 5, name: "awesome"} "##)
    .unwrap()
    .eval(&scope)
    .ok()
    .unwrap();
  assert_eq!(
    Ok(Point {
      x: 12,
      y: 5,
      name: "awesome".to_string()
    }),
    libretto::from_expr::<Point>(&expr)
  );
}
