use libretto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
    t: (i32, f32),
    name: String,
}

#[test]
fn example() {
    let expr =
        libretto::eval_expr(r##" Point {x: 3, y: 5, t: (2, 3.2), name: "awesome"} "##).unwrap();
    assert_eq!(
        Ok(Point {
            x: 3,
            y: 5,
            t: (2, 3.2),
            name: "awesome".to_string()
        }),
        libretto::from_expr(&expr)
    );
}

// #[test]
// fn example_uneval() {
//     let expr = libretto::process_expr(r##" Point {x: 3 + 4, y: 5, name: "awesome"} "##).unwrap();
//     assert_eq!(
//         Err(libretto::Error::Unevaluated("".to_owned())),
//         libretto::from_expr::<Point>(&expr)
//     );
// }

#[test]
fn example_eval() {
    let expr =
        libretto::eval_expr(r##" Point {x: 3 + 4, y: 5, t: (2, 3.2), name: "awesome"} "##).unwrap();
    assert_eq!(
        Ok(Point {
            x: 7,
            y: 5,
            t: (2, 3.2),
            name: "awesome".to_string()
        }),
        libretto::from_expr::<Point>(&expr)
    );
}

#[test]
fn example_eval_pass() {
    let mut scope = libretto::Scope::new();
    scope.set("heads", 5).unwrap();
    let expr = libretto::process_expr(
        r##" Point {x: 3 + 4 + heads, y: 5, t: (2, 3.2), name: "awesome"} "##,
    )
    .unwrap()
    .into_eval(&mut scope)
    .ok()
    .unwrap();
    assert_eq!(
        Ok(Point {
            x: 12,
            y: 5,
            t: (2, 3.2),
            name: "awesome".to_string()
        }),
        libretto::from_expr::<Point>(&expr)
    );
}

#[test]
fn fn_call() {
    assert_eq!(
        libretto::from_expr::<usize>(
            &libretto::eval_expr(
                r##"
fn awesome() {
  5
}
fn add10(x: any) {
  x + 10
}
add10(awesome() * 2)
"##
            )
            .unwrap()
        ),
        Ok(20)
    )
}

#[test]
fn file() {
    let mut scope = libretto::eval_file(
        r##"
fn party(x: any, y: any, z: any) {
  x * (y + z)
}
"##,
    )
    .unwrap();
    assert_eq!(libretto::call_fn!(scope, "party", 10, 2, 3), Ok(50))
}

#[test]
fn file_pointer() {
    let mut scope = libretto::eval_file(
        r##"
fn party(x: any, y: any, z: any, m: any) {
  x * (y + z) + m.x
}
"##,
    )
    .unwrap();
    assert_eq!(
        libretto::call_fn!(
            scope,
            "party",
            10,
            2,
            3,
            Point {
                x: 1,
                y: 2,
                t: (2, 3.2),
                name: "ok".into()
            }
        ),
        Ok(51)
    )
}

// fn check<'de, T: serde::Serialize, U: serde::Deserialize<'de>>(input: &str, arg: T, output: U) {
//   assert_eq!(
//     libretto::call_fn!(
//       libretto::eval_file(input).unwrap(),
//       "run",
//       arg
//     )
//     , Ok(output))
// }

#[test]
fn member_fns() {
    assert_eq!(
        libretto::call_fn!(
            libretto::eval_file(
                r#"
      fn go(x: any, y: any) { x.cos() + pi.cos().abs() + (y as f32) }
      "#
            )
            .unwrap(),
            "go",
            std::f32::consts::PI,
            23
        ),
        Ok(23.0)
    )
}

#[test]
fn ifs() {
    assert_eq!(
        libretto::from_expr::<usize>(
            &libretto::eval_expr(
                r##"
let party = 16;
if 5 == 10 {
  5
} else if party / 2 == 8 {
  23
} else {
  10
}
"##
            )
            .unwrap()
        ),
        Ok(23)
    )
}

#[test]
fn matches() {
    assert_eq!(
        libretto::from_expr::<Vec<usize>>(
            &libretto::eval_expr(
                r##"
let party = 16;
let res = vec![
  match party {
    16 => 5,
    _ => 2
  },
  match party {
    15 => 5,
    _ => 2
  },
];
res.push(
  match Points { x: 3, y: 5 } {
    Points { x: 2 } => 111,
    Points { x, y: 5 } => x,
    _ => 0
  }
);
let aa = log(res.len(), res.clone());
res
"##
            )
            .unwrap()
        ),
        Ok(vec![5, 2, 3])
    )
}
