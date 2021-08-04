pub trait FxFn: FnMut(FxContext) -> f32 + 'static {}

#[derive(Debug)]
pub struct FxContext<'x> {
    index: usize,
    i0: usize,
    i1: usize,
    time: f32,
    t0: f32,
    t1: f32,
    delta_time: f32,
    value: f32,
    progress: f32,
    elapsed: f32,
    resolution: usize,
    length: usize,
    array: &'x [f32],
    //shouldNotInterpolate,
    //setShouldNotInterpolate,
    //get_value,
    init: bool,
    //state,
}
