use std::{collections::HashMap, fmt::Debug};

use serde_json::Value;

pub trait FxFn: FnMut(FxContext) -> f32 + 'static {}

pub type FxFnBoxFn = fn() -> Box<dyn FxFn>;

#[derive(Debug, Clone, PartialEq)]
pub struct FxSection {
    /// Beginning time of the section.
    pub time: f32,
    /// Time length of the section.
    pub length: f32,
    /// Row of the section.
    pub row: usize,
    /// Fx definition name of the section.
    pub def: String,
    // Params of the section.
    pub params: FxParams,
}

pub struct FxContext<'x> {
    pub index: usize,
    pub i0: usize,
    pub i1: usize,
    pub time: f32,
    pub t0: f32,
    pub t1: f32,
    pub delta_time: f32,
    pub value: f32,
    pub progress: f32,
    pub elapsed: f32,
    pub resolution: usize,
    pub length: f32,
    pub array: &'x [f32],
    //pub shouldNotInterpolate,
    //pub setShouldNotInterpolate,
    pub get_value: &'x dyn Fn(f32) -> f32,
    pub init: bool,
    //pub state: FxParams,
}

impl Debug for FxContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FxContext")
        .field("index", &self.index)
        .field("i0", &self.i0)
        .field("i1", &self.i1)
        .field("time", &self.time)
        .field("t0", &self.t0)
        .field("t1", &self.t1)
        .field("delta_time", &self.delta_time)
        .field("value", &self.value)
        .field("progress", &self.progress)
        .field("elapsed", &self.elapsed)
        .field("resolution", &self.resolution)
        .field("length", &self.length)
        .field("array", &self.array)
        .field("init", &self.init)
        //.field("state", &self.state)
        .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FxParams(HashMap<String, Value>);

impl FxParams {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.0.get(name).and_then(Value::as_f64)
    }

    pub fn get_i64(&self, name: &str) -> Option<i64> {
        self.0.get(name).and_then(Value::as_i64)
    }

    pub fn get_u64(&self, name: &str) -> Option<u64> {
        self.0.get(name).and_then(Value::as_u64)
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.0.get(name).and_then(Value::as_bool)
    }
}
