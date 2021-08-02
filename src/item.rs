use serde_json::Value;
use std::sync::Arc;

use crate::curve::Curve;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub time: f32,
    pub length: f32,
    pub value: f32,
    pub offset: f32,
    pub speed: f32,
    pub amp: f32,
    pub reset: bool,
    pub curve: Option<Arc<Curve>>,
}

impl Item {
    pub(crate) fn from_json(json: &Value, curves: &[Arc<Curve>]) -> Self {
        Self {
            time: json.get("time").and_then(Value::as_f64).unwrap_or(0.0) as _,
            length: json.get("length").and_then(Value::as_f64).unwrap_or(0.0) as _,
            value: json.get("value").and_then(Value::as_f64).unwrap_or(0.0) as _,
            offset: json.get("offset").and_then(Value::as_f64).unwrap_or(0.0) as _,
            speed: json.get("speed").and_then(Value::as_f64).unwrap_or(1.0) as _,
            amp: json.get("amp").and_then(Value::as_f64).unwrap_or(1.0) as _,
            reset: json.get("reset").and_then(Value::as_bool).unwrap_or(false) as _,
            curve: json
                .get("curve")
                .and_then(Value::as_u64)
                .and_then(|index| curves.get(index as usize))
                .cloned(),
        }
    }

    pub fn get_value(&self, time: f32) -> f32 {
        if self.reset && self.length <= time {
            return 0.0;
        }

        if let Some(curve) = &self.curve {
            let t = self.offset + time * self.speed;
            return self.value + self.amp * curve.get_value(t);
        }

        self.value
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::curve::Node;

    #[test]
    fn item_parse_constant() {
        let json = r#"{
            "time": 7.5,
            "length": 1.2,
            "value": 0.4,
            "reset": true
        }"#;

        let value = serde_json::from_str(json).unwrap();
        let actual = Item::from_json(&value, &[]);

        let expected = Item {
            time: 7.5,
            length: 1.2,
            value: 0.4,
            offset: 0.0,
            speed: 1.0,
            amp: 1.0,
            reset: true,
            curve: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn item_parse_curve() {
        let json = r#"{
            "time": 3.1,
            "length": 2.0,
            "curve": 0
        }"#;

        let nodes = &[Node::new(1.0, 2.0), Node::new(3.0, 4.0)];
        let curve = Arc::new(Curve::new(nodes));

        let value = serde_json::from_str(json).unwrap();
        let actual = Item::from_json(&value, &[curve.clone()]);

        let expected = Item {
            time: 3.1,
            length: 2.0,
            value: 0.0,
            offset: 0.0,
            speed: 1.0,
            amp: 1.0,
            reset: false,
            curve: Some(curve),
        };

        assert_eq!(actual, expected);
    }
}
