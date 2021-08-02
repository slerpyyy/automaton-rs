use serde_json::Value;

use crate::bezier::bezier_easing;

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

        Self::with_resolution(&nodes, resolution)
    }

    pub fn new(nodes: &[Node]) -> Self {
        Self::with_resolution(nodes, 100)
    }

    pub fn with_resolution(nodes: &[Node], resolution: usize) -> Self {
        if nodes.len() < 2 {
            panic!(
                "A curve must consist of at least 2 nodes, got {}",
                nodes.len()
            );
        }

        let mut this = Self {
            nodes: nodes.to_vec(),
            values: Vec::new(),
        };

        this.precalc(resolution);
        this
    }

    fn precalc(&mut self, resolution: usize) {
        self.generate_curve(resolution);
        self.apply_fxs(resolution);
    }

    fn generate_curve(&mut self, resolution: usize) {
        self.nodes.sort_unstable_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.values.clear();
        let total = (self.length() * resolution as f32).floor() as usize;
        self.values.reserve(total);

        let mut node_iter = self.nodes.iter();
        let mut last = node_iter.next().unwrap();

        for curr in node_iter {
            let steps = ((curr.time - last.time) * resolution as f32).floor() as usize;

            for _ in 0..steps {
                let time = self.values.len() as f32 / resolution as f32;
                let value = bezier_easing(last, curr, time);
                self.values.push(value);
            }

            last = curr;
        }

        self.values
            .extend(std::iter::repeat(last.value).take(total.saturating_sub(self.values.len())));
    }

    fn apply_fxs(&mut self, _resolution: usize) {
        // TODO: figure out what the hell this is
    }

    pub fn get_value(&self, time: f32) -> f32 {
        if time < 0.0 {
            return *self.values.first().unwrap();
        }

        let length = self.length();
        if time > length {
            return *self.values.last().unwrap();
        }

        let last = self.values.len() - 2;
        let index = last as f32 * time / length;
        let index_i = index.floor() as usize;
        let index_f = index.fract();

        let low = self.values[index_i];
        let high = self.values[index_i + 1];
        low + (high - low) * index_f
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
        assert_approx_eq!(f32, curve.get_value(1.0), 0.5);
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
        assert_approx_eq!(f32, curve.get_value(1.0), 1.5);
        assert_approx_eq!(f32, curve.get_value(2.0), 0.0);
        assert_approx_eq!(f32, curve.get_value(3.0), 0.0);
    }
}
