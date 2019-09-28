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

pub struct Scope<'a> {
    pub id: usize,
    pub vbls: HashMap<String, Expr>,
    pub fns: HashMap<String, (Args, Expr)>,
    pub parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn empty() -> Self {
        // println!("New empty scope");
        Scope {
            id: 0,
            vbls: HashMap::new(),
            fns: HashMap::new(),
            parent: None,
        }
    }

    pub fn from(parent: &'a Scope<'a>) -> Self {
        Scope {
            id: parent.id + 1,
            vbls: HashMap::new(),
            fns: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn sub(&'a self) -> Scope<'a> {
        // println!("New sub scope from {}", self.show());
        Self::from(self)
    }

    pub fn get_fn(&self, key: &str) -> Option<&(Args, Expr)> {
        self.fns.get(key)
    }

    pub fn set_fn(&mut self, key: &str, args: Args, body: Expr) {
        self.fns.insert(key.to_owned(), (args, body));
    }

    // pub fn call_fn<'de, T>(&self, name: &str, args: Vec<Expr>) -> crate::error::Result<T>
    // where
    //   T: serde::Deserialize<'de>,
    // {
    //   let result = self.call_fn_raw(name, args)?;
    //   crate::de::from_expr(&result)
    // }

    pub fn call_fn_raw(&self, name: &str, args: Vec<Expr>) -> Result<Expr, EvalError> {
        match self.fns.get(name) {
            None => match self.parent {
                None => {
                    match name {
                        "log" => {
                            let mut res = vec![];
                            for arg in args {
                                res.push(match arg {
                                    Expr::String(string) => string,
                                    arg => format!("{:?}", arg),
                                });
                            }
                            // println!("{}", res.concat());
                            Ok(Expr::Unit)
                        }
                        _ => {
                            // println!("{:?}", self.fns);
                            Err(EvalError::MissingReference(name.to_owned()))
                        }
                    }
                }
                Some(parent) => parent.call_fn_raw(name, args),
            },
            Some(f) if f.0.len() != args.len() => {
                Err(EvalError::FunctionWrongNumberArgs(f.0.len(), args.len()))
            }
            Some(f) => {
                let mut sub = self.sub();
                for (aname, aval) in f.0.iter().zip(args) {
                    sub.set_raw(aname, aval.eval(&self)?);
                }
                f.1.clone().eval(&sub)
            }
        }
    }

    // pub fn get<'de, T>(&self, key: &str) -> crate::error::Result<T>
    // where
    //   T: serde::Deserialize<'de>,
    // {
    //   match self.vbls.get(key) {
    //     None => match self.parent {
    //       None => Err(crate::error::Error::Message("Missing vbl".to_owned())),
    //       Some(parent) => parent.get::<T>(key),
    //     },
    //     Some(expr) => crate::de::from_expr(&expr.clone()),
    //   }
    // }

    pub fn show(&self) -> String {
        let own = format!("own({}): {:?}", self.id, self.vbls.keys());
        match self.parent {
            None => own,
            Some(parent) => format!("{}, parent: {}", own, parent.show()),
        }
    }

    pub fn get_raw(&self, key: &str) -> Option<&Expr> {
        // println!("Looking for {} in {} ", key, self.show());
        match self.vbls.get(key) {
            None => match self.parent {
                None => match key {
                    "e" => Some(&Expr::Float(std::f32::consts::E)),
                    "pi" => Some(&Expr::Float(std::f32::consts::PI)),
                    "tau" => Some(&Expr::Float(std::f32::consts::PI * 2.0)),
                    "half_pi" => Some(&Expr::Float(std::f32::consts::FRAC_PI_2)),
                    _ => None,
                },
                Some(parent) => parent.get_raw(key),
            },
            Some(v) => Some(v),
        }
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> crate::error::Result<()>
    where
        T: serde::Serialize,
    {
        self.vbls
            .insert(key.to_owned(), crate::ser::to_expr(&value)?);
        Ok(())
    }

    pub fn set_raw(&mut self, key: &str, value: Expr) {
        // println!("Setting {} in {}", key, self.show());
        self.vbls.insert(key.to_owned(), value);
    }
}
