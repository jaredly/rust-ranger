use std::collections::HashMap;
use crate::ast::Expr;

pub struct LocalScope(im_rc::HashMap<String, Expr>);
impl LocalScope {
  pub fn empty() -> Self {
    LocalScope(im_rc::HashMap::new())
  }
  pub fn set(&self, key: &str, v: Expr) -> LocalScope {
    LocalScope(self.0.update(key.to_owned(), v))
  }
  pub fn get(&self, key: &str) -> Option<&Expr> {
    self.0.get(key)
  }
}

pub struct Scope(HashMap<String, Expr>);
impl Scope {
    pub fn empty() -> Self {
        Scope(HashMap::new())
    }

    pub fn get<'de, T>(&self, key: &str) -> crate::error::Result<()> where T: serde::Deserialize<'de> {
        match self.0.get(key) {
            None => Err(crate::error::Error::Message("Missing vbl".to_owned())),
            Some(expr) => crate::de::from_expr(expr)
        }
    }

    pub fn get_raw(&self, key: &str) -> Option<&Expr> {
      self.0.get(key)
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> crate::error::Result<()> where T: serde::Serialize {
        self.0.insert(key.to_owned(), crate::ser::to_expr(&value)?);
        Ok(())
    }

    pub fn set_raw(&mut self, key: &str, value: Expr) {
        self.0.insert(key.to_owned(), value);
    }
}
