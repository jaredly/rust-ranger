use crate::ast::{Args, Expr, ExprDesc, Pos};
use crate::error::{EvalError, EvalErrorDesc};
use std::collections::HashMap;

#[macro_export]
macro_rules! call_fn {
  ($scope: expr, $name: expr, $($arg: expr),*) => {
    {
      let args = vec![$( libretto::to_expr(&$arg).unwrap() ),*];
      match $scope.call_fn_raw($name, args, libretto::Pos::default()) {
        Err(e) => Err(e.into()),
        Ok(result) => libretto::from_expr(&result)
      }
    }
  };
}

#[derive(Debug, PartialEq)]
pub struct SingleScope {
    id: usize,
    vbls: HashMap<String, Expr>,
    fns: HashMap<String, (Args, Expr)>,
}

#[derive(Debug, PartialEq)]
pub struct Scope(Vec<SingleScope>);

impl Scope {
    pub fn new() -> Self {
        Scope(vec![SingleScope::globals()])
    }
    pub fn push(&mut self) {
        self.0.insert(0, SingleScope::empty());
    }
    pub fn pop(&mut self) {
        self.0.remove(0);
    }

    pub fn call_fn_raw(
        &mut self,
        name: &str,
        args: Vec<Expr>,
        pos: Pos,
    ) -> Result<Expr, EvalError> {
        let mut scopesi = self.0.iter();
        let (fargs, mut body) = loop {
            if let Some(scope) = scopesi.next() {
                if let Some(f) = scope.fns.get(name) {
                    if f.0.len() != args.len() {
                        return Err(
                            EvalErrorDesc::FunctionWrongNumberArgs(f.0.len(), args.len())
                                .with_pos(pos),
                        );
                    }
                    break f.clone();
                }
            } else {
                if name == "log" {
                    let args = args
                        .into_iter()
                        .map(|m| match m.desc {
                            ExprDesc::String(s) => s,
                            _ => format!("{:?}", m),
                        })
                        .collect::<Vec<String>>()
                        .concat();
                    println!("{} at {}:{}", args, pos.start.0, pos.start.1);
                    return Ok(ExprDesc::Unit.into());
                }
                return Err(EvalErrorDesc::MissingReference(name.to_owned()).with_pos(pos));
            }
        };
        self.push();
        // let mut sub = self.sub();
        for (aname, aval) in fargs.iter().zip(args) {
            self.set_raw(aname, aval);
        }
        body.eval(self)?;
        self.pop();
        return Ok(body);
    }

    pub fn get_fn(&self, key: &str) -> Option<&(Args, Expr)> {
        self.0[0].fns.get(key)
    }

    pub fn set_fn(&mut self, key: &str, args: Args, body: Expr) {
        self.0[0].fns.insert(key.to_owned(), (args, body));
    }

    pub fn show(&self) -> String {
        format!("{:?}", self)
    }

    pub fn move_raw(&mut self, key: &str) -> Option<Expr> {
        for scope in self.0.iter_mut() {
            if let Some(x) = scope.vbls.remove(key) {
                let replacement = match x.desc {
                    ExprDesc::Float(_)
                    | ExprDesc::Int(_)
                    | ExprDesc::Bool(_)
                    | ExprDesc::String(_)
                    | ExprDesc::Char(_)
                    | ExprDesc::Unit => x.clone(),
                    _ => ExprDesc::Moved.match_pos(&x),
                };
                scope.vbls.insert(key.to_owned(), replacement);
                return Some(x);
            }
        }
        None
    }

    pub fn get_raw_mut(&mut self, key: &str) -> Option<&mut Expr> {
        for scope in self.0.iter_mut() {
            if let Some(x) = scope.vbls.get_mut(key) {
                return Some(x);
            }
        }
        None
    }

    pub fn get_raw(&self, key: &str) -> Option<&Expr> {
        for scope in self.0.iter() {
            match scope.vbls.get(key) {
                None => (),
                Some(x) => return Some(x),
            }
        }
        None
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> crate::error::Result<()>
    where
        T: serde::Serialize,
    {
        self.0[0]
            .vbls
            .insert(key.to_owned(), crate::ser::to_expr(&value)?);
        Ok(())
    }

    pub fn set_raw(&mut self, key: &str, value: Expr) {
        self.0[0].vbls.insert(key.to_owned(), value);
    }
}

impl SingleScope {
    pub fn empty() -> Self {
        SingleScope {
            id: 0,
            vbls: HashMap::new(),
            fns: HashMap::new(),
        }
    }
    pub fn globals() -> Self {
        let mut scope = Self::empty();
        scope
            .vbls
            .insert("e".to_owned(), ExprDesc::Float(std::f32::consts::E).into());
        scope.vbls.insert(
            "pi".to_owned(),
            ExprDesc::Float(std::f32::consts::PI).into(),
        );
        scope.vbls.insert(
            "tau".to_owned(),
            ExprDesc::Float(std::f32::consts::PI * 2.0).into(),
        );
        scope.vbls.insert(
            "half_pi".to_owned(),
            ExprDesc::Float(std::f32::consts::FRAC_PI_2).into(),
        );
        scope
    }
}
