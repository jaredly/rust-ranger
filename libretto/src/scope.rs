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
    pub parent: Option<&'a mut Scope<'a>>,
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

    pub fn from(parent: &'a mut Scope<'a>) -> Self {
        Scope {
            id: parent.id + 1,
            vbls: HashMap::new(),
            fns: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn sub(&'a mut self) -> Scope<'a> {
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

    pub fn call_fn_raw<'b>(&'b mut self, name: &str, args: Vec<Expr>) -> Result<Expr, EvalError> {
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
                let sub = Scope::from(&mut self);
                for (aname, aval) in f.0.iter().zip(args) {
                    sub.set_raw(aname, aval);
                }
                let body = f.1.clone();
                body.eval(&sub)?;
                Ok(body)
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

    pub fn move_raw(&mut self, key: &str) -> Option<Expr> {
        // println!("Looking for {} in {} ", key, self.show());
        match self.vbls.remove(key) {
            None => match self.parent {
                None => match key {
                    "e" => Some(Expr::Float(std::f32::consts::E)),
                    "pi" => Some(Expr::Float(std::f32::consts::PI)),
                    "tau" => Some(Expr::Float(std::f32::consts::PI * 2.0)),
                    "half_pi" => Some(Expr::Float(std::f32::consts::FRAC_PI_2)),
                    _ => None,
                },
                Some(parent) => parent.move_raw(key),
            },
            Some(v) => {
                let replacement = match v {
                    Expr::Float(_)
                    | Expr::Int(_)
                    | Expr::Bool(_)
                    | Expr::String(_)
                    | Expr::Char(_)
                    | Expr::Unit => v.clone(),
                    _ => Expr::Moved
                };
                self.vbls.insert(key.to_owned(), replacement);
                Some(v)
            },
        }
    }

    pub fn get_raw_mut(&self, key: &str) -> Option<&mut Expr> {
        // println!("Looking for {} in {} ", key, self.show());
        match self.vbls.get_mut(key) {
            None => match self.parent {
                None => match key {
                    "e" => Some(&mut Expr::Float(std::f32::consts::E)),
                    "pi" => Some(&mut Expr::Float(std::f32::consts::PI)),
                    "tau" => Some(&mut Expr::Float(std::f32::consts::PI * 2.0)),
                    "half_pi" => Some(&mut Expr::Float(std::f32::consts::FRAC_PI_2)),
                    _ => None,
                },
                Some(parent) => parent.get_raw_mut(key),
            },
            Some(v) => Some(v),
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
