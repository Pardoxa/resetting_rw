use std::{
    collections::{BTreeMap, BinaryHeap},
    f64::consts::SQRT_2,
    io::Write, num::NonZeroUsize,
    sync::Mutex
};
use camino::Utf8PathBuf;
use indicatif::{ProgressIterator, ProgressStyle};
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, StandardNormal};
use rand_pcg::{Pcg32, Pcg64, Pcg64Mcg};
use serde::{Serialize, Deserialize};
use derivative::Derivative;
use rayon::prelude::*;

use crate::{
    misc::{create_buf_with_command_and_version, write_slice_head},
    sync_queue::SyncQueue
};

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
    bisection_amount: usize
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
        let global_sum_fpt = Mutex::new(0.0);
        (0..opt.j.get())
            .into_par_iter()
            .for_each(
                |_|
                {
                    let mut sum_fpt = 0.0;
                    while let Some((mut walker, amount)) = queue.pop() {
                        let work = amount.min(samples_per_packet);
                        let left = amount - work;

                        for _ in 0..work{
                            for _ in 0..opt.bisection_amount{
                                walker.bisection_step();
                            }
                            sum_fpt += walker.fpt;
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
        let mfpt = global_sum_fpt.into_inner().unwrap() / opt.samples_per_point.get() as f64;
        writeln!(
            buf,
            "{lambda} {mfpt}"
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
        lambda_mirror: 0.1
    };
    let rng = Pcg64Mcg::seed_from_u64(0xff00abcf);
    let mut walker = EffRandWalk::new(
        settings,
        rng
    );
    for _ in 0..100000{
        walker.bisection_step();
    }
    for (idx, walk) in walker.walk.iter().enumerate(){
        let name = format!("walker{idx}");
        let mut buf = create_buf_with_command_and_version(name);
        for bla in walk.iter(){
            let time = bla.0;
            let pos = bla.1.left_pos;
            writeln!(buf, "{time} {pos}").unwrap();
        }
        let (last_time, last_val) = walk.last_key_value().unwrap();
        let time = last_time + last_val.delta_t;
        let val = last_val.right_pos;
        writeln!(buf, "{time} {val}").unwrap();
    }
}

#[derive(Debug)]
pub struct EffRandWalk<R>
{
    // Later I should check if HashMap is faster!
    walk: Vec<BTreeMap<OrderedFloat<f64>, Delta>>,
    prob: BinaryHeap<NextProb>,
    fpt: f64,
    rng: R,
    settings: RadomWalkSettings
}

#[derive(Debug)]
pub struct NextProb
{
    which_vec: usize,
    time: OrderedFloat<f64>,
    prob: OrderedFloat<f64>,
}

impl Eq for NextProb {}

impl PartialEq for NextProb {
    fn eq(&self, other: &Self) -> bool {
        self.prob == other.prob && self.time == other.time
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
    #[derivative(Default(value="2e-5"))]
    rough_step_size: f64,
    #[derivative(Default(value="1.0"))]
    target: f64,
    a: f64,
    #[derivative(Default(value="14"))]
    max_depth: usize
}

fn create_initial_walk<R>(
    settings: &RadomWalkSettings,
    mut rng: R,
    walk: &mut BTreeMap<OrderedFloat<f64>, Delta>
) -> f64
where R: Rng
{
    walk.clear();
    let mirror_dist = Exp::new(settings.lambda_mirror)
        .unwrap();
    let mut next_mirror_time = mirror_dist.sample(&mut rng);
    let sqrt_step_size = settings.rough_step_size.sqrt();
    let sq = sqrt_step_size * SQRT_2;
    let mut current_pos = 0.0;
    let mut current_time = 0.0;
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
                delta_t: settings.rough_step_size
            };
            walk.insert(OrderedFloat(left_time), delta);
            if (left_pos..=current_pos).contains(&settings.target)
            {
                // TODO in here I could linerarly interpolate to get a more accurate result
                let fpt = settings.rough_step_size.mul_add((i + 1) as f64, current_time);
                return fpt;
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
            delta_t: rest
        };
        walk.insert(OrderedFloat(left_time), delta);
        if (left_pos..=current_pos).contains(&settings.target)
        {
            // TODO in here I could linerarly interpolate to get a more accurate result
            let fpt = current_time;
            return fpt;
        }
        current_pos *= settings.a;
        next_mirror_time = mirror_dist.sample(&mut rng);
    }
}

fn calc_heap(
    target: f64,
    walk: &BTreeMap<OrderedFloat<f64>, Delta>,
    heap: &mut BinaryHeap<NextProb>
)
{
    heap.clear();
    heap.extend(
        walk
            .iter()
            .map(
                |(time, val)|
                {
                    let prob = val.calc_prob(target);
                    NextProb{
                        which_vec: 0,
                        time: *time,
                        prob: OrderedFloat(prob)
                    }
                }
            )
    );
}


impl<R> EffRandWalk<R>
where R: Rng
{
    pub fn new(
        settings: RadomWalkSettings,
        mut rng: R,
    ) -> Self
    {
        let mut initial_walk = BTreeMap::new();
        let fpt = create_initial_walk(
            &settings, 
            &mut rng,
            &mut initial_walk
        );
        let mut heap = BinaryHeap::new();
        calc_heap(settings.target, &initial_walk, &mut heap);
        let mut walk = vec![initial_walk];
        walk.extend(
            (1..settings.max_depth).map(|_| BTreeMap::new())
        );
        Self {
            walk, 
            prob: heap, 
            fpt,
            rng,
            settings
        }
    }

    #[allow(dead_code)]
    pub fn recycle(&mut self)
    {
        self.walk[1..]
            .iter_mut()
            .for_each(|walk| walk.clear());
        let fpt = create_initial_walk(
            &self.settings, 
            &mut self.rng, 
            &mut self.walk[0]
        );
        self.fpt = fpt;
        calc_heap(
            self.settings.target, 
            &self.walk[0], 
            &mut self.prob
        );
    }

    fn bisection_step(&mut self)
    {
        while let Some(val) = self.prob.pop(){
            
            // TODO: Check if this is correct or if there is a delta t missing for val.time
            if val.time.into_inner() > self.fpt + 1e-4 {
                continue;
            }

            let delta = self.walk[val.which_vec].get(&val.time)
                .expect("Has to exist!");

            let (left, right) = delta.bisect(&mut self.rng);
            if left.contains(&self.settings.target)
            {
                self.fpt = self.fpt.min(val.time.into_inner() + left.delta_t);
            }

            let next_vec_id = val.which_vec + 1;
            let time_right = val.time + left.delta_t;
            if next_vec_id + 1 < self.walk.len(){
                let prob_left = left.calc_prob(self.settings.target);
                let prob_right = right.calc_prob(self.settings.target);
                self.prob.push(
                    NextProb { which_vec: next_vec_id, time: val.time, prob: OrderedFloat(prob_left) }
                );
                self.prob.push(
                    NextProb { which_vec: next_vec_id, time: time_right, prob: OrderedFloat(prob_right) }
                );
            }
            self.walk[next_vec_id].insert(val.time, left);
            self.walk[next_vec_id].insert(time_right, right);
            
            break;
        }
    }
}

#[derive(Debug)]
pub struct Delta{
    left_pos: f64,
    right_pos: f64,
    delta_t: f64
}

impl Delta{
    pub fn calc_prob(&self, target: f64) -> f64
    {
        // Currently only implemented as test case, 
        // insert correct equation later!
        // TODO
        let diff = self.left_pos - target;
        self.delta_t / (diff * diff)
    }

    pub fn contains(&self, target: &f64) -> bool
    {
        (self.left_pos..=self.right_pos).contains(target)
    }

    // Uses a brownian bridge
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
                delta_t
            },
            Self{
                left_pos: mid,
                right_pos: self.right_pos,
                delta_t
            }
        )
    }
}

