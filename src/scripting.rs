use serde::Deserialize;
use std::collections::HashMap;

pub trait Animatable {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self;
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self;
    fn add(a: Self, b: Self) -> Self;
    fn mul(a: Self, b: Self) -> Self;
    fn abs(a: Self) -> Self;
}

impl Animatable for f32 {
    fn sin(center: f32, frequency: f32, amplitude: f32, offset: f32) -> f32 {
        (offset / frequency * std::f32::consts::PI * 2.0).sin() * amplitude + center
    }

    fn linear(from: f32, to: f32, speed: f32, offset: f32) -> f32 {
        let at = offset % (speed * 2.0);
        if at > speed {
            from + (to - from) * (at - speed) / speed
        } else {
            from + (to - from) * at / speed
        }
    }

    fn add(a: Self, b: Self) -> Self {
        a + b
    }

    fn mul(a: Self, b: Self) -> Self {
        a * b
    }

    fn abs(a: Self) -> Self {
        a.abs()
    }
}

impl<T: Animatable> Animatable for (T, T) {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self {
        (
            T::sin(center.0, frequency, amplitude.0, offset),
            T::sin(center.1, frequency, amplitude.1, offset),
        )
    }
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self {
        (
            T::linear(from.0, to.0, speed, offset),
            T::linear(from.1, to.1, speed, offset),
        )
    }
    fn add(a: Self, b: Self) -> Self {
        (Animatable::add(a.0, b.0), Animatable::add(a.1, b.1))
    }
    fn mul(a: Self, b: Self) -> Self {
        (Animatable::mul(a.0, b.0), Animatable::mul(a.1, b.1))
    }
    fn abs(a: Self) -> Self {
        (Animatable::abs(a.0), Animatable::abs(a.1))
    }
}

#[derive(Copy, Clone)]
pub struct Vbls {
    pub time: f32,
    pub vel: na::Vector2<f32>,
    // pub action:
}

pub type Shared = HashMap<String, Animated<f32>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Fn<T: Animatable + na::base::Scalar> {
    args: Vec<String>,
    body: Animated<T>,
}
pub type Fns = HashMap<String, Fn<f32>>;

#[derive(Debug, Deserialize, Clone)]
pub enum Bool<T: Animatable + na::base::Scalar> {
    True,
    False,
    Or(Box<Bool<T>>, Box<Bool<T>>),
    And(Box<Bool<T>>, Box<Bool<T>>),
    If(Box<Bool<T>>, Box<Bool<T>>, Box<Bool<T>>),
    Gt(Animated<T>, Animated<T>),
    Lt(Animated<T>, Animated<T>),
    Eq(Animated<T>, Animated<T>),
    StrEq { key: String, val: String },
    Within(Animated<T>, Animated<T>, Animated<T>),
}

impl Bool<f32> {
    // pub fn f() -> Self {
    //     Bool::False
    // }
    // pub fn t() -> Self {
    //     Bool::True
    // }
    pub fn eval(&self, ctx: &Context, args: &Vec<(String, f32)>) -> Result<bool, EvalErr> {
        Ok(match self {
            Bool::True => true,
            Bool::False => false,
            Bool::StrEq { key, val } => ctx.strings.get(key) == Some(val),
            Bool::If(a, b, c) => {
                if a.eval(ctx, args)? {
                    b.eval(ctx, args)?
                } else {
                    c.eval(ctx, args)?
                }
            }
            Bool::Or(a, b) => a.eval(ctx, args)? || b.eval(ctx, args)?,
            Bool::And(a, b) => a.eval(ctx, args)? && b.eval(ctx, args)?,
            Bool::Gt(a, b) => a.eval(ctx, args)? > b.eval(ctx, args)?,
            Bool::Lt(a, b) => a.eval(ctx, args)? < b.eval(ctx, args)?,
            Bool::Eq(a, b) => a.eval(ctx, args)? == b.eval(ctx, args)?,
            Bool::Within(a, b, c) => {
                (a.eval(ctx, args)? - b.eval(ctx, args)?).abs() < c.eval(ctx, args)?
            }
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum Simple<T: Clone> {
    None,
    Plain(T),
    Shared(String),
    IfStrEq {
        key: String,
        value: String,
        yes: Box<Simple<T>>,
        no: Box<Simple<T>>,
    },
    If(Bool<f32>, Box<Simple<T>>, Box<Simple<T>>),
    StrMatch(String, HashMap<String, Simple<T>>),
}

pub type SimpleShared<T> = HashMap<String, Simple<T>>;

pub struct SimpleContext<'a, T: Clone> {
    pub shared: &'a SimpleShared<T>,
}

impl<T: Clone> Simple<T> {
    pub fn eval(
        &self,
        ctx: &Context,
        simples: &SimpleContext<T>,
        args: &Vec<(String, f32)>,
    ) -> Result<Option<T>, EvalErr> {
        match self {
            Simple::If(cond, iftrue, iffalse) => {
                if cond.eval(ctx, args)? {
                    iftrue.eval(ctx, simples, args)
                } else {
                    iffalse.eval(ctx, simples, args)
                }
            }
            Simple::None => Ok(None),
            Simple::Plain(t) => Ok(Some(t.clone())),
            Simple::Shared(name) => match simples.shared.get(name) {
                None => Err(EvalErr::MissingShared(name.clone())),
                Some(t) => t.eval(ctx, simples, args),
            },
            Simple::IfStrEq {
                key,
                value,
                yes,
                no,
            } => {
                if ctx.strings.get(key) == Some(value) {
                    yes.eval(ctx, simples, args)
                } else {
                    no.eval(ctx, simples, args)
                }
            }
            Simple::StrMatch(key, map) => match ctx.strings.get(key) {
                None => Err(EvalErr::MissingShared(key.clone())),
                Some(v) => match map.get(v) {
                    None => Ok(None),
                    Some(v) => v.eval(ctx, simples, args),
                },
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum Animated<T: Animatable + na::base::Scalar> {
    Plain(T),
    Mul(Box<Animated<T>>, Box<Animated<T>>),
    Div(Box<Animated<T>>, Box<Animated<T>>),
    Inv(Box<Animated<T>>),
    Max(Box<Animated<T>>, Box<Animated<T>>),
    Min(Box<Animated<T>>, Box<Animated<T>>),
    If(Box<Bool<T>>, Box<Animated<T>>, Box<Animated<T>>),
    StrMatch(String, HashMap<String, Animated<T>>),
    PI,
    TAU,
    Add(Box<Animated<T>>, Box<Animated<T>>),
    Abs(Box<Animated<T>>),
    Shared(String),
    Var(String),
    Time,
    Vx,
    Vy,
    V,
    Sin(Box<Animated<T>>),
    Cos(Box<Animated<T>>),
    // fns
    Arg(String),
    Call(String, Vec<Animated<T>>),
}

#[derive(std::fmt::Debug)]
pub enum EvalErr {
    MissingShared(String),
    MissingVar(String),
    MissingArg(String),
    MissingFn(String),
}

pub struct Context<'a> {
    pub vbls: Vbls,
    pub shared: &'a Shared,
    pub strings: HashMap<String, String>,
    pub floats: HashMap<String, f32>,
    pub fns: &'a Fns,
}

impl Animated<f32> {
    pub fn zero() -> Animated<f32> {
        Animated::Plain(1.0)
    }
    pub fn one() -> Animated<f32> {
        Animated::Plain(1.0)
    }
    pub fn origin() -> (Animated<f32>, Animated<f32>) {
        (Animated::Plain(0.0), Animated::Plain(0.0))
    }
    pub fn eval(&self, ctx: &Context, args: &Vec<(String, f32)>) -> Result<f32, EvalErr> {
        Ok(match self {
            Animated::Call(name, call_args) => match ctx.fns.get(name) {
                None => return Err(EvalErr::MissingFn(name.clone())),
                Some(f) => {
                    let mut new_args = vec![];
                    for (argname, argv) in f.args.iter().zip(call_args.iter()) {
                        new_args.push((argname.clone(), argv.eval(ctx, args)?));
                    }
                    f.body.eval(ctx, &new_args)?
                }
            },
            Animated::If(cond, iftrue, iffalse) => {
                if cond.eval(ctx, args)? {
                    iftrue.eval(ctx, args)?
                } else {
                    iffalse.eval(ctx, args)?
                }
            }
            Animated::StrMatch(key, map) => match ctx.strings.get(key) {
                None => Err(EvalErr::MissingShared(key.clone())),
                Some(v) => match map.get(v) {
                    None => Err(EvalErr::MissingShared(key.clone())),
                    Some(v) => v.eval(ctx, args),
                },
            }?,
            Animated::Arg(name) => {
                for (n, a) in args {
                    if n == name {
                        return Ok(*a);
                    }
                }
                return Err(EvalErr::MissingArg(name.clone()));
            }
            // normal life
            Animated::Plain(t) => *t,
            Animated::Var(key) => match ctx.floats.get(key) {
                Some(v) => *v,
                None => return Err(EvalErr::MissingVar(key.clone())),
            },
            Animated::PI => std::f32::consts::PI,
            Animated::TAU => std::f32::consts::PI * 2.0,
            Animated::Time => ctx.vbls.time,
            Animated::Vx => ctx.vbls.vel.x,
            Animated::Vy => ctx.vbls.vel.y,
            Animated::V => ctx.vbls.vel.norm_squared().sqrt(),
            Animated::Mul(a, b) => a.eval(ctx, args)? * b.eval(ctx, args)?,
            Animated::Add(a, b) => a.eval(ctx, args)? + b.eval(ctx, args)?,
            Animated::Div(a, b) => a.eval(ctx, args)? / b.eval(ctx, args)?,
            Animated::Inv(a) => -(a.eval(ctx, args)?),
            Animated::Max(a, b) => a.eval(ctx, args)?.max(b.eval(ctx, args)?),
            Animated::Min(a, b) => a.eval(ctx, args)?.min(b.eval(ctx, args)?),
            Animated::Abs(a) => Animatable::abs(a.eval(ctx, args)?),
            Animated::Shared(key) => match ctx.shared.get(key) {
                Some(v) => v.eval(ctx, args)?,
                None => Err(EvalErr::MissingShared(key.clone()))?,
            },
            Animated::Sin(a) => a.eval(ctx, args)?.sin(),
            Animated::Cos(a) => a.eval(ctx, args)?.sin(),
        })
    }
}
