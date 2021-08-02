mod bezier;

pub mod channel;
pub mod curve;
pub mod item;

use channel::Channel;
use curve::Curve;
use serde_json::Value;
use std::sync::Arc;

pub struct Automaton {
    time: f32,
    resolution: usize,
    curves: Vec<Arc<Curve>>,
    channels: Vec<Channel>,
    //labels: Vec<Label>,
}

impl Automaton {
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

        Self {
            time: 0.0,
            resolution,
            curves,
            channels,
        }
    }
}

pub struct Label {
    name: String,
    time: f32,
}

#[cfg(test)]
mod tests {

    #[cfg(disabled)]
    #[test]
    fn websocket_test() {
        let (mut ws, _) = tungstenite::connect("ws://localhost:12250/").unwrap();

        loop {
            let msg = ws.read_message().unwrap();
            println!("{}", msg);
        }
    }
}
