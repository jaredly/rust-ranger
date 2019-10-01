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
    ExprDesc(Expr),
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
            fns: vec![],
        }])
    }

    fn push(&mut self) {
        self.0.push(Locals {
            vbls: vec![],
            fns: vec![],
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
    pub fn walk<E, F: Fn(&mut Expr) -> Result<(), E>>(&mut self, f: &F) -> Result<(), E> {
        match self {
            Statement::Let(_, v) => v.walk(f),
            Statement::ExprDesc(v) => v.walk(f),
            Statement::FnDefn(_, _, body) => body.walk(f),
        }
    }

    pub fn move_nonlocal_vars(
        &mut self,
        local_vars: &mut LocalVars,
        scope: &mut Scope,
    ) -> Result<(), EvalError> {
        match self {
            Statement::Let(name, v) => {
                v.move_nonlocal_vars(local_vars, scope)?;
                local_vars.add(name);
            }
            Statement::ExprDesc(e) => {
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
            Statement::ExprDesc(mut e) => {
                e.eval(scope)?;
            }
            Statement::FnDefn(name, args, body) => {
                scope.set_fn(&name, args, body)
                // scope.set_raw(&name, ExprDesc::Lambda(args, Box::new(body)))
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

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Pos {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Default for Pos {
    fn default() -> Self {
        Pos {
            start: (0, 0),
            end: (0, 0),
        }
    }
}

impl<T: pest::RuleType> From<&pest::iterators::Pair<'_, T>> for Pos {
    fn from(other: &pest::iterators::Pair<T>) -> Pos {
        Pos::from(&other.as_span())
    }
}

impl From<&pest::Span<'_>> for Pos {
    fn from(span: &pest::Span) -> Pos {
        Pos {
            start: span.start_pos().line_col(),
            end: span.end_pos().line_col(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub desc: ExprDesc,
    pub pos: Pos,
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.desc == other.desc
    }
}

impl Expr {
    pub fn into_eval(mut self, scope: &mut Scope) -> Result<Self, EvalError> {
        self.eval(scope)?;
        Ok(self)
    }
}

impl std::ops::DerefMut for Expr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.desc
    }
}

impl std::ops::Deref for Expr {
    type Target = ExprDesc;

    fn deref(&self) -> &Self::Target {
        &self.desc
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExprDesc {
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

impl From<ExprDesc> for Expr {
    fn from(desc: ExprDesc) -> Self {
        Expr {
            desc,
            pos: Pos {
                start: (0, 0),
                end: (0, 0),
            },
        }
    }
}

impl From<f32> for Expr {
    fn from(i: f32) -> Self {
        ExprDesc::Float(i).into()
    }
}
impl From<i32> for Expr {
    fn from(i: i32) -> Self {
        ExprDesc::Int(i).into()
    }
}
impl From<bool> for Expr {
    fn from(i: bool) -> Self {
        ExprDesc::Bool(i).into()
    }
}
impl From<char> for Expr {
    fn from(i: char) -> Self {
        ExprDesc::Char(i).into()
    }
}
impl From<String> for Expr {
    fn from(i: String) -> Self {
        ExprDesc::String(i).into()
    }
}
impl<T> From<Vec<T>> for Expr
where
    T: Into<Expr>,
{
    fn from(i: Vec<T>) -> Self {
        ExprDesc::Array(i.into_iter().map(|t| t.into()).collect()).into()
    }
}

#[derive(Debug, Clone)]
pub struct EvalError {
    pub desc: EvalErrorDesc,
    pub pos: Pos,
}

impl PartialEq for EvalError {
    fn eq(&self, other: &Self) -> bool {
        self.desc == other.desc
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum EvalErrorDesc {
    InvalidType(&'static str),
    MissingMember(String),
    CannotGetMember(String, &'static str),
    MissingReference(String),
    FunctionValue,
    FunctionWrongNumberArgs(usize, usize),
    Unmatched,
}

impl From<EvalErrorDesc> for EvalError {
    fn from(other: EvalErrorDesc) -> Self {
        EvalError {
            desc: other,
            pos: Pos::default(),
        }
    }
}

impl EvalErrorDesc {
    pub fn with_pos(self, pos: Pos) -> EvalError {
        EvalError { desc: self, pos }
    }
}

impl Expr {
    pub fn array(arr: Vec<Expr>) -> Self {
        ExprDesc::Array(arr).into()
    }

    pub fn clear_pos(mut self) -> Self {
        let empty = crate::ast::Pos::default();
        let _ = self.walk::<(), _>(&|e: &mut Expr| {
            e.pos = empty;
            Ok(())
        });
        self
    }

    pub fn walk<E, F: Fn(&mut Self) -> Result<(), E>>(&mut self, f: &F) -> Result<(), E> {
        f(self)?;
        match &mut self.desc {
            ExprDesc::Float(_)
            | ExprDesc::Moved
            | ExprDesc::Int(_)
            | ExprDesc::Bool(_)
            | ExprDesc::String(_)
            | ExprDesc::Char(_)
            | ExprDesc::Ident(_)
            | ExprDesc::Unit => (),
            ExprDesc::Tuple(items) | ExprDesc::Array(items) => {
                for item in items {
                    item.walk(f)?;
                }
            }
            ExprDesc::Object(items) => {
                for (_key, value) in items {
                    value.walk(f)?;
                }
            }
            ExprDesc::Option(item) => {
                if let Some(v) = &mut *item.as_mut() {
                    v.walk(f)?;
                }
            }
            ExprDesc::Struct(_name, items) => {
                for (_key, value) in items {
                    value.walk(f)?;
                }
            }
            ExprDesc::NamedTuple(_name, items) => {
                for item in items {
                    item.walk(f)?;
                }
            }

            // some computation!
            ExprDesc::Plus(a, b)
            | ExprDesc::Minus(a, b)
            | ExprDesc::Times(a, b)
            | ExprDesc::Divide(a, b)
            | ExprDesc::Eq(a, b)
            | ExprDesc::Neq(a, b)
            | ExprDesc::Lt(a, b)
            | ExprDesc::Gt(a, b) => {
                a.walk(f)?;
                b.walk(f)?;
            }
            ExprDesc::Block(stmts, last) => {
                for stmt in stmts {
                    stmt.walk(f)?;
                }
                last.walk(f)?;
            }

            ExprDesc::FnCall(_name, args) => {
                for arg in args.iter_mut() {
                    arg.walk(f)?;
                }
            }

            ExprDesc::Cast(expr, _typ) => {
                expr.walk(f)?;
            }

            ExprDesc::MemberAccess(expr, _items) => {
                expr.walk(f)?;
            }

            ExprDesc::IfChain(chain, else_) => {
                for (cond, body) in chain {
                    body.walk(f)?;
                    match cond {
                        IfCond::Value(value) => {
                            value.walk(f)?;
                        }
                        IfCond::IfLet(_pattern, value) => {
                            value.walk(f)?;
                        }
                    }
                }
                match else_.take() {
                    None => (),
                    Some(mut block) => {
                        block.walk(f)?;
                    }
                }
            }

            ExprDesc::Match(value, cases) => {
                value.walk(f)?;
                for (_pattern, body) in cases {
                    body.walk(f)?;
                }
            }
        }
        Ok(())
    }

    pub fn eval(&mut self, scope: &mut Scope) -> Result<(), EvalError> {
        match &mut self.desc {
            ExprDesc::Float(_)
            | ExprDesc::Moved
            | ExprDesc::Int(_)
            | ExprDesc::Bool(_)
            | ExprDesc::String(_)
            | ExprDesc::Char(_)
            | ExprDesc::Unit => Ok(()),
            ExprDesc::Tuple(items) | ExprDesc::Array(items) => {
                for item in items {
                    item.eval(scope)?;
                }
                Ok(())
            }
            ExprDesc::Object(items) => {
                for (_key, value) in items {
                    value.eval(scope)?;
                }
                Ok(())
            }
            ExprDesc::Option(item) => {
                if let Some(v) = &mut *item.as_mut() {
                    v.eval(scope)?;
                }
                Ok(())
            }
            ExprDesc::Ident(name) => match scope.move_raw(&name) {
                None => Err(EvalErrorDesc::MissingReference(name.to_string()).with_pos(self.pos)),
                Some(expr) => {
                    *self = expr;
                    Ok(())
                }
            },
            ExprDesc::Struct(_name, items) => {
                for (_key, value) in items {
                    value.eval(scope)?;
                }
                Ok(())
            }
            ExprDesc::NamedTuple(_name, items) => {
                for item in items {
                    item.eval(scope)?;
                }
                Ok(())
            }

            // some computation!
            ExprDesc::Plus(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Int(*a + *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Float(*a + *b),
                    _ => return Err(EvalErrorDesc::InvalidType("Cannot add").with_pos(self.pos)),
                };
                Ok(())
            }
            ExprDesc::Minus(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Int(*a - *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Float(*a - *b),
                    _ => {
                        return Err(EvalErrorDesc::InvalidType("Cannot subtract").with_pos(self.pos))
                    }
                };
                Ok(())
            }
            ExprDesc::Times(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Int(*a * *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Float(*a * *b),
                    _ => {
                        return Err(EvalErrorDesc::InvalidType("Cannot multiply").with_pos(self.pos))
                    }
                };
                Ok(())
            }
            ExprDesc::Divide(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Int(*a / *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Float(*a / *b),
                    _ => return Err(EvalErrorDesc::InvalidType("Cannot divide").with_pos(self.pos)),
                };
                Ok(())
            }

            ExprDesc::Eq(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                println!("Eq check: {:?} == {:?}", a, b);
                self.desc = ExprDesc::Bool(a == b);
                Ok(())
            }

            ExprDesc::Neq(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = ExprDesc::Bool(a != b);
                Ok(())
            }

            ExprDesc::Lt(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Bool(*a < *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Bool(*a < *b),
                    _ => {
                        return Err(EvalErrorDesc::InvalidType("Cannot compare").with_pos(self.pos))
                    }
                };
                Ok(())
            }

            ExprDesc::Gt(a, b) => {
                a.eval(scope)?;
                b.eval(scope)?;
                self.desc = match (&mut a.as_mut().desc, &mut b.as_mut().desc) {
                    (ExprDesc::Int(a), ExprDesc::Int(b)) => ExprDesc::Bool(*a > *b),
                    (ExprDesc::Float(a), ExprDesc::Float(b)) => ExprDesc::Bool(*a > *b),
                    _ => {
                        return Err(EvalErrorDesc::InvalidType("Cannot compare").with_pos(self.pos))
                    }
                };
                Ok(())
            }

            //
            ExprDesc::Block(stmts, last) => {
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
                self.desc = std::mem::replace(last, ExprDesc::Unit);
                Ok(())
            }

            ExprDesc::FnCall(name, args) => {
                for arg in args.iter_mut() {
                    arg.eval(scope)?;
                }
                println!("Fn Call {:?}", args);
                let args = std::mem::replace(args, vec![]);
                self.desc = scope.call_fn_raw(&name, args, self.pos)?.desc;
                Ok(())
            }

            ExprDesc::Cast(expr, typ) => {
                expr.eval(scope)?;
                self.desc = match (&mut expr.as_mut().desc, typ) {
                    (ExprDesc::Float(f), Type::I32) => Ok(ExprDesc::Int(*f as i32)),
                    (ExprDesc::Float(f), Type::F32) => Ok(ExprDesc::Float(*f)),
                    (ExprDesc::Int(i), Type::F32) => Ok(ExprDesc::Float(*i as f32)),
                    (ExprDesc::Int(i), Type::I32) => Ok(ExprDesc::Int(*i)),
                    _ => Err(EvalErrorDesc::InvalidType("Cannot cast")),
                }?;
                Ok(())
            }

            ExprDesc::MemberAccess(expr, items) => {
                let mut target = match &mut expr.as_mut().desc {
                    ExprDesc::Ident(name) => {
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
                                None => {
                                    return Err(EvalErrorDesc::MissingReference(name.to_owned())
                                        .with_pos(self.pos))
                                }
                                Some(v) => v,
                            };
                            let mut items = items.into_iter();
                            let mut owned = loop {
                                if let Some((name, args)) = items.next() {
                                    if let Some(args) = args {
                                        break member_function(
                                            target,
                                            name,
                                            std::mem::replace(args, vec![]),
                                        )?;
                                    } else {
                                        target = member_access(target, name, self.pos)?;
                                    }
                                } else {
                                    unreachable!()
                                }
                            };
                            for (name, args) in items {
                                if let Some(args) = args.take() {
                                    owned = member_function(&mut owned, name, args)?;
                                } else {
                                    owned = member_move(owned, name, self.pos)?;
                                }
                            }
                            // TODO preserve location?
                            *self = owned;
                            // do the borrow one I guess
                            return Ok(());
                        } else {
                            match scope.move_raw(&name) {
                                None => {
                                    return Err(EvalErrorDesc::MissingReference(name.to_owned())
                                        .with_pos(self.pos))
                                }
                                Some(v) => v,
                            }
                        }
                    }
                    _ => {
                        expr.eval(scope)?;
                        std::mem::replace(&mut **expr, ExprDesc::Unit.into())
                    }
                };

                for (name, args) in items {
                    if let Some(args) = args.take() {
                        target = member_function(&mut target, name, args)?;
                    } else {
                        target = member_move(target, name, self.pos)?;
                    }
                }
                *self = target;
                Ok(())
            }

            ExprDesc::IfChain(chain, else_) => {
                for (cond, body) in chain {
                    match cond {
                        IfCond::Value(value) => {
                            println!("if cond {:?}", value);
                            value.eval(scope)?;
                            match value.desc {
                                ExprDesc::Bool(true) => {
                                    body.eval(scope)?;
                                    self.desc = std::mem::replace(body, ExprDesc::Unit);
                                    return Ok(());
                                }
                                ExprDesc::Bool(false) => {
                                    println!("if cond faaallthrough {:?}", value);
                                    ()
                                }
                                _ => {
                                    return Err(EvalErrorDesc::InvalidType(
                                        "If condition must be a bool",
                                    )
                                    .with_pos(self.pos))
                                }
                            };
                        }
                        IfCond::IfLet(pattern, value) => {
                            if let Some(bindings) = match_pattern(
                                std::mem::replace(pattern, Pattern::Any),
                                std::mem::replace(value, ExprDesc::Unit.into()),
                            ) {
                                scope.push();
                                // let mut sub = scope.sub();
                                for (name, value) in bindings {
                                    scope.set_raw(&name, value)
                                }
                                body.eval(scope)?;
                                self.desc = std::mem::replace(body, ExprDesc::Unit);
                                scope.pop();
                                return Ok(());
                            }
                        }
                    }
                }
                match else_.take() {
                    None => {
                        self.desc = ExprDesc::Unit;
                        Ok(())
                    }
                    Some(mut block) => {
                        block.eval(scope)?;
                        *self = *block;
                        Ok(())
                    }
                }
            }

            ExprDesc::Match(value, cases) => {
                value.eval(scope)?;
                for (pattern, body) in cases {
                    // TODO don't need to clone here, could return the value if unused
                    if let Some(bindings) =
                        match_pattern(std::mem::replace(pattern, Pattern::Any), *value.clone())
                    {
                        scope.push();
                        // let mut sub = scope.sub();
                        for (name, value) in bindings {
                            scope.set_raw(&name, value)
                        }
                        body.eval(scope)?;
                        self.desc = std::mem::replace(body, ExprDesc::Moved);
                        scope.pop();
                        return Ok(());
                    }
                }
                Err(EvalErrorDesc::Unmatched.with_pos(self.pos))
            }
        }
    }

    pub fn move_nonlocal_vars(
        &mut self,
        local_vars: &mut LocalVars,
        scope: &mut Scope,
    ) -> Result<(), EvalError> {
        match &mut self.desc {
            ExprDesc::Float(_)
            | ExprDesc::Moved
            | ExprDesc::Int(_)
            | ExprDesc::Bool(_)
            | ExprDesc::String(_)
            | ExprDesc::Char(_)
            | ExprDesc::Unit => Ok(()),
            ExprDesc::Tuple(items) | ExprDesc::Array(items) => {
                for item in items {
                    item.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            ExprDesc::Object(items) => {
                for (_key, value) in items {
                    value.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            ExprDesc::Option(item) => {
                if let Some(v) = &mut *item.as_mut() {
                    v.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            ExprDesc::Ident(name) => {
                if !local_vars.check(name) {
                    match scope.move_raw(&name) {
                        None => {
                            return Err(EvalErrorDesc::MissingReference(name.to_string())
                                .with_pos(self.pos))
                        }
                        Some(expr) => {
                            *self = expr;
                        }
                    }
                }
                Ok(())
            }
            ExprDesc::Struct(_name, items) => {
                for (_key, value) in items {
                    value.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }
            ExprDesc::NamedTuple(_name, items) => {
                for item in items {
                    item.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }

            // some computation!
            ExprDesc::Plus(a, b)
            | ExprDesc::Minus(a, b)
            | ExprDesc::Times(a, b)
            | ExprDesc::Divide(a, b)
            | ExprDesc::Eq(a, b)
            | ExprDesc::Neq(a, b)
            | ExprDesc::Lt(a, b)
            | ExprDesc::Gt(a, b) => {
                a.move_nonlocal_vars(local_vars, scope)?;
                b.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            //
            ExprDesc::Block(stmts, last) => {
                local_vars.push();
                for stmt in stmts {
                    stmt.move_nonlocal_vars(local_vars, scope)?;
                }
                last.move_nonlocal_vars(local_vars, scope)?;
                local_vars.pop();
                Ok(())
            }

            ExprDesc::FnCall(_name, args) => {
                for arg in args.iter_mut() {
                    arg.move_nonlocal_vars(local_vars, scope)?;
                }
                Ok(())
            }

            ExprDesc::Cast(expr, _typ) => {
                expr.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            ExprDesc::MemberAccess(expr, items) => {
                // if it's a .clone(), then don't move. Otherwise, we go ahead and move.
                if let ExprDesc::Ident(ident) = &mut expr.as_mut().desc {
                    if let Some(args) = &items[0].1 {
                        if items[0].0 == "clone" && args.is_empty() {
                            if let Some(expr) = scope.move_raw(&ident) {
                                items.remove(0);
                                // its a clone
                                *self = ExprDesc::MemberAccess(
                                    Box::new(expr),
                                    std::mem::replace(items, vec![]),
                                )
                                .with_pos(self.pos);
                                return Ok(());
                            }
                        }
                    }
                }
                expr.move_nonlocal_vars(local_vars, scope)?;
                Ok(())
            }

            ExprDesc::IfChain(chain, else_) => {
                for (cond, body) in chain {
                    match cond {
                        IfCond::Value(_) => {
                            body.move_nonlocal_vars(local_vars, scope)?;
                        }
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
                            return Ok(());
                        }
                    }
                }
                match else_.as_mut() {
                    None => (),
                    Some(expr) => expr.move_nonlocal_vars(local_vars, scope)?,
                }
                Ok(())
            }

            ExprDesc::Match(value, cases) => {
                value.eval(scope)?;
                for (pattern, body) in cases {
                    let mut bindings = vec![];
                    pattern_names(pattern, &mut bindings);
                    local_vars.push();
                    for name in bindings {
                        local_vars.add(&name);
                    }
                    body.move_nonlocal_vars(local_vars, scope)?;
                    local_vars.pop();
                }
                Ok(())
            }
        }
    }
}

impl ExprDesc {
    pub fn kind(&self) -> &'static str {
        match self {
            ExprDesc::Float(_) => "float",
            ExprDesc::Int(_) => "int",
            ExprDesc::Bool(_) => "bool",
            ExprDesc::Char(_) => "char",
            ExprDesc::String(_) => "string",

            ExprDesc::Array(_) => "array",
            ExprDesc::Tuple(_) => "tuple",
            ExprDesc::Object(_) => "object",
            ExprDesc::Option(_) => "option",
            ExprDesc::Ident(_) => "ident",

            ExprDesc::Unit => "unit",
            ExprDesc::Struct(_, _) => "struct",
            ExprDesc::NamedTuple(_, _) => "named tuple",

            ExprDesc::Plus(_, _) => "plus",
            ExprDesc::Minus(_, _) => "minus",
            ExprDesc::Times(_, _) => "times",
            ExprDesc::Divide(_, _) => "divide",

            ExprDesc::Eq(_, _) => "==",
            ExprDesc::Neq(_, _) => "!=",
            ExprDesc::Lt(_, _) => "<",
            ExprDesc::Gt(_, _) => ">",

            ExprDesc::MemberAccess(_, _) => "member access",
            ExprDesc::Cast(_, _) => " as ",

            ExprDesc::Block(_, _) => "block",
            ExprDesc::FnCall(_, _) => "call()",

            ExprDesc::IfChain(_, _) => "if",
            ExprDesc::Match(_, _) => "match",

            ExprDesc::Moved => "moved value",
        }
    }

    pub fn with_span(self, span: &pest::Span) -> Expr {
        Expr {
            desc: self,
            pos: span.into(),
        }
    }

    pub fn with_pos(self, pos: Pos) -> Expr {
        Expr { desc: self, pos }
    }

    pub fn match_pos(self, other: &Expr) -> Expr {
        Expr {
            desc: self,
            pos: other.pos.clone(),
        }
    }
    pub fn needs_evaluation(&self) -> bool {
        match self {
            ExprDesc::Float(_)
            | ExprDesc::Int(_)
            | ExprDesc::Bool(_)
            | ExprDesc::String(_)
            | ExprDesc::Char(_) => false,
            ExprDesc::NamedTuple(_, items) | ExprDesc::Array(items) | ExprDesc::Tuple(items) => {
                items.iter().any(|e| e.desc.needs_evaluation())
            }
            ExprDesc::Struct(_, items) | ExprDesc::Object(items) => {
                items.iter().any(|(_, expr)| expr.desc.needs_evaluation())
            }
            ExprDesc::Option(inner) => inner
                .as_ref()
                .as_ref()
                .map_or(false, |expr| expr.desc.needs_evaluation()),
            _ => true,
        }
    }
}

/// TODO this allocates a bunch of empty vectors
fn match_pattern(pattern: Pattern, value: Expr) -> Option<Vec<(String, Expr)>> {
    match (pattern, value) {
        (Pattern::Any, _) => Some(vec![]),
        (Pattern::Ident(name), value) => Some(vec![(name, value)]),
        (
            Pattern::Const(Const::Bool(b)),
            Expr {
                desc: ExprDesc::Bool(bb),
                ..
            },
        ) if b == bb => Some(vec![]),
        (
            Pattern::Const(Const::Int(b)),
            Expr {
                desc: ExprDesc::Int(bb),
                ..
            },
        ) if b == bb => Some(vec![]),
        (
            Pattern::Const(Const::Float(b)),
            Expr {
                desc: ExprDesc::Float(bb),
                ..
            },
        ) if b == bb => Some(vec![]),
        (
            Pattern::Const(Const::String(ref b)),
            Expr {
                desc: ExprDesc::String(ref bb),
                ..
            },
        ) if b == bb => Some(vec![]),
        (
            Pattern::Const(Const::Char(b)),
            Expr {
                desc: ExprDesc::Char(bb),
                ..
            },
        ) if b == bb => Some(vec![]),
        (
            Pattern::Tuple(name, items),
            Expr {
                desc: ExprDesc::NamedTuple(bname, bitems),
                ..
            },
        ) => {
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
        (
            Pattern::Struct(name, items),
            Expr {
                desc: ExprDesc::Struct(bname, bitems),
                ..
            },
        ) => {
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
        (_pattern, _value) => {
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

fn member_move<'a>(value: Expr, name: &str, pos: Pos) -> Result<Expr, EvalError> {
    Ok(match name.parse::<usize>() {
        Ok(index) => match value.desc {
            ExprDesc::Array(mut children) | ExprDesc::NamedTuple(_, mut children) => {
                children.remove(index)
            }
            _ => {
                return Err(EvalErrorDesc::InvalidType(
                    "Can only get index of array or namedtuple",
                ).with_pos(pos))
            }
        },
        Err(_) => match value.desc {
            ExprDesc::Object(children) | ExprDesc::Struct(_, children) => {
                for (sname, child) in children {
                    if sname == name {
                        return Ok(child);
                    }
                }
                return Err(EvalErrorDesc::MissingMember(name.to_owned()).with_pos(pos));
            }
            _ => {
                return Err(EvalErrorDesc::CannotGetMember(
                    name.to_owned(),
                    value.desc.kind(),
                ).with_pos(pos))
            }
        },
    })
}

fn member_access<'a>(value: &'a mut Expr, name: &str, pos: Pos) -> Result<&'a mut Expr, EvalError> {
    let kind = value.desc.kind();
    Ok(match name.parse::<usize>() {
        Ok(index) => match &mut value.desc {
            ExprDesc::Array(children) | ExprDesc::NamedTuple(_, children) => &mut children[index],
            _ => {
                return Err(EvalErrorDesc::InvalidType(
                    "Can only get index of array or namedtuple",
                ).with_pos(pos))
            }
        },
        Err(_) => match &mut value.desc {
            ExprDesc::Object(children) | ExprDesc::Struct(_, children) => {
                for (sname, child) in children {
                    if sname == name {
                        return Ok(child);
                    }
                }
                return Err(EvalErrorDesc::MissingMember(name.to_owned()).with_pos(pos));
            }
            _ => {
                return Err(EvalErrorDesc::CannotGetMember(
                    name.to_owned(),
                    kind,
                ).with_pos(pos))
            }
        },
    })
}

fn member_function(
    value: &mut Expr,
    name: &str,
    mut args: Vec<Expr>,
) -> Result<Expr, EvalErrorDesc> {
    if name == "clone" {
        return Ok(value.clone());
    }
    Ok(match &mut value.desc {
        ExprDesc::Array(items) => match name.as_ref() {
            "len" if args.is_empty() => ExprDesc::Int(items.len() as i32),
            "push" => {
                if args.len() == 1 {
                    items.push(args.remove(0));
                    ExprDesc::Unit
                } else {
                    return Err(EvalErrorDesc::InvalidType(
                        "vec.push() takes a single argument",
                    ));
                }
            }
            _ => {
                println!("{} - {:?}", name, args);
                return Err(EvalErrorDesc::InvalidType("unknown array fn"));
            }
        },
        ExprDesc::Float(f) => match name.as_ref() {
            "sin" if args.is_empty() => ExprDesc::Float(f.sin()),
            "cos" if args.is_empty() => ExprDesc::Float(f.cos()),
            "tan" if args.is_empty() => ExprDesc::Float(f.tan()),
            "abs" if args.is_empty() => ExprDesc::Float(f.abs()),
            "atan2" if args.len() == 1 => match args[0].desc {
                ExprDesc::Float(x) => ExprDesc::Float(f.atan2(x)),
                _ => return Err(EvalErrorDesc::InvalidType("atan2 takes a float argument")),
            },
            // "to_int" if args.is_empty() => ExprDesc::Int(f as i32),
            _ => {
                println!("{} - {:?}", name, args);
                return Err(EvalErrorDesc::InvalidType("unknown float fn"));
            }
        },
        ExprDesc::Int(i) => match name.as_ref() {
            "to_float" if false => ExprDesc::Float(*i as f32),
            _ => {
                println!("int {} - {:?}", name, args);
                return Err(EvalErrorDesc::InvalidType("Unknown int fn"));
            }
        },
        _ => {
            println!("other {:?} : {} - {:?}", value, name, args);
            return Err(EvalErrorDesc::InvalidType(
                "Can only do fns on floats and ints",
            ));
        }
    }
    .match_pos(value))
}
