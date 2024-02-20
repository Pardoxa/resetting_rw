use crate::sync_queue::*;
use std::f64::consts::SQRT_2;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::{cmp::Ordering, path::Path};
use std::sync::atomic::AtomicU64;

use rand_pcg::Pcg64;
use rand_distr::{Exp, Uniform};
use rayon::prelude::*;
use std::io::Write;
use serde::{Serialize, Deserialize};
use rand::{SeedableRng, distributions::Distribution};
use crate::*;
use rand_distr::StandardNormal;
use rand::prelude::*;

use self::{misc::*, parse::parse_and_add_to_global};

const RELAXED: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;

#[inline]
fn interpolate(
    old: f64, 
    new: f64,
    target: f64, 
    step_size: f64, 
    time_steps_performed: u64
) -> f64
{
    // now linear interpolation:
    let delta = new - old;
    let frac = (target - old) / delta;
    debug_assert!(frac >= 0.0);
    debug_assert!(frac <= 1.0);
    (time_steps_performed as f64 + frac) * step_size
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResettingUniWalkerHusk{
    pub rng_seed: u64,
    pub uni_mid: f64,
    pub uni_delta_2: f64,
    pub reset_lambda: f64,
    pub mirror_lambda: f64,
    pub target_pos: f64,
    pub step_size: f64
}

impl Default for ResettingUniWalkerHusk {
    fn default() -> Self {
        Self { 
            rng_seed: 123, 
            reset_lambda: 1.25, 
            target_pos: 1.0,
            step_size: 0.00025,
            mirror_lambda: 1.0,
            uni_delta_2: 0.1,
            uni_mid: -1.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirroringWalkerHistJob{
    pub rng_seed: u64,
    pub uni_mid: f64,
    pub uni_delta_2: f64,
    pub mirror_lambda: f64,
    pub step_size: f64,
    pub hist_positions: Vec<f64>,
    pub samples: NonZeroUsize
}

impl MirroringWalkerHistJob{

    pub fn get_times(&self) -> (Vec<u64>, Vec<u64>)
    {
        let times: Vec<_> = self.hist_positions
            .iter()
            .map(
                |val|
                {
                    (val / self.step_size).round() as u64
                }
            ).collect();

        let mut sum = 0;
        let step_helper: Vec<_> = times.iter()
            .map(
                |time|
                {
                    let s = time - sum;
                    sum += s;
                    s
                }
            ).collect();
        (times, step_helper)
    }

    fn get_walkers(&self) -> Vec<ResettingUniWalker>{
        let mut seed_rng = Pcg64::seed_from_u64(self.rng_seed);

        (0..self.samples.get())
            .map(
                |_|
                {
                    let rng = Pcg64::from_rng(&mut seed_rng).unwrap();
                    self.get_walker(rng)
                }
            ).collect()
    }

    fn get_walker(&self, rng: Pcg64) -> ResettingUniWalker
    {

        let reset_distr = Exp::<f64>::new(1.0).unwrap();
        let mirror_time_distr = Exp::new(self.mirror_lambda).unwrap();
        let low = self.uni_mid - self.uni_delta_2;
        let high = self.uni_mid + self.uni_delta_2;
        let mirror_dist = Uniform::new_inclusive(low, high);
        ResettingUniWalker { 
            rng, 
            x_pos: 0.0, 
            steps_until_next_mirror: 0,
            mirrors_performed: 0,
            reset_distr, 
            mirror_time_distr,
            mirror_dist,
            reset_lambda: 1.0,
            mirror_lambda: self.mirror_lambda, 
            time_steps_performed: 0, 
            target_pos: 0.0, 
            resets_performed: 0,
            steps_until_next_reset: 0,
            sqrt_step_size: self.step_size.sqrt(),
            step_size: self.step_size
        }
    }
}

impl Default for MirroringWalkerHistJob {
    fn default() -> Self {
        Self { 
            rng_seed: 123, 
            step_size: 0.00025,
            mirror_lambda: 1.0,
            uni_delta_2: 0.1,
            uni_mid: -1.0,
            hist_positions: vec![1.0, 2.0],
            samples: NonZeroUsize::new(1).unwrap()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ResettingUniWalker{
    rng: Pcg64,
    x_pos: f64,
    reset_distr: Exp<f64>,
    mirror_time_distr: Exp<f64>,
    mirror_dist: Uniform<f64>,
    reset_lambda: f64,
    mirror_lambda: f64,
    time_steps_performed: u64,
    target_pos: f64,
    resets_performed: u64,
    mirrors_performed: u64,
    steps_until_next_reset: u64,
    steps_until_next_mirror: u64,
    step_size: f64,
    sqrt_step_size: f64
}

pub enum What{
    Reset,
    Mirror,
    Both
}

impl ResettingUniWalker{

    pub fn reset_and_draw_next_reset_time(&mut self)
    {
        let reset_time = self.reset_distr.sample(&mut self.rng);
        let steps = (reset_time / self.step_size).floor() as u64;
        self.steps_until_next_reset = steps;
        self.resets_performed += 1;
        self.x_pos = 0.0;
    }

    pub fn mirror_and_draw_next_mirror_time(&mut self) -> f64
    {
        let mirror_time = self.mirror_time_distr.sample(&mut self.rng);
        let steps = (mirror_time / self.step_size).floor() as u64;
        self.steps_until_next_mirror = steps;
        self.mirrors_performed += 1; 
        let mirror_factor = self.mirror_dist.sample(&mut self.rng);
        self.x_pos *= mirror_factor;
        mirror_time
    }

    pub fn reset(&mut self)
    {
        self.reset_and_draw_next_reset_time();
        self.mirror_and_draw_next_mirror_time();
        self.mirrors_performed = 0;
        self.resets_performed = 0;
        self.x_pos = 0.0;
        self.time_steps_performed = 0;
    }

    /// Dont forget that you might need to call self.reset(); before calling this,
    /// depends on what you are doing
    pub fn only_mirror_steps(&mut self, mut steps: u64)
    {
        loop{
            let s = steps.min(self.steps_until_next_mirror);
            for _ in 0..s
            {
                self.x_pos += self.rng.sample::<f64,_>(StandardNormal) * self.sqrt_step_size;
            }
            self.time_steps_performed += s;
            steps -= s;
            self.steps_until_next_mirror -= s;
            if self.steps_until_next_mirror == 0{
                self.mirror_and_draw_next_mirror_time();
            }
            if steps == 0{
                break;
            }
        }
    }

    pub fn walk_until_found(&mut self) -> f64
    {
        self.reset();
        assert!(self.x_pos < self.target_pos);
        // ACHTUNG!!! SQRT2 added
        let sq_st = self.sqrt_step_size * SQRT_2;
        'outer: loop {
            let (steps, what) = match self.steps_until_next_mirror.cmp(&self.steps_until_next_reset)
            {
                Ordering::Equal => {
                    (self.steps_until_next_reset, What::Both)
                },
                Ordering::Less => {
                    (self.steps_until_next_mirror, What::Mirror)
                },
                Ordering::Greater => {
                    (self.steps_until_next_reset, What::Reset)
                }
            };
            
            for i in 0..steps
            {
                let old = self.x_pos;
                
                self.x_pos += self.rng.sample::<f64,_>(StandardNormal) * sq_st;
                if (old..=self.x_pos).contains(&self.target_pos){
                    
                    self.time_steps_performed += i;
                    let time = interpolate(
                        old, 
                        self.x_pos,
                        self.target_pos, 
                        self.step_size, 
                        self.time_steps_performed
                    );
                    self.time_steps_performed += 1;
                    
                    break 'outer time;
                }
            }
            self.time_steps_performed += steps;
            match what{
                What::Both => {
                    self.reset_and_draw_next_reset_time();
                    self.mirror_and_draw_next_mirror_time();
                },
                What::Mirror => {
                    self.steps_until_next_reset -= self.steps_until_next_mirror;
                    self.mirror_and_draw_next_mirror_time();
                },
                What::Reset => {
                    self.steps_until_next_mirror -= self.steps_until_next_reset;
                    self.reset_and_draw_next_reset_time();
                }
            }
            if self.target_pos == self.x_pos{
                let time = self.time_steps_performed as f64 * self.step_size;
                break time;
            }
        }
    }

    pub fn mirror_until_found(&mut self) -> f64
    {
        self.reset();
        assert!(self.x_pos < self.target_pos);
        // ACHTUNG!!! SQRT2 added
        let sq_st = SQRT_2 * self.sqrt_step_size;
        'outer: loop {
            let steps = self.steps_until_next_mirror;
            for i in 0..steps
            {
                let old = self.x_pos;
                self.x_pos += self.rng.sample::<f64,_>(StandardNormal) * sq_st;
                if (old..=self.x_pos).contains(&self.target_pos){
                    self.time_steps_performed += i;
                    let time = interpolate(
                        old, 
                        self.x_pos,
                        self.target_pos, 
                        self.step_size, 
                        self.time_steps_performed
                    );
                    self.time_steps_performed += 1;
                    break 'outer time;
                }
            }
            self.time_steps_performed += steps;
            self.mirror_and_draw_next_mirror_time();
            if self.target_pos == self.x_pos{
                let time = self.time_steps_performed as f64 * self.step_size;
                break time;
            }
        }
    }

    pub fn adaptive_mirror_until_found(&mut self) -> f64
    {
        #[inline]
        fn calc_stepsize(
            pos: f64, 
            target: f64,
        ) -> f64
        {
            let abs_dist = (target-pos).abs();
            //#[allow(clippy::suspicious_else_formatting)]
            let exp = if abs_dist >= 0.1
            {
                return 1e-5;
            }
            else if abs_dist <= 0.01 
            {
                abs_dist.mul_add(340.0, -12.0)
            } 
            else {
                abs_dist.mul_add(40.0, -9.0)
            };
            10.0_f64.powf(exp)
        }

        self.reset();
        assert!(self.x_pos < self.target_pos);
        let mut total_time = 0.0;
        // ACHTUNG!!! SQRT2 added
        'outer: loop {
            let mut time = 0.0;
            let mirror_time = self.mirror_and_draw_next_mirror_time();
            loop{
                let left = mirror_time - time;
                let mut sz = calc_stepsize(
                    self.x_pos, 
                    self.target_pos
                );
                sz = sz.min(left);
                //println!("sz {sz:e}");
                let sq_sz = SQRT_2 * sz.sqrt();
                let old = self.x_pos;
                self.x_pos += self.rng.sample::<f64,_>(StandardNormal) * sq_sz;
                self.time_steps_performed += 1;
                time += sz;
                if (old..=self.x_pos).contains(&self.target_pos){
                    break 'outer total_time + time;
                }
                if (time - mirror_time).abs() < 1e-9 {
                    total_time += time;
                    break;
                }
            }
        }
    }
}

impl From<ResettingUniWalkerHusk> for ResettingUniWalker
{
    fn from(value: ResettingUniWalkerHusk) -> Self {
        
        let rng = Pcg64::seed_from_u64(value.rng_seed);

        let reset_distr = Exp::new(value.reset_lambda).unwrap();
        let mirror_time_distr = Exp::new(value.mirror_lambda).unwrap();
        let low = value.uni_mid - value.uni_delta_2;
        let high = value.uni_mid + value.uni_delta_2;
        let mirror_dist = Uniform::new_inclusive(low, high);
        Self { 
            rng, 
            x_pos: 0.0, 
            steps_until_next_mirror: 0,
            mirrors_performed: 0,
            reset_distr, 
            mirror_time_distr,
            mirror_dist,
            reset_lambda: value.reset_lambda,
            mirror_lambda: value.mirror_lambda, 
            time_steps_performed: 0, 
            target_pos: value.target_pos, 
            resets_performed: 0,
            steps_until_next_reset: 0,
            sqrt_step_size: value.step_size.sqrt(),
            step_size: value.step_size
        }
    }
}

pub fn execute_uni(opts: UniScanOpts)
{
    execute_uni_helper(opts, ResettingUniWalker::walk_until_found, true);
}

pub fn execute_uni_only_mirror(opts: UniScanOpts)
{
    execute_uni_helper(opts, ResettingUniWalker::mirror_until_found, true);
}

pub fn execute_uni_only_mirror_adaptive(opts: UniScanOpts)
{
    execute_uni_helper(opts, ResettingUniWalker::adaptive_mirror_until_found, false);
}

pub fn execute_uni_helper<F>(opts: UniScanOpts, fun: F, const_step_size: bool)
where F: Sync + Fn(&mut ResettingUniWalker) -> f64
{
    let husk: ResettingUniWalkerHusk = parse_and_add_to_global(opts.json);

    let mut buf = create_buf_with_command_and_version(opts.out.as_deref().unwrap());
    let mut header = vec![
        "lambda",
        "average_resets",
        "average_steps",
        "average_mirrors",
        "average_time"
    ];
    if const_step_size{
        header.push("interpolated_average_time");
    }
    write_slice_head(&mut buf, header).unwrap();

    let step_size = husk.step_size;
    let total_samples = opts.samples as f64;
    for i in 0..opts.lambda_samples{

        let lambda = opts.lambda_start + i as f64 *(opts.lambda_end - opts.lambda_start) / (opts.lambda_samples - 1) as f64;

        let mut husk = husk.clone();
        husk.mirror_lambda = lambda;

        let mut walker: ResettingUniWalker = husk.into(); 

        let queue = SyncQueue::create_work_queue(
            opts.samples, 
            NonZeroUsize::new(opts.threads.get() * 3).unwrap()
        );

        let queue = queue.map(
            |amount|
            {
                let mut w = walker.clone();
                w.rng = Pcg64::from_rng(&mut walker.rng).unwrap();
                w.reset();
                (w, amount)
            }
        );

        let samples_per_packet = (opts.samples / (opts.threads.get() * 12)).max(1);

        let sum_resets = AtomicU64::new(0);
        let sum_mirrors = AtomicU64::new(0);
        let sum_time_steps = AtomicU64::new(0);
        let sum_time = Mutex::new(0.0);

        (0..opts.threads.get())
            .into_par_iter()
            .for_each(
                |_|
                {
                    let mut tmp_time_sum = 0.0;
                    let mut tmp_sum_resets = 0;
                    let mut tmp_sum_time_steps = 0;
                    let mut tmp_sum_mirrors = 0;
                    while let Some((mut walker, amount)) = queue.pop() {
                        let work = amount.min(samples_per_packet);
                        let left = amount - work;

                        for _ in 0..work{
                            let time = fun(&mut walker);
                            tmp_time_sum += time;
                            tmp_sum_resets += walker.resets_performed;
                            tmp_sum_time_steps += walker.time_steps_performed;
                            tmp_sum_mirrors += walker.mirrors_performed;
                        }
                        if left > 0{
                            queue.push(
                                (walker, left)
                            );
                        }
                    }
                    sum_resets.fetch_add(tmp_sum_resets, RELAXED);
                    sum_time_steps.fetch_add(tmp_sum_time_steps, RELAXED);
                    sum_mirrors.fetch_add(tmp_sum_mirrors, RELAXED);
                    let mut time_sum_lock = sum_time.lock().unwrap();
                    *time_sum_lock += tmp_time_sum;
                    drop(time_sum_lock);
                }
            );


        let sum_resets = sum_resets.into_inner();
        let sum_time_steps = sum_time_steps.into_inner();
        let sum_mirrors = sum_mirrors.into_inner();

        let average_time_interpol = sum_time.into_inner().unwrap() / total_samples;

        let average_resets = sum_resets as f64 / total_samples;
        let average_steps = sum_time_steps as f64 / total_samples;
        let average_time = average_steps * step_size;
        let average_mirrors = sum_mirrors as f64 / total_samples;
        if const_step_size{
            println!("lambda {lambda} average resets: {average_resets}, average_steps {average_steps} average_time {average_time} interp_time {average_time_interpol}");
            writeln!(buf, "{lambda} {average_resets} {average_steps} {average_mirrors} {average_time} {average_time_interpol}")
        } else {
            println!("lambda {lambda} average resets: {average_resets}, average_steps {average_steps} average_time {average_time_interpol}");
            writeln!(buf, "{lambda} {average_resets} {average_steps} {average_mirrors} {average_time_interpol}")
        }.unwrap();
    }
}

pub fn exec_mirroring_hists<P>(path: Option<P>)
where P: AsRef<Path>
{
    exec_hist_helper(path, ResettingUniWalker::only_mirror_steps)
}

fn exec_hist_helper<P, F>(path: Option<P>, fun: F)
where P: AsRef<Path>,
    F: Fn(&mut ResettingUniWalker, u64) + Sync
{
    let opt: MirroringWalkerHistJob = parse_and_add_to_global(path);

    let mut walkers = opt.get_walkers();

    let (times, step_helper) = opt.get_times();

    let writer: Vec<_> = times.iter()
        .map(
            |time|
            {
                let name = format!("test_{time}.dat");
                let buf = create_buf_with_command_and_version(name);
                Mutex::new(buf)
            }
        ).collect();

    walkers.par_iter_mut()
        .for_each(
            |walker|
            {
                step_helper
                    .iter()
                    .zip(writer.iter())
                    .for_each(
                        |(&steps, writer)|
                        {
                            fun(walker, steps);
                            
                            let pos = walker.x_pos;
                            let mut lock = writer.lock().unwrap();
                            writeln!(lock, "{pos:e}").unwrap();
                            drop(lock);
                        }
                    )
            }
        )
}