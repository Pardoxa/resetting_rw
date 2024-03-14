use std::{
    collections::BinaryHeap, f64::consts::SQRT_2, io::Write, num::NonZeroUsize, sync::Mutex
};
use camino::Utf8PathBuf;
use indicatif::{ProgressIterator, ProgressStyle};
use kahan::KahanSum;
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, StandardNormal};
use rand_pcg::{Pcg32, Pcg64, Pcg64Mcg};
use serde::{Deserialize, Serialize};
use derivative::Derivative;
use rayon::prelude::*;

use crate::{
    misc::{create_buf_with_command_and_version, write_slice_head},
    sync_queue::SyncQueue
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Bisect{
    Steps(NonZeroUsize),
    Threshold(f64)
}

impl Default for Bisect{
    fn default() -> Self {
        Self::Threshold(1e-6)
    }
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
pub struct MeasureMfptOpt
{
    settimgs: RadomWalkSettings,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    samples_per_point: NonZeroUsize,
    #[derivative(Default(value="0.1"))]
    lambda_left: f64,
    #[derivative(Default(value="5.0"))]
    lambda_right: f64,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    lambda_samples: NonZeroUsize,
    /// Number of threads. 
    #[derivative(Default(value="NonZeroUsize::new(1).unwrap()"))]
    j: NonZeroUsize,
    seed: u64,
    bisection: Bisect
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
pub struct MeasureMfptLOpt
{
    settimgs: RadomWalkSettings,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    samples_per_point: NonZeroUsize,
    #[derivative(Default(value="0.1"))]
    target_left: f64,
    #[derivative(Default(value="5.0"))]
    target_right: f64,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    target_samples: NonZeroUsize,
    /// Number of threads. 
    #[derivative(Default(value="NonZeroUsize::new(1).unwrap()"))]
    j: NonZeroUsize,
    seed: u64,
    bisection: Bisect
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
pub struct MeasureMfptBetaOpt
{
    pub settimgs: RadomWalkSettings,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    samples_per_point: NonZeroUsize,
    #[derivative(Default(value="0.1"))]
    beta_left: f64,
    #[derivative(Default(value="5.0"))]
    beta_right: f64,
    #[derivative(Default(value="NonZeroUsize::new(100).unwrap()"))]
    beta_samples: NonZeroUsize,
    /// Number of threads. 
    #[derivative(Default(value="NonZeroUsize::new(1).unwrap()"))]
    j: NonZeroUsize,
    seed: u64,
    bisection: Bisect
}

pub fn eff_measure_mfpt_beta(
    opt: MeasureMfptBetaOpt,
    file_name: Utf8PathBuf
)
{
    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.j.get())
        .build_global()
        .unwrap();

    let delta = (opt.beta_right - opt.beta_left) / (opt.beta_samples.get() - 1) as f64;

    let mut seeding_rng = Pcg32::seed_from_u64(opt.seed);
    let samples_per_packet = (opt.samples_per_point.get() / (opt.j.get() * 12)).max(1);

    let header = [
        "β",
        "mfpt"
    ];
    let mut buf = create_buf_with_command_and_version(file_name);
    write_slice_head(&mut buf, header).unwrap();
    let mut settings = opt.settimgs.clone();

    let style = ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise} - {eta_precise}] {wide_bar}")
        .unwrap();
    
    for i in (0..opt.beta_samples.get()).progress_with_style(style)
    {
        let beta = delta.mul_add(i as f64, opt.beta_left);
        let b2 = beta / settings.target;
        settings.lambda_mirror = b2 * b2;
        let queue = SyncQueue::create_work_queue(
            opt.samples_per_point.get(), 
            NonZeroUsize::new(opt.j.get() * 3).unwrap()
        );
        let queue = queue.map(
            |amount|
            {
                let rng = Pcg64::from_rng(&mut seeding_rng).unwrap();
                let walk = EffRandWalk::new(
                    settings.clone(), 
                    rng
                );
                (walk, amount)
            }
        );
        let global_sum_fpt = Mutex::new(KahanSum::new());
        (0..opt.j.get())
            .into_par_iter()
            .for_each(
                |_|
                {
                    let mut sum_fpt = KahanSum::new();
                    while let Some((mut walker, amount)) = queue.pop() {
                        let work = amount.min(samples_per_packet);
                        let left = amount - work;

                        for _ in 0..work{
                            walker.bisect(opt.bisection);
                            let (i,j) = walker.delta_fpt;
                            let delta = &walker.walk[i][j];
                            let fpt = delta.interpolate(walker.settings.target);
                            sum_fpt += fpt;
                            walker.recycle();
                        }
                        
                        if left > 0{
                            queue.push(
                                (walker, left)
                            );
                        }
                    }
                    let mut lock = global_sum_fpt
                        .lock()
                        .unwrap();
                    *lock += sum_fpt;
                    drop(lock);
                }
            );
        let mfpt = global_sum_fpt.into_inner().unwrap().sum() / opt.samples_per_point.get() as f64;
        writeln!(
            buf,
            "{beta} {mfpt}"
        ).unwrap();
    }
    
    
}

pub fn eff_measure_mfpt_lambda(
    opt: MeasureMfptOpt,
    file_name: Utf8PathBuf
)
{
    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.j.get())
        .build_global()
        .unwrap();

    let delta = (opt.lambda_right - opt.lambda_left) / (opt.lambda_samples.get() - 1) as f64;

    let mut seeding_rng = Pcg32::seed_from_u64(opt.seed);
    let samples_per_packet = (opt.samples_per_point.get() / (opt.j.get() * 12)).max(1);

    let header = [
        "lambda",
        "mfpt"
    ];
    let mut buf = create_buf_with_command_and_version(file_name);
    write_slice_head(&mut buf, header).unwrap();
    let mut settings = opt.settimgs.clone();

    let style = ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise} - {eta_precise}] {wide_bar}")
        .unwrap();
    
    for i in (0..opt.lambda_samples.get()).progress_with_style(style)
    {
        let lambda = delta.mul_add(i as f64, opt.lambda_left);
        settings.lambda_mirror = lambda;
        let queue = SyncQueue::create_work_queue(
            opt.samples_per_point.get(), 
            NonZeroUsize::new(opt.j.get() * 3).unwrap()
        );
        let queue = queue.map(
            |amount|
            {
                let rng = Pcg64::from_rng(&mut seeding_rng).unwrap();
                let walk = EffRandWalk::new(
                    settings.clone(), 
                    rng
                );
                (walk, amount)
            }
        );
        let global_sum_fpt = Mutex::new(KahanSum::new());
        (0..opt.j.get())
            .into_par_iter()
            .for_each(
                |_|
                {
                    let mut sum_fpt = KahanSum::new();
                    while let Some((mut walker, amount)) = queue.pop() {
                        let work = amount.min(samples_per_packet);
                        let left = amount - work;

                        for _ in 0..work{
                            walker.bisect(opt.bisection);
                            let (i,j) = walker.delta_fpt;
                            let delta = &walker.walk[i][j];
                            let fpt = delta.interpolate(walker.settings.target);
                            sum_fpt += fpt;
                            walker.recycle();
                        }
                        
                        if left > 0{
                            queue.push(
                                (walker, left)
                            );
                        }
                    }
                    let mut lock = global_sum_fpt
                        .lock()
                        .unwrap();
                    *lock += sum_fpt;
                    drop(lock);
                }
            );
        let mfpt = global_sum_fpt.into_inner().unwrap().sum() / opt.samples_per_point.get() as f64;
        writeln!(
            buf,
            "{lambda} {mfpt}"
        ).unwrap();
    }
    
    
}

pub fn eff_measure_mfpt_target(
    opt: MeasureMfptLOpt,
    file_name: Utf8PathBuf
)
{
    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.j.get())
        .build_global()
        .unwrap();

    let delta = (opt.target_right - opt.target_left) / (opt.target_samples.get() - 1) as f64;

    let mut seeding_rng = Pcg32::seed_from_u64(opt.seed);
    let samples_per_packet = (opt.samples_per_point.get() / (opt.j.get() * 12)).max(1);

    let header = [
        "L",
        "mfpt"
    ];
    let mut buf = create_buf_with_command_and_version(file_name);
    write_slice_head(&mut buf, header).unwrap();
    let mut settings = opt.settimgs.clone();

    let style = ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise} - {eta_precise}] {wide_bar}")
        .unwrap();
    
    for i in (0..opt.target_samples.get()).progress_with_style(style)
    {
        let target = delta.mul_add(i as f64, opt.target_left);
        settings.target = target;
        let queue = SyncQueue::create_work_queue(
            opt.samples_per_point.get(), 
            NonZeroUsize::new(opt.j.get() * 3).unwrap()
        );
        let queue = queue.map(
            |amount|
            {
                let rng = Pcg64::from_rng(&mut seeding_rng).unwrap();
                let walk = EffRandWalk::new(
                    settings.clone(), 
                    rng
                );
                (walk, amount)
            }
        );
        let global_sum_fpt = Mutex::new(KahanSum::new());
        (0..opt.j.get())
            .into_par_iter()
            .for_each(
                |_|
                {
                    let mut sum_fpt = KahanSum::new();
                    while let Some((mut walker, amount)) = queue.pop() {
                        let work = amount.min(samples_per_packet);
                        let left = amount - work;

                        for _ in 0..work{
                            walker.bisect(opt.bisection);
                            let (i,j) = walker.delta_fpt;
                            let delta = &walker.walk[i][j];
                            let fpt = delta.interpolate(walker.settings.target);
                            sum_fpt += fpt;
                            walker.recycle();
                        }
                        
                        if left > 0{
                            queue.push(
                                (walker, left)
                            );
                        }
                    }
                    let mut lock = global_sum_fpt
                        .lock()
                        .unwrap();
                    *lock += sum_fpt;
                    drop(lock);
                }
            );
        let mfpt = global_sum_fpt.into_inner().unwrap().sum() / opt.samples_per_point.get() as f64;
        writeln!(
            buf,
            "{target} {mfpt}"
        ).unwrap();
    }
    
    
}


pub fn test_eff_rand_walker()
{
    let settings = RadomWalkSettings{
        target: 1.0,
        rough_step_size: 3e-5,
        max_depth: 10,
        a: 0.5,
        lambda_mirror: 0.1,
        origin: 0.0
    };
    let rng = Pcg64Mcg::seed_from_u64(0xff00abcf);
    let mut walker = EffRandWalk::new(
        settings,
        rng
    );
    for _ in 0..100000{
        walker.bisection_step();
    }
    for (idx, walk) in walker.walk.iter_mut().enumerate(){
        let name = format!("walker{idx}");
        let mut buf = create_buf_with_command_and_version(name);
        walk.sort_unstable_by_key(|v| OrderedFloat(v.left_time));
        for delta in walk.iter(){
            let time = delta.left_time;
            let pos = delta.left_pos;
            writeln!(buf, "{time} {pos}").unwrap();
        }
    }
}

#[derive(Debug)]
pub struct EffRandWalk<R>
{
    // Later I should check if HashMap is faster!
    walk: Vec<Vec<Delta>>,
    prob: BinaryHeap<NextProb>,
    fpt: f64,
    delta_fpt: (usize, usize),
    seeding_rng: R,
    rng: R,
    settings: RadomWalkSettings
}

#[derive(Debug)]
pub struct NextProb
{
    which_vec: usize,
    index: usize,
    prob: OrderedFloat<f64>,
}

impl Eq for NextProb {}

impl PartialEq for NextProb {
    fn eq(&self, other: &Self) -> bool {
        self.prob == other.prob && self.index == other.index
    }
}

impl PartialOrd for NextProb{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NextProb{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.prob.cmp(&other.prob)
    }
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
pub struct RadomWalkSettings{
    #[derivative(Default(value="0.1"))]
    lambda_mirror: f64,
    #[derivative(Default(value="1.0"))]
    rough_step_size: f64,
    #[derivative(Default(value="1.0"))]
    target: f64,
    pub a: f64,
    #[derivative(Default(value="40"))]
    max_depth: usize,
    #[derivative(Default(value="0.0"))]
    origin: f64
}

fn create_initial_walk<R>(
    settings: &RadomWalkSettings,
    mut rng: R,
    walk: &mut Vec<Delta>
) -> (f64, (usize, usize))
where R: Rng
{
    walk.clear();
    let mirror_dist = Exp::new(settings.lambda_mirror)
        .unwrap();
    let mut next_mirror_time = mirror_dist.sample(&mut rng);
    let sqrt_step_size = settings.rough_step_size.sqrt();
    let sq = sqrt_step_size * SQRT_2;
    let mut current_pos = settings.origin;
    let mut current_time = 0.0;
    let mut delta_fpt = (0,0);
    loop {
        let div = next_mirror_time / settings.rough_step_size;
        let floored = div.floor();
        let rest = div.fract() * settings.rough_step_size;
        let steps = floored as usize;
        for i in 0..steps{
            let left_time = settings.rough_step_size.mul_add(i as f64, current_time);
            let left_pos = current_pos;
            current_pos += rng.sample::<f64, _>(StandardNormal) * sq;
            let delta = Delta{
                left_pos,
                right_pos: current_pos,
                delta_t: settings.rough_step_size,
                left_time
            };
            let contained = delta.contains(&settings.target);
            walk.push(delta);
            if contained
            {
                // TODO in here I could linerarly interpolate to get a more accurate result
                let fpt = settings.rough_step_size.mul_add((i + 1) as f64, current_time);
                delta_fpt.1 = walk.len() - 1;
                return (fpt, delta_fpt);
            }
        }
        current_time = settings.rough_step_size.mul_add(steps as f64, current_time);
        let rest_sq = rest.sqrt() * SQRT_2;
        let left_time = current_time;
        current_time += rest;
        let left_pos = current_pos;
        current_pos += rng.sample::<f64, _>(StandardNormal) * rest_sq;
        let delta = Delta{
            left_pos,
            right_pos: current_pos,
            delta_t: rest,
            left_time
        };
        walk.push(delta);
        if (left_pos..=current_pos).contains(&settings.target)
        {
            // TODO in here I could linerarly interpolate to get a more accurate result
            let fpt = current_time;
            delta_fpt.1 = walk.len() - 1;
            return (fpt, delta_fpt);
        }
        current_pos *= settings.a;
        next_mirror_time = mirror_dist.sample(&mut rng);
    }
}

fn calc_heap(
    target: f64,
    walk: &[Delta],
    heap: &mut BinaryHeap<NextProb>
)
{
    heap.clear();
    heap.extend(
        walk
            .iter()
            .enumerate()
            .map(
                |(idx, val)|
                {
                    let prob = val.calc_prob(target);
                    NextProb{
                        which_vec: 0,
                        index: idx,
                        prob: OrderedFloat(prob)
                    }
                }
            )
    );
}


impl<R> EffRandWalk<R>
where R: Rng + SeedableRng
{
    pub fn new(
        settings: RadomWalkSettings,
        mut rng: R,
    ) -> Self
    {
        let mut initial_walk = Vec::with_capacity(1024*1024);
        let mut walker_rng = R::from_rng(&mut rng).unwrap();
        let (fpt, delta_fpt) = create_initial_walk(
            &settings, 
            &mut walker_rng,
            &mut initial_walk
        );
        let mut heap = BinaryHeap::new();
        calc_heap(settings.target, &initial_walk, &mut heap);
        let mut walk = vec![initial_walk];
        walk.extend(
            (1..settings.max_depth).map(|_| Vec::new())
        );
        Self {
            walk, 
            prob: heap, 
            fpt,
            seeding_rng: rng,
            settings,
            rng: walker_rng,
            delta_fpt
        }
    }

    
    pub fn recycle(&mut self)
    {
        self.walk[1..]
            .iter_mut()
            .for_each(|walk| walk.clear());
        self.rng = R::from_rng(&mut self.seeding_rng).unwrap();
        let (fpt, delta_fpt) = create_initial_walk(
            &self.settings, 
            &mut self.rng, 
            &mut self.walk[0]
        );
        self.fpt = fpt;
        self.delta_fpt = delta_fpt;
        calc_heap(
            self.settings.target, 
            &self.walk[0], 
            &mut self.prob
        );
    }

    fn bisect(&mut self, bisection: Bisect)
    {
        match bisection{
            Bisect::Steps(s) => {
                for _ in 0..s.get()
                {
                    self.bisection_step()
                }
            },
            Bisect::Threshold(th) => {
                while let Some(top) = self.prob.peek(){
                    if top.prob.into_inner() < th{
                        break;
                    }
                    self.bisection_step();
                }
            }
        }
    }

    fn bisection_step(&mut self)
    {
        let max_len = self.walk.len();
        while let Some(val) = self.prob.pop(){
            
            let item = &self.walk[val.which_vec][val.index];
            if item.left_time + item.delta_t > self.fpt {
                continue;
            }

            let next_vec_id = val.which_vec + 1;
            let (left, right) = item.bisect(&mut self.rng);
            let walk = &mut self.walk[next_vec_id];
            if left.contains(&self.settings.target)
            {
                self.fpt = left.left_time + left.delta_t;
                self.delta_fpt = (next_vec_id, walk.len());
            } else if right.contains(&self.settings.target) {
                self.fpt = right.left_time + right.delta_t;
                self.delta_fpt = (next_vec_id, walk.len() + 1);
            }

            
            if next_vec_id + 1 < max_len {
                let prob_left = left.calc_prob(self.settings.target);
                let prob_right = right.calc_prob(self.settings.target);
                let idx = walk.len();
                self.prob.push(
                    NextProb { which_vec: next_vec_id, index: idx, prob: OrderedFloat(prob_left) }
                );
                self.prob.push(
                    NextProb { which_vec: next_vec_id, index: idx + 1, prob: OrderedFloat(prob_right) }
                );
            }
            walk.push(left);
            walk.push(right);
            
            break;
        }
    }
}

#[derive(Debug)]
pub struct Delta{
    left_pos: f64,
    right_pos: f64,
    delta_t: f64,
    left_time: f64
}

impl Delta{

    pub fn interpolate(&self, target: f64) -> f64
    {
        let delta = self.right_pos - self.left_pos;
        let frac = (target - self.left_pos) / delta;
        self.left_time + frac * self.delta_t
    }

    /// $$
    ///     \exp[- (2L-x_1-x_2)^2/{4Dt}]/\exp[- (x_2-x_1)^2/{4Dt}/]= \exp[- (L-x_1)(L-x_2)/{Dt}]
    /// $$
    /// 
    #[inline]
    pub fn calc_prob(&self, target: f64) -> f64
    {
        let inner = -(target - self.left_pos) * (target - self.right_pos) / self.delta_t;
        inner.exp()
    }

    #[inline]
    pub fn contains(&self, target: &f64) -> bool
    {
        (self.left_pos..=self.right_pos).contains(target)
    }

    // Uses a brownian bridge
    #[inline]
    pub fn bisect<R: Rng>(
        &self, 
        rng: &mut R
    ) -> (Delta, Delta)
    {
        let diff = self.right_pos - self.left_pos;

        let delta_t = self.delta_t * 0.5;
        let sq = delta_t.sqrt() * SQRT_2;

        let mut mid = rng.sample::<f64, _>(StandardNormal) * sq;
        let end = mid + rng.sample::<f64,_>(StandardNormal) * sq;
        mid -= 0.5 * (end - diff);
        mid += self.left_pos;
        
        (
            Self{
                left_pos: self.left_pos,
                right_pos: mid,
                delta_t,
                left_time: self.left_time
            },
            Self{
                left_pos: mid,
                right_pos: self.right_pos,
                delta_t,
                left_time: self.left_time + delta_t
            }
        )
    }
}

