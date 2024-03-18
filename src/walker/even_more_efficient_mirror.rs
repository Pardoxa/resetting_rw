use std::{
    collections::VecDeque, 
    f64::consts::SQRT_2, 
    io::Write, 
    num::*, 
    sync::Mutex
};
use camino::Utf8PathBuf;
use indicatif::{ProgressIterator, ProgressStyle};
use kahan::KahanSum;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, StandardNormal};
use rand_pcg::{Pcg32, Pcg64};
use rayon::prelude::*;
use super::{Delta, RadomWalkSettings, MeasureMfptBetaOpt};
use crate::{misc::*, sync_queue::*};

#[derive(Debug)]
pub struct DeltaWithLevel
{
    delta: Delta,
    level: usize
}

#[derive(Debug)]
pub struct EffRandWalk2<R>
{
    stack_queue: VecDeque<DeltaWithLevel>,
    delta_fpt: Delta,
    fpt: f64,
    rng: R,
    settings: RadomWalkSettings,
    threshold: f64
}

fn create_initial_walk<R>(
    settings: &RadomWalkSettings,
    mut rng: R,
    stack_queue: &mut VecDeque<DeltaWithLevel>,
    threshold: f64
) -> Delta
where R: Rng
{
    stack_queue.clear();
    let mirror_dist = Exp::new(settings.lambda_mirror)
        .unwrap();
    let mut next_mirror_time = mirror_dist.sample(&mut rng);
    let sqrt_step_size = settings.rough_step_size.sqrt();
    let sq = sqrt_step_size * SQRT_2;
    let mut current_pos = settings.origin;
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
                delta_t: settings.rough_step_size,
                left_time
            };
            let prob = delta.calc_prob(settings.target);
            let contained = delta.contains(&settings.target);
            if prob > threshold{
                let item = DeltaWithLevel{
                    delta,
                    level: 0
                };
                stack_queue.push_back(item);
            }
            
            if contained
            {
                return delta;
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
        let prob = delta.calc_prob(settings.target);
        if prob > threshold{
            let item = DeltaWithLevel{
                delta,
                level: 0
            };
            stack_queue.push_back(item);
        }
        if (left_pos..=current_pos).contains(&settings.target)
        {
           return delta;
        }
        current_pos *= settings.a;
        next_mirror_time = mirror_dist.sample(&mut rng);
    }
}

impl<R> EffRandWalk2<R> 
where R: Rng + SeedableRng
{
    pub fn new(
        settings: RadomWalkSettings,
        mut rng: R,
        threshold: f64
    ) -> Self
    {
        let mut stack_queue = VecDeque::with_capacity(1024*1024);
        let delta = create_initial_walk(
            &settings, 
            &mut rng,
            &mut stack_queue,
            threshold
        );
        let fpt = delta.left_time + delta.delta_t;

        Self {
            stack_queue, 
            fpt,
            settings,
            rng,
            delta_fpt: delta,
            threshold
        }
    }


    pub fn recycle(&mut self)
    {
        let delta_fpt = create_initial_walk(
            &self.settings, 
            &mut self.rng, 
            &mut self.stack_queue, 
            self.threshold
        );
        self.fpt = delta_fpt.left_time + delta_fpt.delta_t;
        self.delta_fpt = delta_fpt;
    }

    fn bisection(&mut self, threshold: f64)
    {
        let max_len = self.settings.max_depth;
        while let Some(item) = self.stack_queue.pop_front(){

            let next_level = item.level + 1;
            let (left, right) = item.delta.bisect(&mut self.rng);
            let mut add_right = true;
            if left.contains(&self.settings.target)
            {
                self.stack_queue.clear();
                self.fpt = left.left_time + left.delta_t;
                self.delta_fpt = left;
                add_right = false;
            } else if right.contains(&self.settings.target) {
                self.stack_queue.clear();
                self.fpt = right.left_time + right.delta_t;
                self.delta_fpt = right;
            }

            if next_level < max_len {
                let prob_left = left.calc_prob(self.settings.target);
                let prob_right = right.calc_prob(self.settings.target);
                if add_right && prob_right > threshold{
                    let right = DeltaWithLevel{
                        delta: right,
                        level: next_level
                    };
                    self.stack_queue.push_front(
                        right
                    );
                }
                if prob_left > threshold{
                    let left = DeltaWithLevel{
                        delta: left,
                        level: next_level
                    };
                    self.stack_queue.push_front(
                        left
                    );
                }
            }
        }
    }
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
    let threshold = opt.bisection.threshold()
        .expect("Only bisection with threshold allowed here!");
    
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
                let walk = EffRandWalk2::new(
                    settings.clone(), 
                    rng,
                    threshold
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
                            walker.bisection(threshold);
                            let fpt = walker.delta_fpt.interpolate(walker.settings.target);
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