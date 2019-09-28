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
  let expr = libretto::eval_expr(r##" Point {x: 3, y: 5, name: "awesome"} "##).unwrap();
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
  let expr = libretto::process_expr(r##" Point {x: 3 + 4, y: 5, name: "awesome"} "##).unwrap();
  assert_eq!(
    Err(libretto::Error::Unevaluated),
    libretto::from_expr::<Point>(&expr)
  );
}

#[test]
fn example_eval() {
  let expr = libretto::eval_expr(r##" Point {x: 3 + 4, y: 5, name: "awesome"} "##).unwrap();
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
  let expr = libretto::process_expr(r##" Point {x: 3 + 4 + heads, y: 5, name: "awesome"} "##)
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

#[test]
fn fn_call() {
  assert_eq!(
    libretto::from_expr::<usize>(&libretto::eval_expr(r##"
fn awesome() {
  5
}
fn add10(x) {
  x + 10
}
add10(awesome() * 2)
"##).unwrap()), Ok(20))
}

#[test]
fn file() {
  let scope = libretto::eval_file(r##"
fn party(x, y, z) {
  x * (y + z)
}
"##).unwrap();
  assert_eq!(libretto::call_fn!(scope, "party", 10, 2, 3), Ok(50))
}

#[test]
fn file_pointer() {
  let scope = libretto::eval_file(r##"
fn party(x, y, z, m) {
  x * (y + z) + m.x
}
"##).unwrap();
  assert_eq!(libretto::call_fn!(scope, "party", 10, 2, 3, Point {x:1,y:2,name:"ok".into()}), Ok(51))
}

#[test]
fn member_fns() {
  assert_eq!(
    libretto::call_fn!(
      libretto::eval_file(r#"
      fn go(x) { x.cos() + 0.0.sin().abs() }
      "#).unwrap(),
      "go",
      std::f32::consts::PI
    )
    , Ok(-1.0))
}