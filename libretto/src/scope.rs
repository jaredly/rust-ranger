use crate::ast::{Args, EvalError, Expr};
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
      match $scope.call_fn_raw($name, args) {
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
        Scope(vec![SingleScope::empty()])
    }
    pub fn push(&mut self) {
        self.0.insert(0, SingleScope::empty());
    }
    pub fn pop(&mut self) {
        self.0.remove(0);
    }

    pub fn call_fn_raw(&mut self, name: &str, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let mut scopesi = self.0.iter();
        let (fargs, mut body) = loop {
            if let Some(scope) = scopesi.next() {
                if let Some(f) = scope.fns.get(name) {
                    if f.0.len() != args.len() {
                        return Err(EvalError::FunctionWrongNumberArgs(f.0.len(), args.len()))
                    }
                    break f.clone()
                }
            } else {
                return Err(EvalError::MissingReference(name.to_owned()))
            }
        };
        self.push();
        // let mut sub = self.sub();
        for (aname, aval) in fargs.iter().zip(args) {
            self.set_raw(aname, aval);
        }
        body.eval(self)?;
        self.pop();
        return Ok(body)
    }


    pub fn get_fn(&self, key: &str) -> Option<&(Args, Expr)> {
        self.0[0].fns.get(key)
    }

    pub fn set_fn(&mut self, key: &str, args: Args, body: Expr) {
        self.0[0].fns.insert(key.to_owned(), (args, body));
    }

    // pub fn call_fn<'de, T>(&self, name: &str, args: Vec<Expr>) -> crate::error::Result<T>
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
                let replacement = match x {
                    Expr::Float(_)
                    | Expr::Int(_)
                    | Expr::Bool(_)
                    | Expr::String(_)
                    | Expr::Char(_)
                    | Expr::Unit => x.clone(),
                    _ => Expr::Moved
                };
                scope.vbls.insert(key.to_owned(), replacement);
                return Some(x)
            }
        }
        None
        // match self.0[0].vbls.remove(key) {
        //     // None => match self.parent {
        //     //     None => match key {
        //     //         "e" => Some(Expr::Float(std::f32::consts::E)),
        //     //         "pi" => Some(Expr::Float(std::f32::consts::PI)),
        //     //         "tau" => Some(Expr::Float(std::f32::consts::PI * 2.0)),
        //     //         "half_pi" => Some(Expr::Float(std::f32::consts::FRAC_PI_2)),
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
        //             Expr::Float(_)
        //             | Expr::Int(_)
        //             | Expr::Bool(_)
        //             | Expr::String(_)
        //             | Expr::Char(_)
        //             | Expr::Unit => v.clone(),
        //             _ => Expr::Moved
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
                return Some(x)
            }
        }
        None
        // match self.vbls.get_mut(key) {
        //     // None => match self.parent {
        //     //     None => match key {
        //     //         // "e" => Some(&mut Expr::Float(std::f32::consts::E)),
        //     //         // "pi" => Some(&mut Expr::Float(std::f32::consts::PI)),
        //     //         // "tau" => Some(&mut Expr::Float(std::f32::consts::PI * 2.0)),
        //     //         // "half_pi" => Some(&mut Expr::Float(std::f32::consts::FRAC_PI_2)),
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
                Some(x) => return Some(x)
            }
        }
        None
        // self.vbls.get(key) 
        // match self.vbls.get(key) {
        //     None => match self.parent {
        //         None => match key {
        //             "e" => Some(&Expr::Float(std::f32::consts::E)),
        //             "pi" => Some(&Expr::Float(std::f32::consts::PI)),
        //             "tau" => Some(&Expr::Float(std::f32::consts::PI * 2.0)),
        //             "half_pi" => Some(&Expr::Float(std::f32::consts::FRAC_PI_2)),
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
        self.0[0].vbls
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
}
