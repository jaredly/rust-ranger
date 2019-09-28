use crate::scope::Scope;

pub type Args = Vec<String>;

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Let(String, Expr),
    Expr(Expr),
    FnDefn(String, Args, Expr),
}

impl Statement {
    pub fn eval(self, scope: &mut Scope) {
        match self {
            Statement::Let(name, v) => scope.set_raw(&name, v),
            Statement::Expr(e) => {
                let _ = e.eval(&scope);
            }
            Statement::FnDefn(name, args, body) => {
                scope.set_fn(&name, args, body)
                // scope.set_raw(&name, Expr::Lambda(args, Box::new(body)))
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type { F32, I32 }

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

    IfChain(Box<IfCond>, Box<Expr>, Vec<(IfCond, Expr)>, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
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
    IfLet(Pattern, Expr)
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

    pub fn eval(self, scope: &Scope) -> Result<Self, EvalError> {
        match self {
            Expr::Float(_)
            | Expr::Int(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Char(_)
            | Expr::Unit => Ok(self),
            Expr::Array(items) => {
                let mut res = vec![];
                for item in items {
                    res.push(item.eval(scope)?);
                }
                Ok(Expr::Array(res))
            }
            Expr::Object(items) => {
                let mut res = vec![];
                for (key, value) in items {
                    res.push((key, value.eval(scope)?));
                }
                Ok(Expr::Object(res))
            }
            Expr::Option(item) => Ok(Expr::Option(Box::new(
                item.map(|v| v.eval(scope)).transpose()?,
            ))),
            Expr::Ident(name) => scope
                .get_raw(&name)
                .map(|v| v.clone().eval(scope))
                .ok_or_else(|| EvalError::MissingReference(name.to_string()))?,
            Expr::Struct(name, items) => {
                let mut res = vec![];
                for (key, value) in items {
                    res.push((key, value.eval(scope)?));
                }
                Ok(Expr::Struct(name, res))
            }
            Expr::NamedTuple(name, items) => {
                let mut res = vec![];
                for item in items {
                    res.push(item.eval(scope)?);
                }
                Ok(Expr::NamedTuple(name, res))
            }

            // some computation!
            Expr::Plus(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a + b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a + b)),
                _ => Err(EvalError::InvalidType("Cannot add")),
            },
            Expr::Minus(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a - b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a - b)),
                _ => Err(EvalError::InvalidType("Cannot subtract")),
            },
            Expr::Times(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a * b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a * b)),
                _ => Err(EvalError::InvalidType("Cannot multiply")),
            },
            Expr::Divide(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a / b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a / b)),
                _ => Err(EvalError::InvalidType("Cannot divide")),
            },

            Expr::Eq(a, b) => Ok(Expr::Bool(a.eval(scope)? == b.eval(scope)?)),
            Expr::Neq(a, b) => Ok(Expr::Bool(a.eval(scope)? != b.eval(scope)?)),

            Expr::Lt(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Bool(a < b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Bool(a < b)),
                _ => Err(EvalError::InvalidType("Cannot compare")),
            },

            Expr::Gt(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Bool(a > b)),
                (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Bool(a > b)),
                _ => Err(EvalError::InvalidType("Cannot compare")),
            },

            //
            Expr::Block(stmts, last) => {
                let mut scope = scope.sub();
                for stmt in stmts {
                    stmt.eval(&mut scope);
                }
                last.eval(&scope)
            }

            Expr::FnCall(name, args) => scope.call_fn_raw(&name, args),

            Expr::Cast(expr, typ) => {
                let expr = expr.eval(&scope)?;
                match (expr, typ) {
                    (Expr::Float(f), Type::I32) => Ok(Expr::Int(f as i32)),
                    (Expr::Float(f), Type::F32) => Ok(Expr::Float(f)),
                    (Expr::Int(i), Type::F32) => Ok(Expr::Float(i as f32)),
                    (Expr::Int(i), Type::I32) => Ok(Expr::Int(i)),
                    _ => Err(EvalError::InvalidType("Cannot cast"))
                }
            }

            // Expr::MemberCall(expr, calls) => {
            //     let mut target = expr.eval(&scope)?;
            //     for (name, args) in calls {
            //     }
            //     Ok(target)
            // }

            Expr::MemberAccess(expr, items) => {
                let mut target = expr.eval(&scope)?;
                for (name, args) in items {
                    if let Some(args) = args {
                        target = match target {
                            Expr::Float(f) => match name.as_ref() {
                                "sin" if args.is_empty() => Expr::Float(f.sin()),
                                "cos" if args.is_empty() => Expr::Float(f.cos()),
                                "tan" if args.is_empty() => Expr::Float(f.tan()),
                                "abs" if args.is_empty() => Expr::Float(f.abs()),
                                // "to_int" if args.is_empty() => Expr::Int(f as i32),
                                _ => {
                                    println!("{} - {:?}", name, args);
                                    return Err(EvalError::InvalidType("unknown float fn"))
                                },
                            },
                            Expr::Int(i) => match name.as_ref() {
                                "to_float" if false => Expr::Float(i as f32),
                                _ => {
                                    println!("int {} - {:?}", name, args);
                                    return Err(EvalError::InvalidType("Unknown int fn"))
                                },
                            },
                            _ => {
                                    println!("other {:?} : {} - {:?}", target, name, args);
                                return Err(EvalError::InvalidType("Can only do fns on floats and ints"))
                            },
                        }
                    } else {
                        match name.parse::<usize>() {
                            Ok(index) => match target {
                                Expr::Array(children) | Expr::NamedTuple(_, children) => {
                                    target = children[index].clone()
                                }
                                _ => return Err(EvalError::InvalidType("Can only get index of array or namedtuple")),
                            },
                            Err(_) => match target {
                                Expr::Object(children) | Expr::Struct(_, children) => {
                                    let mut found = false;
                                    target = Expr::Unit;
                                    for (sname, child) in children {
                                        if sname == name {
                                            target = child;
                                            found = true;
                                            break;
                                        }
                                    }
                                    if !found {
                                        return Err(EvalError::MissingMember(name));
                                    }
                                }
                                _ => return Err(EvalError::InvalidType("Can only get member of object or struct")),
                            },
                        }
                    }
                }
                Ok(target)
            } // Expr::Lambda(args, block) => Err(EvalError::FunctionValue)
        }
    }
}
