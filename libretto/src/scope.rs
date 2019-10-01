use crate::ast::{Args, Expr, ExprDesc, Pos};
use crate::error::{EvalError, EvalErrorDesc};
use std::collections::HashMap;

#[macro_export]
macro_rules! call_fn {
  ($scope: expr, $name: expr, $($arg: expr),*) => {
    {
      // let args = vec![$( libretto::to_expr(&$arg) ),*];
      // match args.iter().find(Result::is_err) {
      //   None => match $scope.call_fn_raw($name, args.iter().map(Result::unwrap).collect()) {
      //     Ok(expr) => libretto::from_expr(&expr),
      //     Err(e) => Err(e.into())
      //   },
      //   Some(item) => item.clone().into()
      // }

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
        // TODO prepopulate with globals
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

    // pub fn call_fn<'de, T>(&self, name: &str, args: Vec<ExprDesc>) -> crate::error::Result<T>
    // where
    //   T: serde::Deserialize<'de>,
    // {
    //   let result = self.call_fn_raw(name, args)?;
    //   crate::de::from_expr(&result)
    // }

    // pub fn get<'de, T>(&self, key: &str) -> crate::error::Result<T>
    // where
    //   T: serde::Deserialize<'de>,
    // {
    //     for scope in self.0.iter() {
    //         if let Some(x) = scope.vbls.get(key) {
    //             return crate::de::from_expr(x)
    //         }
    //     }
    //     Err(crate::error::Error::Message("mope".to_owned()))
    // }

    pub fn show(&self) -> String {
        // let own = format!("own({}): {:?}", self.id, self.vbls.keys());
        // match self.parent {
        //     None => own,
        //     Some(parent) => format!("{}, parent: {}", own, parent.show()),
        // }
        format!("{:?}", self)
    }

    pub fn move_raw(&mut self, key: &str) -> Option<Expr> {
        // println!("Looking for {} in {} ", key, self.show());
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
        // match self.0[0].vbls.remove(key) {
        //     // None => match self.parent {
        //     //     None => match key {
        //     //         "e" => Some(ExprDesc::Float(std::f32::consts::E)),
        //     //         "pi" => Some(ExprDesc::Float(std::f32::consts::PI)),
        //     //         "tau" => Some(ExprDesc::Float(std::f32::consts::PI * 2.0)),
        //     //         "half_pi" => Some(ExprDesc::Float(std::f32::consts::FRAC_PI_2)),
        //     //         _ => None,
        //     //     },
        //     //     Some(parent) => {
        //     //         // parent.move_raw(key)
        //     //         // TODO new enum
        //     //         None
        //     //     },
        //     // },
        //     None => None,
        //     Some(v) => {
        //         let replacement = match v {
        //             ExprDesc::Float(_)
        //             | ExprDesc::Int(_)
        //             | ExprDesc::Bool(_)
        //             | ExprDesc::String(_)
        //             | ExprDesc::Char(_)
        //             | ExprDesc::Unit => v.clone(),
        //             _ => ExprDesc::Moved
        //         };
        //         self.0[0].vbls.insert(key.to_owned(), replacement);
        //         Some(v)
        //     },
        // }
    }

    pub fn get_raw_mut(&mut self, key: &str) -> Option<&mut Expr> {
        // println!("Looking for {} in {} ", key, self.show());
        for scope in self.0.iter_mut() {
            if let Some(x) = scope.vbls.get_mut(key) {
                return Some(x);
            }
        }
        None
        // match self.vbls.get_mut(key) {
        //     // None => match self.parent {
        //     //     None => match key {
        //     //         // "e" => Some(&mut ExprDesc::Float(std::f32::consts::E)),
        //     //         // "pi" => Some(&mut ExprDesc::Float(std::f32::consts::PI)),
        //     //         // "tau" => Some(&mut ExprDesc::Float(std::f32::consts::PI * 2.0)),
        //     //         // "half_pi" => Some(&mut ExprDesc::Float(std::f32::consts::FRAC_PI_2)),
        //     //         _ => None,
        //     //     },
        //     //     Some(parent) => {
        //     //         // parent.get_raw_mut(key)
        //     //         None
        //     //     },
        //     // },
        //     Some(v) => Some(v),
        // }
    }

    pub fn get_raw(&self, key: &str) -> Option<&Expr> {
        // println!("Looking for {} in {} ", key, self.show());
        for scope in self.0.iter() {
            match scope.vbls.get(key) {
                None => (),
                Some(x) => return Some(x),
            }
        }
        None
        // self.vbls.get(key)
        // match self.vbls.get(key) {
        //     None => match self.parent {
        //         None => match key {
        //             "e" => Some(&ExprDesc::Float(std::f32::consts::E)),
        //             "pi" => Some(&ExprDesc::Float(std::f32::consts::PI)),
        //             "tau" => Some(&ExprDesc::Float(std::f32::consts::PI * 2.0)),
        //             "half_pi" => Some(&ExprDesc::Float(std::f32::consts::FRAC_PI_2)),
        //             _ => None,
        //         },
        //         Some(parent) => parent.get_raw(key),
        //     },
        //     Some(v) => Some(v),
        // }
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
        // println!("Setting {} in {}", key, self.show());
        self.0[0].vbls.insert(key.to_owned(), value);
    }
}

impl SingleScope {
    pub fn empty() -> Self {
        // println!("New empty scope");
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
