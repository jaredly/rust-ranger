use std::collections::HashMap;
use crate::ast::{Args, Expr, EvalError};

pub struct Scope<'a> {
  vbls: HashMap<String, Expr>,
  fns: HashMap<String, (Args, Expr)>,
  parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn empty() -> Self {
        Scope{vbls: HashMap::new(), fns: HashMap::new(), parent: None}
    }

    pub fn from(parent: &'a Scope<'a>) -> Self {
      Scope{ vbls: HashMap::new(), fns: HashMap::new(), parent: Some(parent)}
    }

    pub fn sub(&'a self) -> Scope<'a> {
      Self::from(self)
    }

    pub fn get_fn(&self, key: &str) -> Option<&(Args, Expr)> {
      self.fns.get(key)
    }

    pub fn set_fn(&mut self, key: &str, args: Args, body: Expr) {
      self.fns.insert(key.to_owned(), (args, body));
    }

    pub fn call_fn_raw(&self, name: &str, args: Vec<Expr>) -> Result<Expr, EvalError> {
      match self.fns.get(name) {
        None => match self.parent {
          None => {
            // println!("{:?}", self.fns);
            Err(EvalError::MissingReference(name.to_owned()))
          },
          Some(parent) => parent.call_fn_raw(name, args)
        },
        Some(f) if f.0.len() != args.len() => Err(EvalError::FunctionWrongNumberArgs(f.0.len(), args.len())),
        Some(f) => {
          let mut sub = self.sub();
          for (aname, aval) in f.0.iter().zip(args) {
            sub.set_raw(aname, aval);
          }
          f.1.clone().eval(&sub)
        }
      }
    }

    pub fn get<'de, T>(&self, key: &str) -> crate::error::Result<()> where T: serde::Deserialize<'de> {
        match self.vbls.get(key) {
            None => match self.parent {
              None => Err(crate::error::Error::Message("Missing vbl".to_owned())),
              Some(parent) => parent.get::<T>(key)
            },
            Some(expr) => crate::de::from_expr(expr)
        }
    }

    pub fn get_raw(&self, key: &str) -> Option<&Expr> {
      match self.vbls.get(key) {
        None => match self.parent {
          None => None,
          Some(parent) => parent.get_raw(key)
        },
        Some(v) => Some(v)
      }
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> crate::error::Result<()> where T: serde::Serialize {
        self.vbls.insert(key.to_owned(), crate::ser::to_expr(&value)?);
        Ok(())
    }

    pub fn set_raw(&mut self, key: &str, value: Expr) {
        self.vbls.insert(key.to_owned(), value);
    }
}
