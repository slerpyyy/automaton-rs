use serde_json::Value;
use std::sync::Arc;

use crate::{curve::Curve, item::Item};

#[derive(Debug)]
pub struct Channel {
    items: Vec<Item>,
    value: f32,
    time: f32,
    head: usize,
}

impl Channel {
    pub(crate) fn from_json(json: &Value, curves: &[Arc<Curve>]) -> Self {
        let items = json
            .as_array()
            .into_iter()
            .flatten()
            .map(|item_json| Item::from_json(item_json, curves))
            .collect();

        Self {
            items,
            value: 0.0,
            time: f32::NEG_INFINITY,
            head: 0,
        }
    }

    pub fn current_value(&self) -> f32 {
        self.value
    }

    pub fn current_time(&self) -> f32 {
        self.time
    }

    pub fn reset(&mut self) {
        self.time = f32::NEG_INFINITY;
        self.value = 0.0;
        self.head = 0;
    }

    // TODO: figure out what a listener is
    //pub fn subscribe(&mut self, listener: ??) {}

    pub fn get_value(&self, time: f32) -> f32 {
        let index = self
            .items
            .iter()
            .position(|item| time < item.time)
            .unwrap_or(self.items.len() - 1);

        if index == 0 {
            return 0.0;
        }

        let item = &self.items[index];
        let t = (time - item.time).min(item.length);
        item.get_value(t)
    }

    // TODO: consume
}
