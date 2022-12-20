use std::ops::DerefMut;
use std::sync::Mutex;

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use lazy_static::*;
use slice_deque::SliceDeque;

lazy_static! {
    static ref FRAME_TIMES_VEC: Mutex<FrameTimesVec> = Mutex::new(FrameTimesVec {
        deltas: Vec::new(),
        fps: Vec::new()
    });
    static ref FRAME_TIMES_SLICE_DEQUE: Mutex<FrameTimesSliceDeque> =
        Mutex::new(FrameTimesSliceDeque {
            deltas: SliceDeque::new(),
            fps: SliceDeque::new()
        });
}
// For NUM=120, Vec wins (22ns vs 1us)
// For Num=12000, SliceDeque just wins (1us vs 1.4us)
static NUM_FRAME_TIMES_TO_TRACK: usize = 3600usize;

#[derive(Debug, Clone)]
struct FrameTimesVec {
    deltas: Vec<f32>,
    fps: Vec<f32>,
}

#[derive(Debug, Clone)]
struct FrameTimesSliceDeque {
    deltas: SliceDeque<f32>,
    fps: SliceDeque<f32>,
}

fn bench_vec(delta: f32) {
    let mut guard_frame_times = match FRAME_TIMES_VEC.lock() {
        Err(poisoned) => poisoned.into_inner(),
        Ok(guard) => guard,
    };
    let frame_times: &mut FrameTimesVec = guard_frame_times.deref_mut();
    let d: &mut Vec<f32> = &mut frame_times.deltas;
    let f: &mut Vec<f32> = &mut frame_times.fps;

    // Insert into first index, pushing everything else back
    d.insert(0, delta);
    f.insert(0, 1f32 / delta);
    // Cut off the end to fit our item count
    d.truncate(NUM_FRAME_TIMES_TO_TRACK);
    f.truncate(NUM_FRAME_TIMES_TO_TRACK);

    plot_fake(d);
    plot_fake(f);
}

fn plot_fake(dummy: &[f32]) {
    black_box(dummy);
}

fn bench_slice_deque(delta: f32) {
    let mut guard_frame_times = match FRAME_TIMES_SLICE_DEQUE.lock() {
        Err(poisoned) => poisoned.into_inner(),
        Ok(guard) => guard,
    };
    let frame_times: &mut FrameTimesSliceDeque = guard_frame_times.deref_mut();
    let d: &mut SliceDeque<f32> = &mut frame_times.deltas;
    let f: &mut SliceDeque<f32> = &mut frame_times.fps;

    // Insert into first index, pushing everything else back
    d.push_front(delta);
    f.push_front(1f32 / delta);
    d.truncate(NUM_FRAME_TIMES_TO_TRACK);
    f.truncate(NUM_FRAME_TIMES_TO_TRACK);

    plot_fake(d);
    plot_fake(f);
}

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("vec", |b| b.iter(|| bench_vec(black_box(1f32 / 69f32))));
    criterion.bench_function("slice_deque", |b| {
        b.iter(|| bench_slice_deque(black_box(1f32 / 69f32)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
