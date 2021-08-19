use std::collections::HashMap;

use serde_json::Value;

use crate::{bezier::bezier_easing, fx::{FxContext, FxFnBoxFn, FxSection}};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Node {
    pub time: f32,
    pub value: f32,
    pub in_time: f32,
    pub in_value: f32,
    pub out_time: f32,
    pub out_value: f32,
}

impl Node {
    pub(crate) fn from_json(json: &Value) -> Self {
        let mut iter = json
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(Value::as_f64);

        Self {
            time: iter.next().unwrap_or(0.0) as _,
            value: iter.next().unwrap_or(0.0) as _,
            in_time: iter.next().unwrap_or(0.0) as _,
            in_value: iter.next().unwrap_or(0.0) as _,
            out_time: iter.next().unwrap_or(0.0) as _,
            out_value: iter.next().unwrap_or(0.0) as _,
        }
    }

    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            ..Default::default()
        }
    }

    pub fn with_in(time: f32, value: f32, in_time: f32, in_value: f32) -> Self {
        Self {
            time,
            value,
            in_time,
            in_value,
            ..Default::default()
        }
    }

    pub fn with_out(time: f32, value: f32, out_time: f32, out_value: f32) -> Self {
        Self {
            time,
            value,
            out_time,
            out_value,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Curve {
    pub nodes: Vec<Node>,
    pub fxs: Vec<FxSection>,
    values: Vec<f32>,
}

impl Curve {
    pub(crate) fn from_json(json: &Value, resolution: usize) -> Self {
        let nodes = json
            .get("nodes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(Node::from_json)
            .collect::<Vec<_>>();

        Self::with_params(&nodes, resolution, &HashMap::new())
    }

    pub fn new(nodes: &[Node]) -> Self {
        Self::with_params(nodes, 100, &HashMap::new())
    }

    pub fn with_params(nodes: &[Node], resolution: usize, fxs: &HashMap<String, FxFnBoxFn>) -> Self {
        if nodes.len() < 2 {
            panic!(
                "A curve must consist of at least 2 nodes, got {}",
                nodes.len()
            );
        }

        let mut this = Self {
            nodes: nodes.to_vec(),
            values: Vec::new(),
            fxs: Vec::new(),
        };

        this.precalc(resolution, fxs);
        this
    }

    fn precalc(&mut self, resolution: usize, fxs: &HashMap<String, FxFnBoxFn>) {
        self.generate_curve(resolution);
        self.apply_fxs(resolution, fxs);
    }

    fn generate_curve(&mut self, resolution: usize) {
        self.nodes.sort_unstable_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let values_length = (resolution as f32 * self.length()).ceil() as usize + 1;
        self.values = Vec::with_capacity(values_length);
        unsafe { self.values.set_len(values_length) };

        let mut node_tail = self.nodes.first().unwrap();
        let mut i_tail = 0;
        for i_node in 0..(self.nodes.len() - 1) {
            let node0 = node_tail;
            node_tail = &self.nodes[i_node + 1];

            let i0 = i_tail;
            i_tail = (node_tail.time * resolution as f32).floor() as _;

            self.values[i0] = node0.value;

            for i in (i0 + 1)..=i_tail {
                let time = i as f32 / resolution as f32;
                let value = bezier_easing(node0, node_tail, time);
                self.values[i] = value;
            }
        }

        for i in (i_tail + 1)..self.values.len() {
            self.values[i] = node_tail.value;
        }
    }

    fn apply_fxs(&mut self, resolution: usize, fxs: &HashMap<String, FxFnBoxFn>) {
        for fx in &self.fxs {
            let fx_def = fxs.get(&fx.def);
            let mut fx_fn = match fx_def {
                Some(fx_def) => fx_def(),
                _ => {
                    eprintln!("No such fx definition: {}", fx.def);
                    continue;
                }
            };

            let available_end = f32::min(self.length(), fx.time + fx.length);
            let i0 = f32::ceil(resolution as f32 * fx.time) as usize;
            let i1 = f32::floor(resolution as f32 * available_end) as usize;
            if i1 <= i0 {
                eprintln!("Length of the fx section is being negative");
                continue;
            }

            let temp_length = i1 - i0 + 1;
            for i in 0..temp_length {
                let index = i + i0;
                let time = index as f32 / resolution as f32;
                let elapsed = time - fx.time;
                let progress = elapsed / fx.length;

                let context = FxContext {
                    index,
                    i0,
                    i1,
                    time,
                    t0: fx.time,
                    t1: fx.time + fx.length,
                    delta_time: 1.0 / resolution as f32,
                    value: self.values[i + i0],
                    progress,
                    elapsed,
                    resolution,
                    length: fx.length,
                    //params: fx.params,
                    array: &self.values,
                    //shouldNotInterpolate: this.__shouldNotInterpolate[ i0 ] === 1,
                    //setShouldNotInterpolate: ( shouldNotInterpolate: boolean ) => {
                    //  this.__shouldNotInterpolate[ context.index ] = shouldNotInterpolate ? 1 : 0;
                    //},
                    get_value: &|t: f32| self.get_value(t),
                    init: i == 0,
                    //state: FxParams::new(),
                };

                //context.shouldNotInterpolate = this.__shouldNotInterpolate[ i + i0 ] == 1;

                self.values[i] = fx_fn(context);
            }
        }
    }

    pub fn get_value(&self, time: f32) -> f32 {
        if time < 0.0 {
            return *self.values.first().unwrap();
        }

        let length = self.length();
        if time >= length {
            return *self.values.last().unwrap();
        }

        let last = self.values.len() - 2;
        let index = last as f32 * time / length;
        let index_i = index.floor() as usize;
        let index_f = index.fract();

        let v0 = self.values[index_i];
        let v1 = self.values[index_i + 1];
        v0 + (v1 - v0) * index_f
    }

    pub fn length(&self) -> f32 {
        self.nodes.last().map(|n| n.time).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn node_parse_full() {
        let json = "[5.2, 0, -3, 1.0, -2.9, 2]";
        let value = serde_json::from_str(json).unwrap();
        let node = Node::from_json(&value);

        assert_eq!(node.time, 5.2);
        assert_eq!(node.value, 0.0);
        assert_eq!(node.in_time, -3.0);
        assert_eq!(node.in_value, 1.0);
        assert_eq!(node.out_time, -2.9);
        assert_eq!(node.out_value, 2.0);
    }

    #[test]
    fn node_parse_empty() {
        let json = "[]";
        let value = serde_json::from_str(json).unwrap();
        let node = Node::from_json(&value);

        assert_eq!(node.time, 0.0);
        assert_eq!(node.value, 0.0);
        assert_eq!(node.in_time, 0.0);
        assert_eq!(node.in_value, 0.0);
        assert_eq!(node.out_time, 0.0);
        assert_eq!(node.out_value, 0.0);
    }

    #[test]
    fn curve_parse_simple() {
        let json = r#"{
            "nodes": [
                [0,1,2,3,4,5],
                [1.2,5,-0.4,-7.3]
            ]
        }"#;

        let value = serde_json::from_str(json).unwrap();
        let curve = Curve::from_json(&value, 100);

        assert_eq!(curve.nodes[0].time, 0.0);
        assert_eq!(curve.nodes[0].value, 1.0);
        assert_eq!(curve.nodes[0].in_time, 2.0);
        assert_eq!(curve.nodes[0].in_value, 3.0);
        assert_eq!(curve.nodes[0].out_time, 4.0);
        assert_eq!(curve.nodes[0].out_value, 5.0);

        assert_eq!(curve.nodes[1].time, 1.2);
        assert_eq!(curve.nodes[1].value, 5.0);
        assert_eq!(curve.nodes[1].in_time, -0.4);
        assert_eq!(curve.nodes[1].in_value, -7.3);
        assert_eq!(curve.nodes[1].out_time, 0.0);
        assert_eq!(curve.nodes[1].out_value, 0.0);

        assert_eq!(curve.nodes.len(), 2);
    }

    #[test]
    fn curve_line() {
        let n0 = Node::new(0.0, 0.0);
        let n1 = Node::new(2.0, 1.0);
        let curve = Curve::new(&[n0, n1]);

        assert_approx_eq!(f32, curve.get_value(-1.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(0.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(1.0), 0.5, epsilon = 0.005);
        assert_approx_eq!(f32, curve.get_value(2.0), 1.0);
        assert_approx_eq!(f32, curve.get_value(3.0), 1.0);
    }

    #[test]
    fn curve_arc() {
        let n0 = Node::with_out(0.0, 0.0, 1.0, 2.0);
        let n1 = Node::with_in(2.0, 0.0, -1.0, 2.0);
        let curve = Curve::new(&[n0, n1]);

        assert_approx_eq!(f32, curve.get_value(-1.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(0.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(1.0), 1.5, epsilon = 0.005);
        assert_approx_eq!(f32, curve.get_value(2.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(3.0), 0.0);
    }
}
