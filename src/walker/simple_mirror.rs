use std::f64::consts::SQRT_2;
use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::{AtomicU64, Ordering};

use rand_pcg::Pcg64;
use rand_distr::Exp;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use sampling::AtomicHistU32;
use std::io::Write;
use serde::{Serialize, Deserialize};
use rand::{SeedableRng, distributions::Distribution};
use crate::*;
use crate::parse::parse;
use rand_distr::StandardNormal;
use rand::prelude::*;


pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResettingMirrorWalkerHusk{
    pub rng_seed: u64,
    pub exp_lambda: f64,
    pub target_pos: f64,
    pub step_size: f64
}

impl Default for ResettingMirrorWalkerHusk {
    fn default() -> Self {
        Self { 
            rng_seed: 123, 
            exp_lambda: 0.01, 
            target_pos: 2.0,
            step_size: 0.00025
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ResettingMirrorWalker{
    rng: Pcg64,
    x_pos: f64,
    distr: Exp<f64>,
    exp_lambda: f64,
    time_steps_performed: u32,
    target_pos: f64,
    resets_performed: u32,
    mirrors_performed: u32,
    steps_until_next_reset: u32,
    step_size: f64,
    sqrt_step_size: f64,
    mirror_prob: f64
}

impl ResettingMirrorWalker{

    pub fn draw_next_reset_or_mirror_time(&mut self)
    {
        let reset_time = self.distr.sample(&mut self.rng);
        let steps = (reset_time / self.step_size).floor() as u32;
        self.steps_until_next_reset = steps;
        
        let decision: f64 = self.rng.gen();
        if decision < self.mirror_prob {
            self.x_pos = -self.x_pos;
            self.mirrors_performed += 1;
        } else {
            self.x_pos = 0.0;
            self.resets_performed += 1;

        }
    }

    pub fn reset(&mut self)
    {
        let reset_time = self.distr.sample(&mut self.rng);
        let steps = (reset_time / self.step_size).floor() as u32;
        self.steps_until_next_reset = steps;
        self.resets_performed = 0;
        self.mirrors_performed = 0;
        self.x_pos = 0.0;
        self.time_steps_performed = 0;
    }

    pub fn walk_until_found(&mut self)
    {
        self.reset();
        assert!(self.x_pos < self.target_pos);
        let sq = self.sqrt_step_size * SQRT_2;
        'outer: loop {
            for _ in 0..self.steps_until_next_reset
            {
                let old = self.x_pos;
                self.x_pos += self.rng.sample::<f64,_>(StandardNormal) * sq;
                self.time_steps_performed += 1;
                if (old..=self.x_pos).contains(&self.target_pos){
                    break 'outer;
                }
            }
            self.draw_next_reset_or_mirror_time();
            if self.target_pos == self.x_pos{
                break;
            }
        }
    }

    fn from(value: ResettingMirrorWalkerHusk, mirror_prob: f64) -> Self {
        
        let rng = Pcg64::seed_from_u64(value.rng_seed);

        let distr = Exp::new(value.exp_lambda).unwrap();
        Self { 
            rng, 
            x_pos: 0.0, 
            distr, 
            exp_lambda: value.exp_lambda, 
            time_steps_performed: 0, 
            target_pos: value.target_pos, 
            resets_performed: 0,
            mirrors_performed: 0,
            steps_until_next_reset: 0,
            sqrt_step_size: value.step_size.sqrt(),
            step_size: value.step_size,
            mirror_prob
        }
    }
}

pub fn execute_mirror(opts: MirrorScanOpts)
{
    let (husk, json): (ResettingMirrorWalkerHusk, _) = parse(opts.json);

    let samples = opts.samples;
    let name = format!("v{VERSION}_mirror_scan_p_{}_samples{samples}.dat", opts.mirror_prob);
    println!("creating {name}");

    let file = File::create(name).unwrap();
    let mut buf = BufWriter::new(file);

    misc::write_json(&mut buf, &json);
    misc::write_commands(&mut buf).unwrap();
    writeln!(buf, "#lambda average_resets var_resets average_steps average_time var_time").unwrap();
    let samples_per_thread = opts.samples / opts.threads;
    for i in 0..opts.lambda_samples{

        let lambda = opts.lambda_start + i as f64 *(opts.lambda_end - opts.lambda_start) / (opts.lambda_samples - 1) as f64;

        let mut husk = husk.clone();
        husk.exp_lambda = lambda;

        let mut walker: ResettingMirrorWalker = ResettingMirrorWalker::from(husk, opts.mirror_prob);

        let mut thread_walker: Vec<_> = (0..opts.threads)
            .map(
                |_|
                {
                    let mut w = walker.clone();
                    w.rng = Pcg64::from_rng(&mut walker.rng).unwrap();
                    w.reset();
                    w
                }
            ).collect();

        

        let sum_resets = AtomicU64::new(0);
        let sum_time_steps = AtomicU64::new(0);

        let sum_resets_sq = AtomicU64::new(0);
        let sum_time_steps_sq = AtomicU64::new(0);

        thread_walker.par_iter_mut()
            .for_each(
                |walker|
                {
                    for _ in 0..samples_per_thread{
                        walker.walk_until_found();
                        let resets = walker.resets_performed;
                        let time_steps = walker.time_steps_performed;
                        sum_time_steps_sq.fetch_add(time_steps as u64 * time_steps as u64, Ordering::Relaxed);
                        sum_resets.fetch_add(resets as u64, Ordering::Relaxed);
                        sum_time_steps.fetch_add(time_steps as u64, Ordering::Relaxed);
                        sum_resets_sq.fetch_add((resets * resets) as u64, Ordering::Relaxed);
                    }
                }
            );

        let sum_resets = sum_resets.into_inner();
        let sum_time_steps = sum_time_steps.into_inner();

        let total_samples = opts.threads * samples_per_thread;

        let step_size = thread_walker[0].step_size;
        let average_resets = sum_resets as f64 / total_samples as f64;
        let average_steps = sum_time_steps as f64 / total_samples as f64;
        let average_time = average_steps * step_size;

        let var_time = sum_time_steps_sq.into_inner() as f64 * step_size * step_size  / total_samples as f64 - average_time * average_time;
        let var_resets = sum_resets_sq.into_inner() as f64 / total_samples as f64 - average_resets * average_resets;

        println!("lambda {lambda} average resets: {average_resets} var: {var_resets}, average_steps {average_steps} average_time {average_time} var {var_time}");
        writeln!(buf, "{lambda} {average_resets} {var_resets} {average_steps} {average_time} {var_time}").unwrap();
    }
}

pub fn execute_simple_mirror_reset_pdf(opts: MirrorResetPdfOpts)
{
    let (husk, _): (ResettingMirrorWalkerHusk, _) = parse(opts.json);

    let name = format!("mirror_pdf_{}.dat", opts.mirror_prob);
    println!("creating: {name}");
    let file = File::create(name).unwrap();
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#resets pdf").unwrap();

    let mut walker: ResettingMirrorWalker = ResettingMirrorWalker::from(husk, opts.mirror_prob);
    let mut thread_walker: Vec<_> = (0..opts.threads)
        .map(
            |_|
            {
                let mut w = walker.clone();
                w.rng = Pcg64::from_rng(&mut walker.rng).unwrap();
                w.reset();
                w
            }
        ).collect();
    let samples_per_thread = opts.samples / opts.threads;
    let mut hist = AtomicHistU32::new_inclusive(0, opts.max_resets, (opts.max_resets + 1) as usize)
        .unwrap();
    thread_walker.par_iter_mut()
        .for_each(
            |walker|
            {
                for _ in 0..samples_per_thread{
                    walker.walk_until_found();
                    hist.increment(walker.resets_performed).unwrap();
                }
            }
        );

    let total_samples = opts.threads * samples_per_thread;
    for (bin, hits) in hist.bin_hits_iter()
    {
        let prob = hits as f64 / total_samples as f64;
        writeln!(buf, "{} {} {prob}", bin[0], hits).unwrap()
    }
    
}