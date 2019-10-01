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

pub struct Locals {
    vbls: Vec<String>,
    fns: Vec<String>,
}
pub struct LocalVars(Vec<Locals>);
impl LocalVars {
    fn new() -> Self {
        LocalVars(vec![Locals {
            vbls: vec![],
            fns: vec![]
        }])
    }

    fn push(&mut self) {
        self.0.push(Locals {
            vbls: vec![],
            fns: vec![]
        });
    }
    fn pop(&mut self) {
        self.0.pop();
    }

    fn last(&mut self) -> &mut Locals {
        let ln = self.0.len();
        &mut self.0[ln - 1]
    }

    fn add(&mut self, key: &str) {
        self.last().vbls.push(key.to_owned());
    }
    fn add_fn(&mut self, key: &str) {
        self.last().fns.push(key.to_owned());
    }
    fn check(&mut self, key: &str) -> bool {
        self.last().vbls.contains(&key.to_owned())
    }
    fn check_fn(&mut self, key: &str) -> bool {
        self.last().fns.contains(&key.to_owned())
    }

}

impl Statement {
    pub fn move_nonlocal_vars(&mut self, local_vars: &mut LocalVars, scope: &mut Scope) -> Result<(), EvalError> {
        match self {
            Statement::Let(name, v) => {
                v.move_nonlocal_vars(local_vars, scope)?;
                local_vars.add(name);
            }
            Statement::Expr(e) => {
                e.move_nonlocal_vars(local_vars, scope)?;
            }
            Statement::FnDefn(name, _args, _body) => {
                local_vars.add_fn(name);
            }
        }
        Ok(())
    }

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
    Tuple(Vec<Expr>),
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
    pub fn move_nonlocal_vars(&mut self, local_vars: &mut LocalVars, scope: &mut Scope) -> Result<(), EvalError> {
        match self {
            Expr::Float(_)
            | Expr::Moved
            | Expr::Int(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Char(_)
            | Expr::Unit => Ok(()),
            Expr::Tuple(items) |
            Expr::Array(items) => {
                for item in items {
                    item.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            Expr::Object(items) => {
                for (_key, value) in items {
                    value.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            Expr::Option(item) => {
                if let Some(v) = &mut *item.as_mut() {
                    v.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            Expr::Ident(name) => {
                if !local_vars.check(name) {
                    match scope.move_raw(&name) {
                        None => return Err(EvalError::MissingReference(name.to_string())),
                        Some(expr) => {
                            *self = expr;
                        }
                    }
                }
                Ok(())
            },
            Expr::Struct(_name, items) => {
                for (_key, value) in items {
                    value.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            Expr::NamedTuple(_name, items) => {
                for item in items {
                    item.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }

            // some computation!
            Expr::Plus(a, b) |
            Expr::Minus(a, b) |
            Expr::Times(a, b) |
            Expr::Divide(a, b) |
            Expr::Eq(a, b) |
            Expr::Neq(a, b) |
            Expr::Lt(a, b) |
            Expr::Gt(a, b) => {
                a.move_nonlocal_vars(local_vars, scope)?;
                b.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            //
            Expr::Block(stmts, last) => {
                local_vars.push();
                for stmt in stmts {
                    stmt.move_nonlocal_vars(local_vars, scope)?;
                }
                last.move_nonlocal_vars(local_vars, scope)?;
                local_vars.pop();
                Ok(())
            }

            Expr::FnCall(_name, args) => {
                for arg in args.iter_mut() {
                    arg.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }

            Expr::Cast(expr, _typ) => {
                expr.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            Expr::MemberAccess(expr, items) => {
                // if it's a .clone(), then don't move. Otherwise, we go ahead and move.
                if let Expr::Ident(ident) = expr.as_mut() {
                    if let Some(args) = &items[0].1 {
                        if items[0].0 == "clone" && args.is_empty() {
                            if let Some(expr) = scope.move_raw(ident) {
                                items.remove(0);
                                // its a clone
                                *self = Expr::MemberAccess(
                                    Box::new(expr),
                                    std::mem::replace(items, vec![])
                                );
                                return Ok(())
                            }
                        }
                    }
                }
                expr.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            Expr::IfChain(chain, else_) => {
                for (cond, body) in chain {
                    match cond {
                        IfCond::Value(_) => {
                            body.move_nonlocal_vars(local_vars, scope)?;
                        },
                        IfCond::IfLet(pattern, _value) => {
                            let mut bindings = vec![];
                            pattern_names(pattern, &mut bindings);

                            local_vars.push();
                            // let mut sub = scope.sub();
                            for name in bindings {
                                local_vars.add(&name);
                            }
                            body.move_nonlocal_vars(local_vars, scope)?;
                            local_vars.pop();
                            return Ok(())
                        }
                    }
                }
                match else_.as_mut() {
                    None => (),
                    Some(expr) => expr.move_nonlocal_vars(local_vars, scope)?,
                }
                Ok(())
            }

            Expr::Match(value, cases) => {
                value.eval(scope)?;
                for (pattern, body) in cases {
                    let mut bindings = vec![];
                    pattern_names(pattern, &mut bindings);
                    // TODO don't need to clone here, could return the value if unused
                    local_vars.push();
                    // let mut sub = scope.sub();
                    for name in bindings {
                        local_vars.add(&name);
                    }
                    body.move_nonlocal_vars(local_vars, scope)?;
                    local_vars.pop();
                    return Ok(());
                }
                Err(EvalError::Unmatched)
            }

        }
    }

    pub fn needs_evaluation(&self) -> bool {
        match self {
            Expr::Float(_) | Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Char(_) => {
                false
            }
            Expr::NamedTuple(_, items) | Expr::Array(items) | Expr::Tuple(items) => {
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
            Expr::Tuple(items) |
            Expr::Array(items) => {
                for item in items {
                    item.eval(scope)?;
                }
                Ok(())
            }
            Expr::Object(items) => {
                for (_key, value) in items {
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
            Expr::Struct(_name, items) => {
                for (_key, value) in items {
                    value.eval(scope)?;
                }
                Ok(())
            }
            Expr::NamedTuple(_name, items) => {
                for item in items {
                    item.eval(scope)?;
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

                // let mut sub = scope.sub();
                scope.push();
                for stmt in stmts {
                    // println!("Procesing {:?} : scope {:?}", stmt, scope.vbls.keys());
                    stmt.eval(scope)?;
                }
                // println!("Block end");
                last.eval(scope)?;
                scope.pop();
                *self = std::mem::replace(last, Expr::Unit);
                Ok(())
            }

            Expr::FnCall(name, args) => {
                for arg in args.iter_mut() {
                    arg.eval(scope)?;
                }
                println!("Fn Call {:?}", args);
                let args = std::mem::replace(args, vec![]);
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
                            for (_name, args) in items.iter_mut() {
                                if let Some(args) = args {
                                    for arg in args {
                                        arg.eval(scope)?;
                                    }
                                }
                            }
                            let mut target = match scope.get_raw_mut(&name) {
                                None => return Err(EvalError::MissingReference(name.to_owned())),
                                Some(v) => v,
                            };
                            let mut items = items.into_iter();
                            let mut owned = loop {
                                if let Some((name, args)) = items.next() {
                                    if let Some(args) = args {
                                        break member_function(target, name, std::mem::replace(args, vec![]))?;
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
                                scope.push();
                                // let mut sub = scope.sub();
                                for (name, value) in bindings {
                                    scope.set_raw(&name, value)
                                }
                                body.eval(scope)?;
                                *self = std::mem::replace(body, Expr::Unit);
                                scope.pop();
                                return Ok(())
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
                        scope.push();
                        // let mut sub = scope.sub();
                        for (name, value) in bindings {
                            scope.set_raw(&name, value)
                        }
                        body.eval(scope)?;
                        *self = std::mem::replace(body, Expr::Moved);
                        scope.pop();
                        return Ok(());
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

fn pattern_names(pattern: &Pattern, vbls: &mut Vec<String>) {
    match pattern {
        Pattern::Any => (),
        Pattern::Ident(name) => vbls.push(name.to_owned()),
        Pattern::Const(_) => (),
        Pattern::Tuple(_name, items) => {
            for item in items {
                pattern_names(item, vbls);
            }
        }
        Pattern::Struct(_name, items) => {
            for (_ident, pat) in items {
                pattern_names(pat, vbls);
            }
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
    if name == "clone" {
        return Ok(value.clone())
    }
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
            "atan2" if args.len() == 1 => match args[0] {
                Expr::Float(x) => Expr::Float(f.atan2(x)),
                _ => return Err(EvalError::InvalidType("atan2 takes a float argument"))
            },
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
