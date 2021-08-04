#![deny(missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

mod bezier;

pub mod channel;
pub mod connection;
pub mod curve;
pub mod fx;
pub mod item;
pub mod state;

use connection::Connection;
use fx::FxFn;
use state::SaveState;
use std::{collections::HashMap, fmt::Debug, io::Read, sync::Arc};

#[derive(Default)]
pub struct Automaton {
    time: f32,
    state: Option<Arc<SaveState>>,
    connection: Option<Connection>,
    fxs: HashMap<String, Box<dyn FxFn>>,
}

impl Debug for Automaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Automaton")
            .field("time", &self.time)
            .field("state", &self.state)
            .field("connection", &self.connection)
            .finish_non_exhaustive()
    }
}

impl Automaton {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            state: None,
            connection: None,
            fxs: HashMap::new(),
        }
    }

    pub fn load<R: Read>(&mut self, data: R) {
        let json = serde_json::from_reader(data).unwrap();
        let state = SaveState::from_json(json);
        self.state = Some(Arc::new(state));
    }

    pub fn add_fx_definition(&mut self, name: String, fx: impl FxFn) {
        self.fxs.insert(name, Box::new(fx));
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
