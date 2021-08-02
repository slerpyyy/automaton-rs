use crate::curve::Node;
use std::cell::RefCell;

const NEWTON_ITER: u32 = 4;
const NEWTON_EPSILON: f32 = 0.001;

const SUBDIV_ITER: u32 = 10;
const SUBDIV_EPSILON: f32 = 0.000001;

const TABLE_SIZE: usize = 21;

thread_local! {
    static CACHE: RefCell<[f32; TABLE_SIZE]> = RefCell::new([0.0; TABLE_SIZE]);
}

#[inline]
fn a(cps: [f32; 4]) -> f32 {
    cps[3] - 3.0 * cps[2] + 3.0 * cps[1] - cps[0]
}

#[inline]
fn b(cps: [f32; 4]) -> f32 {
    3.0 * cps[2] - 6.0 * cps[1] + 3.0 * cps[0]
}

#[inline]
fn c(cps: [f32; 4]) -> f32 {
    3.0 * cps[1] - 3.0 * cps[0]
}

#[inline]
fn cubic_bezier(t: f32, cps: [f32; 4]) -> f32 {
    ((a(cps) * t + b(cps)) * t + c(cps)) * t + cps[0]
}

#[inline]
fn delta_cubic_bezier(t: f32, cps: [f32; 4]) -> f32 {
    (3.0 * a(cps) * t + 2.0 * b(cps)) * t + c(cps)
}

// binary search approximation
#[inline]
fn subdiv(x: f32, mut a: f32, mut b: f32, cps: [f32; 4]) -> f32 {
    let mut candidate_x;
    let mut t = 0.0;

    for _ in 0..SUBDIV_ITER {
        t = a + (b - a) / 2.0;
        candidate_x = cubic_bezier(t, cps) - x;
        if 0.0 < candidate_x {
            b = t
        } else {
            a = t
        }

        if SUBDIV_EPSILON < candidate_x.abs() {
            break;
        }
    }

    t
}

// newton raphson approximation
#[inline]
fn newton(x: f32, mut t: f32, cps: [f32; 4]) -> f32 {
    for _ in 0..NEWTON_ITER {
        let d = delta_cubic_bezier(t, cps);
        if d == 0.0 {
            return t;
        }

        let cx = cubic_bezier(t, cps) - x;
        t = t - cx / d;
    }

    t
}

#[inline]
pub fn bezier_easing(node0: &Node, node1: &Node, time: f32) -> f32 {
    let mut cpsx = [
        node0.time,
        node0.time + node0.out_time,
        node1.time + node1.in_time,
        node1.time,
    ];

    let cpsy = [
        node0.value,
        node0.value + node0.out_value,
        node1.value + node1.in_value,
        node1.value,
    ];

    if time <= cpsx[0] {
        return cpsy[0];
    }

    if time >= cpsx[3] {
        return cpsy[3];
    }

    cpsx[1] = cpsx[1].clamp(cpsx[0], cpsx[3]);
    cpsx[2] = cpsx[2].clamp(cpsx[0], cpsx[3]);

    let (sample, dist) = CACHE.with(|cache_ref| {
        let mut cache = cache_ref.borrow_mut();

        for i in 0..TABLE_SIZE {
            let t = i as f32 / (TABLE_SIZE as f32 - 1.0);
            cache[i] = cubic_bezier(t, cpsx)
        }

        let sample = cache
            .iter()
            .skip(1)
            .position(|&c| time < c)
            .unwrap_or(TABLE_SIZE - 2);

        let dist = (time - cache[sample]) / (cache[sample + 1] - cache[sample]);
        (sample, dist)
    });

    let mut t = (sample as f32 + dist) / (TABLE_SIZE as f32 - 1.0);
    let d = delta_cubic_bezier(t, cpsx) / (cpsx[3] - cpsx[0]);

    if NEWTON_EPSILON <= d {
        t = newton(time, t, cpsx);
    } else if d != 0.0 {
        t = subdiv(
            time,
            // TODO: This might be off by one
            (sample as f32) / (TABLE_SIZE as f32 - 1.0),
            (sample as f32 + 1.0) / (TABLE_SIZE as f32 - 1.0),
            cpsx,
        );
    }

    cubic_bezier(t, cpsy)
}

#[cfg(test)]
mod test {
    use float_cmp::assert_approx_eq;

    use super::*;

    #[test]
    fn straight_line() {
        let n0 = &Node::new(2.0, 4.0);
        let n1 = &Node::new(6.0, 2.0);

        assert_approx_eq!(f32, bezier_easing(n0, n1, 2.0), 4.0);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 3.0), 3.5);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 4.0), 3.0);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 5.0), 2.5);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 6.0), 2.0);
    }

    #[test]
    fn bendy_line() {
        let n0 = &Node::with_out(2.0, 4.0, 1.0, 1.35);
        let n1 = &Node::with_in(6.0, 2.0, -1.0, -1.35);

        assert_approx_eq!(f32, bezier_easing(n0, n1, 2.0), 4.0);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 3.0), 4.0, epsilon = 0.01);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 4.0), 3.0);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 5.0), 2.0, epsilon = 0.01);
        assert_approx_eq!(f32, bezier_easing(n0, n1, 6.0), 2.0);
    }
}
