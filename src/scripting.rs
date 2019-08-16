use serde::Deserialize;
use std::{collections::HashMap, fs::File};

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
}

pub type Shared = HashMap<String, Animated<f32>>;

#[derive(Debug, Deserialize)]
pub struct Fn<T: Animatable + na::base::Scalar> {
    args: Vec<String>,
    body: Animated<T>,
}
pub type Fns = HashMap<String, Fn<f32>>;

#[derive(Debug, Deserialize)]
pub enum Animated<T: Animatable + na::base::Scalar> {
    Plain(T),
    Mul(Box<Animated<T>>, Box<Animated<T>>),
    Div(Box<Animated<T>>, Box<Animated<T>>),
    Inv(Box<Animated<T>>),
    Max(Box<Animated<T>>, Box<Animated<T>>),
    Min(Box<Animated<T>>, Box<Animated<T>>),
    PI,
    TAU,
    Add(Box<Animated<T>>, Box<Animated<T>>),
    Abs(Box<Animated<T>>),
    Shared(String),
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
    MissingArg(String),
    MissingFn(String),
}

pub struct Context<'a> {
    pub vbls: Vbls,
    pub shared: &'a Shared,
    pub fns: &'a Fns,
}

impl Animated<f32> {
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
