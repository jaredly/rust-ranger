use crate::scope::Scope;

pub type Args = Vec<String>;

trait TryMap<T> {
    fn try_map<U, E, F: Fn(T) -> Result<U, E>>(self, f: F) -> Result<Vec<U>, E>;
}

impl<T> TryMap<T> for Vec<T> {
    fn try_map<U, E, F: Fn(T) -> Result<U, E>>(self, f: F) -> Result<Vec<U>, E> {
        let mut res = vec![];
        for item in self {
            res.push(f(item)?);
        }
        Ok(res)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Let(String, Expr),
    Expr(Expr),
    FnDefn(String, Args, Expr),
}

impl Statement {
    pub fn eval(self, scope: &mut Scope) -> Result<(), EvalError> {
        // println!(">> Statement eval {:?} with scope: {}", self, scope.show());
        match self {
            Statement::Let(name, mut v) => {
                v.eval(scope)?;
                scope.set_raw(&name, v)
            }
            Statement::Expr(mut e) => {
                e.eval(scope)?;
            }
            Statement::FnDefn(name, args, body) => {
                scope.set_fn(&name, args, body)
                // scope.set_raw(&name, Expr::Lambda(args, Box::new(body)))
            }
        };
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    F32,
    I32,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Float(f32),
    Int(i32),
    Bool(bool),
    Char(char),
    String(String),

    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Option(Box<Option<Expr>>),
    Ident(String),

    Unit,
    Struct(String, Vec<(String, Expr)>),
    NamedTuple(String, Vec<Expr>),

    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Times(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),

    MemberAccess(Box<Expr>, Vec<(String, Option<Vec<Expr>>)>),
    Cast(Box<Expr>, Type),

    Block(Vec<Statement>, Box<Expr>),
    // Lambda(Args, Box<Expr>),
    FnCall(String, Vec<Expr>),

    IfChain(Vec<(IfCond, Expr)>, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<(Pattern, Expr)>),

    Moved,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Const {
    Float(f32),
    Int(i32),
    Bool(bool),
    Char(char),
    String(String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Pattern {
    Ident(String),
    Const(Const),
    Any,
    Tuple(String, Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum IfCond {
    Value(Expr),
    IfLet(Pattern, Expr),
}

impl From<f32> for Expr {
    fn from(i: f32) -> Self {
        Expr::Float(i)
    }
}
impl From<i32> for Expr {
    fn from(i: i32) -> Self {
        Expr::Int(i)
    }
}
impl From<bool> for Expr {
    fn from(i: bool) -> Self {
        Expr::Bool(i)
    }
}
impl From<char> for Expr {
    fn from(i: char) -> Self {
        Expr::Char(i)
    }
}
impl From<String> for Expr {
    fn from(i: String) -> Self {
        Expr::String(i)
    }
}
impl<T> From<Vec<T>> for Expr
where
    T: Into<Expr>,
{
    fn from(i: Vec<T>) -> Self {
        Expr::Array(i.into_iter().map(|t| t.into()).collect())
    }
}
// impl<T> std::convert::TryFrom<T> for Expr where T: serde::Serialize { fn from(i: T) -> Self { crate::ser::to_expr(&i) } }

#[derive(PartialEq, Debug, Clone)]
pub enum EvalError {
    InvalidType(&'static str),
    MissingMember(String),
    MissingReference(String),
    FunctionValue,
    FunctionWrongNumberArgs(usize, usize),
    Unmatched,
}

impl Expr {
    pub fn needs_evaluation(&self) -> bool {
        match self {
            Expr::Float(_) | Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Char(_) => {
                false
            }
            Expr::NamedTuple(_, items) | Expr::Array(items) => {
                items.iter().any(Expr::needs_evaluation)
            }
            Expr::Struct(_, items) | Expr::Object(items) => {
                items.iter().any(|(_, expr)| expr.needs_evaluation())
            }
            Expr::Option(inner) => inner
                .as_ref()
                .as_ref()
                .map_or(false, |expr| expr.needs_evaluation()),
            _ => true,
        }
    }

    pub fn into_eval(mut self, scope: &mut Scope) -> Result<Self, EvalError> {
        self.eval(scope)?;
        Ok(self)
    }

    pub fn eval(&mut self, scope: &mut Scope) -> Result<(), EvalError> {
        match self {
            Expr::Float(_)
            | Expr::Moved
            | Expr::Int(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Char(_)
            | Expr::Unit => Ok(()),
            Expr::Array(items) => {
                for item in items {
                    item.eval(scope)?;
                }
                Ok(())
            }
            Expr::Object(items) => {
                for (key, value) in items {
                    value.eval(scope)?;
                }
                Ok(())
            }
            Expr::Option(item) => {
                if let Some(v) = &mut *item.as_mut() {
                    v.eval(scope)?;
                }
                Ok(())
            }
            Expr::Ident(name) => match scope.move_raw(&name) {
                None => Err(EvalError::MissingReference(name.to_string())),
                Some(expr) => {
                    *self = expr;
                    Ok(())
                }
            },
            Expr::Struct(name, items) => {
                for (key, value) in items {
                    value.eval(scope)?;
                }
                Ok(())
            }
            Expr::NamedTuple(name, items) => {
                for item in items {
                    item.eval(scope);
                }
                Ok(())
            }

            // some computation!
            Expr::Plus(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(*a + *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Float(*a + *b),
                    _ => return Err(EvalError::InvalidType("Cannot add")),
                };
                Ok(())
            }
            Expr::Minus(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(*a - *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Float(*a - *b),
                    _ => return Err(EvalError::InvalidType("Cannot subtract")),
                };
                Ok(())
            }
            Expr::Times(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(*a * *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Float(*a * *b),
                    _ => return Err(EvalError::InvalidType("Cannot multiply")),
                };
                Ok(())
            }
            Expr::Divide(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(*a / *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Float(*a / *b),
                    _ => return Err(EvalError::InvalidType("Cannot divide")),
                };
                Ok(())
            }

            Expr::Eq(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = Expr::Bool(a == b);
                Ok(())
            }

            Expr::Neq(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = Expr::Bool(a != b);
                Ok(())
            }

            Expr::Lt(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Bool(*a < *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Bool(*a < *b),
                    _ => return Err(EvalError::InvalidType("Cannot compare")),
                };
                Ok(())
            }

            Expr::Gt(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                *self = match (a.as_mut(), b.as_mut()) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Bool(*a > *b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Bool(*a > *b),
                    _ => return Err(EvalError::InvalidType("Cannot compare")),
                };
                Ok(())
            }

            //
            Expr::Block(stmts, last) => {
                // println!("Block start");
                let stmts = std::mem::replace(stmts, vec![]);

                let mut sub = scope.sub();
                for stmt in stmts {
                    // println!("Procesing {:?} : scope {:?}", stmt, scope.vbls.keys());
                    stmt.eval(&mut sub)?;
                }
                // println!("Block end");
                last.eval(&mut sub)?;
                *self = std::mem::replace(last, Expr::Unit);
                Ok(())
            }

            Expr::FnCall(name, args) => {
                let mut args = std::mem::replace(args, vec![]);
                for arg in args.iter_mut() {
                    arg.eval(scope)?;
                }
                *self = scope.call_fn_raw(&name, args)?;
                Ok(())
            }

            Expr::Cast(expr, typ) => {
                expr.eval(scope)?;
                *self = match (expr.as_mut(), typ) {
                    (Expr::Float(f), Type::I32) => Ok(Expr::Int(*f as i32)),
                    (Expr::Float(f), Type::F32) => Ok(Expr::Float(*f)),
                    (Expr::Int(i), Type::F32) => Ok(Expr::Float(*i as f32)),
                    (Expr::Int(i), Type::I32) => Ok(Expr::Int(*i)),
                    _ => Err(EvalError::InvalidType("Cannot cast")),
                }?;
                Ok(())
            }

            Expr::MemberAccess(expr, items) => {
                let mut target = match expr.as_mut() {
                    Expr::Ident(name) => {
                        let can_borrow = items.iter().any(|(_, x)| x.is_some());
                        if can_borrow {
                            let mut target = match scope.get_raw_mut(&name) {
                                None => return Err(EvalError::MissingReference(name.to_owned())),
                                Some(v) => v,
                            };
                            let mut items = items.into_iter();
                            let mut owned = loop {
                                if let Some((name, args)) = items.next() {
                                    if let Some(args) = args {
                                        break member_function(target, name, args.to_vec())?;
                                    } else {
                                        target = member_access(target, name)?;
                                    }
                                } else {
                                    unreachable!()
                                }
                            };
                            for (name, args) in items {
                                if let Some(args) = args.take() {
                                    owned = member_function(&mut owned, name, args)?;
                                } else {
                                    owned = member_move(owned, name)?;
                                }
                            }
                            *self = owned;
                            // do the borrow one I guess
                            return Ok(());
                        } else {
                            match scope.move_raw(&name) {
                                None => return Err(EvalError::MissingReference(name.to_owned())),
                                Some(v) => v,
                            }
                        }
                    }
                    _ => {
                        expr.eval(scope)?;
                        std::mem::replace(&mut **expr, Expr::Unit)
                    }
                };

                for (name, args) in items {
                    if let Some(args) = args.take() {
                        target = member_function(&mut target, name, args)?;
                    } else {
                        target = member_move(target, name)?;
                    }
                }
                *self = target;
                Ok(())
            }

            Expr::IfChain(chain, else_) => {
                for (cond, body) in chain {
                    match cond {
                        IfCond::Value(value) => {
                            value.eval(scope)?;
                            match value {
                                Expr::Bool(true) => {
                                    body.eval(scope)?;
                                    *self = std::mem::replace(body, Expr::Unit);
                                    return Ok(())
                                },
                                Expr::Bool(false) => (),
                                _ => return Err(EvalError::InvalidType("If condition must be a bool")),
                            };
                        },
                        IfCond::IfLet(pattern, value) => {
                            if let Some(bindings) = match_pattern(std::mem::replace(pattern, Pattern::Any), std::mem::replace(value, Expr::Unit)) {
                                let mut sub = scope.sub();
                                for (name, value) in bindings {
                                    sub.set_raw(&name, value)
                                }
                                return body.eval(&mut sub);
                            }
                        }
                    }
                }
                match else_.take() {
                    None => {
                        *self = Expr::Unit;
                        Ok(())
                    },
                    Some(mut block) => {
                        block.eval(scope)?;
                        *self = *block;
                        Ok(())
                    },
                }
            }

            Expr::Match(value, cases) => {
                value.eval(scope)?;
                for (pattern, body) in cases {
                    // TODO don't need to clone here, could return the value if unused
                    if let Some(bindings) = match_pattern(std::mem::replace(pattern, Pattern::Any), *value.clone()) {
                        let mut sub = scope.sub();
                        for (name, value) in bindings {
                            sub.set_raw(&name, value)
                        }
                        return body.eval(&mut sub);
                    }
                }
                Err(EvalError::Unmatched)
            }
        }
    }
}

/// TODO this allocates a bunch of empty vectors
fn match_pattern(pattern: Pattern, value: Expr) -> Option<Vec<(String, Expr)>> {
    match (pattern, value) {
        (Pattern::Any, _) => Some(vec![]),
        (Pattern::Ident(name), value) => Some(vec![(name, value)]),
        (Pattern::Const(Const::Bool(b)), Expr::Bool(bb)) if b == bb => Some(vec![]),
        (Pattern::Const(Const::Int(b)), Expr::Int(bb)) if b == bb => Some(vec![]),
        (Pattern::Const(Const::Float(b)), Expr::Float(bb)) if b == bb => Some(vec![]),
        (Pattern::Const(Const::String(ref b)), Expr::String(ref bb)) if b == bb => Some(vec![]),
        (Pattern::Const(Const::Char(b)), Expr::Char(bb)) if b == bb => Some(vec![]),
        (Pattern::Tuple(name, items), Expr::NamedTuple(bname, bitems)) => {
            if name == bname && items.len() == bitems.len() {
                let mut bindings = vec![];
                for (pat, val) in items.iter().zip(bitems) {
                    if let Some(inner) = match_pattern(pat.clone(), val) {
                        bindings.extend(inner)
                    } else {
                        return None;
                    }
                }
                Some(bindings)
            } else {
                None
            }
        }
        (Pattern::Struct(name, items), Expr::Struct(bname, bitems)) => {
            if name != bname {
                return None;
            }
            let mut bindings = vec![];
            for (ident, pat) in items {
                match bitems.iter().find(|(iname, _)| iname == &ident) {
                    None => return None,
                    Some((_, val)) => {
                        if let Some(inner) = match_pattern(pat, val.clone()) {
                            bindings.extend(inner);
                        } else {
                            return None;
                        }
                    }
                }
            }
            Some(bindings)
        }
        (pattern, value) => {
            // println!("No match {:?} - {:?}", pattern, value);
            None
        }
    }
}

fn member_move<'a>(value: Expr, name: &str) -> Result<Expr, EvalError> {
    Ok(match name.parse::<usize>() {
        Ok(index) => match value {
            Expr::Array(mut children) | Expr::NamedTuple(_, mut children) => children.remove(index),
            _ => {
                return Err(EvalError::InvalidType(
                    "Can only get index of array or namedtuple",
                ))
            }
        },
        Err(_) => match value {
            Expr::Object(children) | Expr::Struct(_, children) => {
                let mut found = false;
                for (sname, child) in children {
                    if sname == name {
                        return Ok(child);
                    }
                }
                return Err(EvalError::MissingMember(name.to_owned()));
            }
            _ => {
                return Err(EvalError::InvalidType(
                    "Can only get member of object or struct",
                ))
            }
        },
    })
}

fn member_access<'a>(value: &'a mut Expr, name: &str) -> Result<&'a mut Expr, EvalError> {
    Ok(match name.parse::<usize>() {
        Ok(index) => match value {
            Expr::Array(children) | Expr::NamedTuple(_, children) => &mut children[index],
            _ => {
                return Err(EvalError::InvalidType(
                    "Can only get index of array or namedtuple",
                ))
            }
        },
        Err(_) => match value {
            Expr::Object(children) | Expr::Struct(_, children) => {
                let mut found = false;
                for (sname, child) in children {
                    if sname == name {
                        return Ok(child);
                    }
                }
                return Err(EvalError::MissingMember(name.to_owned()));
            }
            _ => {
                return Err(EvalError::InvalidType(
                    "Can only get member of object or struct",
                ))
            }
        },
    })
}

fn member_function(value: &mut Expr, name: &str, mut args: Vec<Expr>) -> Result<Expr, EvalError> {
    Ok(match value {
        Expr::Array(items) => match name.as_ref() {
            "len" if args.is_empty() => Expr::Int(items.len() as i32),
            "push" => {
                if args.len() == 1 {
                    items.push(args.remove(0));
                    Expr::Unit
                } else {
                    return Err(EvalError::InvalidType("vec.push() takes a single argument"));
                }
            }
            _ => {
                println!("{} - {:?}", name, args);
                return Err(EvalError::InvalidType("unknown array fn"));
            }
        },
        Expr::Float(f) => match name.as_ref() {
            "sin" if args.is_empty() => Expr::Float(f.sin()),
            "cos" if args.is_empty() => Expr::Float(f.cos()),
            "tan" if args.is_empty() => Expr::Float(f.tan()),
            "abs" if args.is_empty() => Expr::Float(f.abs()),
            // "to_int" if args.is_empty() => Expr::Int(f as i32),
            _ => {
                println!("{} - {:?}", name, args);
                return Err(EvalError::InvalidType("unknown float fn"));
            }
        },
        Expr::Int(i) => match name.as_ref() {
            "to_float" if false => Expr::Float(*i as f32),
            _ => {
                println!("int {} - {:?}", name, args);
                return Err(EvalError::InvalidType("Unknown int fn"));
            }
        },
        _ => {
            println!("other {:?} : {} - {:?}", value, name, args);
            return Err(EvalError::InvalidType("Can only do fns on floats and ints"));
        }
    })
}
