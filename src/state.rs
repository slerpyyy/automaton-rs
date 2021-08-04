use crate::channel::Channel;
use crate::curve::Curve;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug)]
pub struct SaveState {
    time: f32,
    resolution: usize,
    curves: Vec<Arc<Curve>>,
    channels: Vec<Channel>,
    labels: Vec<Label>,
}

impl SaveState {
    pub fn from_json(json: Value) -> Self {
        let resolution = json
            .get("resolution")
            .and_then(Value::as_u64)
            .unwrap_or(100) as _;

        let curves = json
            .get("curves")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|v| Arc::new(Curve::from_json(v, resolution)))
            .collect::<Vec<_>>();

        let channels = json
            .get("channels")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|v| Channel::from_json(v, &curves))
            .collect::<Vec<_>>();

        let labels = json
            .get("labels")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(name, value)| {
                value
                    .as_f64()
                    .map(|time| Label::new(name.clone(), time as _))
            })
            .collect();

        Self {
            time: 0.0,
            resolution,
            curves,
            channels,
            labels,
        }
    }
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
    pub time: f32,
}

impl Label {
    pub fn new(name: String, time: f32) -> Self {
        Self { name, time }
    }
}
