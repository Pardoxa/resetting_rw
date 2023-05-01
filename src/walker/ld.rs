
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand::{Rng, seq::SliceRandom};
use rand_distr::{Exp, OpenClosed01, Uniform, Distribution};
use serde::{Serialize, Deserialize};
use sampling::{MarkovChain, HistUsizeFast};
use crate::WlPdfOpts;
use super::simple::ResettingWalkerHusk;
use crate::parse::parse;
use sampling::*;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::time::*;

const EPS: [f64; 8] = [1e0, 1e-1, 1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7];

#[inline]
pub fn box_müller(u1: f64, u2: f64) -> f64
{
    let f = (-2.0 * u1.ln()).sqrt();
    let inner = std::f64::consts::TAU * u2;
    inner.cos() * f
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResettingWalkerLdHusk{
    rng_seed: u64,
    exp_lambda: f64,
    target_pos: f64,
    step_size: f64
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Noise
{
    noise_source: Vec<f64>,
    actual_noise: Vec<f64>
}

impl Noise
{

    pub fn new<R: Rng>(size: usize, mut rng: R, sigma: f64) -> Self
    {
        let noise_iter = <OpenClosed01 as rand_distr::Distribution<f64>>::sample_iter(OpenClosed01, &mut rng)
            .take(size * 2);

        let noise_source: Vec<_> = noise_iter.collect();

        let actual_noise: Vec<_> = noise_source
            .chunks_exact(2)
            .map(
                |chunk|
                {
                    box_müller(chunk[0], chunk[1]) * sigma
                }
            ).collect();
        
        Self { noise_source, actual_noise }
        
    }

    #[allow(dead_code)]
    pub fn re_randomize<R: Rng>(&mut self, mut rng: R, sigma: f64)
    {
        let noise_iter = <OpenClosed01 as rand_distr::Distribution<f64>>::sample_iter(OpenClosed01, &mut rng);
        self.noise_source.iter_mut()
            .zip(noise_iter)
            .for_each(|(val, new_val)| *val = new_val);

        self.actual_noise.iter_mut()
            .zip(self.noise_source.chunks_exact(2))
            .for_each(
                |(val, chunk)|
                {
                    *val = box_müller(chunk[0], chunk[1]) * sigma
                }
            );
    }


    #[inline]
    pub fn get(&self, index: usize) -> f64
    {
        unsafe{*self.actual_noise.get_unchecked(index)}
    }

    #[inline]
    pub fn draw_new<R: Rng>(&mut self, index: usize, mut rng: R, sigma: f64) -> [f64; 2]
    {
        let start = index * 2;
        let slice = &mut self.noise_source[start..=start+1];
        let old = [slice[0], slice[1]];

        let noise_iter = <OpenClosed01 as rand_distr::Distribution<f64>>::sample_iter(OpenClosed01, &mut rng);

        slice.iter_mut()
            .zip(noise_iter)
            .for_each(
                |(to_change, new_val)| *to_change = new_val
            );

        let new_noise = box_müller(slice[0], slice[1]) * sigma;
        self.actual_noise[index] = new_noise;

        old
    }

    #[inline]
    pub fn slight_change<R: Rng>(
        &mut self, 
        index: usize, 
        mut rng: R, 
        sigma: f64, 
        eps: f64
    ) -> [f64; 2]
    {
        let uniform = Uniform::new_inclusive(-1.0_f64, 1.0);
        let start = index * 2;
        let slice = &mut self.noise_source[start..=start+1];
        let old = [slice[0], slice[1]];

        let decision: f64 = rng.gen();

        if decision < 1.0/3.0 {
            slice[0] = uniform.sample(&mut rng).mul_add(eps, old[0]);
            if slice[0] > 1.0 || slice[0] <= 0.0 {
                slice[0] = old[0];
            }
        } else if decision < 2.0/3.0{
            slice[1] = uniform.sample(&mut rng).mul_add(eps, old[1]);
            if slice[1] > 1.0 || slice[1] <= 0.0 {
                slice[1] = old[1];
            }
        } else {
            slice[0] = uniform.sample(&mut rng).mul_add(eps, old[0]);
            slice[1] = uniform.sample(&mut rng).mul_add(eps, old[1]);
            if slice[0] > 1.0 || slice[0] <= 0.0 {
                slice[0] = old[0];
            }
            if slice[1] > 1.0 || slice[1] <= 0.0 {
                slice[1] = old[1];
            }
        }

        let new_noise = box_müller(slice[0], slice[1]) * sigma;
        self.actual_noise[index] = new_noise;

        old
    }

    #[inline]
    pub fn undo(&mut self, index: usize, floats: [f64; 2], sigma: f64)
    { 
        let start = index * 2;
        let slice = &mut self.noise_source[start..=start+1];
        slice[0] = floats[0];
        slice[1] = floats[1];

        let old_noise = box_müller(floats[0], floats[1]) * sigma;

        self.actual_noise[index] = old_noise;

    }
}

#[derive(Clone)]
pub struct ResetTimes{
    pub resets: Vec<usize>,
    distr: Exp<f64>,
    current_idx: usize,
    step_size: f64
}

impl ResetTimes {

    pub fn new(lambda: f64, step_size: f64) -> Self 
    {
        Self { resets: Vec::new(), distr: Exp::new(lambda).unwrap(), current_idx: 0, step_size }
    }

    pub fn next<R>(&mut self, mut rng: R) -> usize
        where R: Rng
    {
        match self.resets.get(self.current_idx)
        {
            Some(val) => {
                self.current_idx += 1;
                *val
            },
            None => {
                let reset_time = self.distr.sample(&mut rng);
                let steps = (reset_time / self.step_size).floor() as usize;
                self.resets.push(steps);
                self.next(rng)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct LdResettingWalker{
    rng: Pcg64,
    x_pos: f64,
    exp_lambda: f64,
    time_steps_performed: u32,
    target_pos: f64,
    step_size: f64,
    sqrt_step_size: f64,
    noise: Noise,
    reset_times: ResetTimes
}

impl LdResettingWalker{

    pub fn new(husk: ResettingWalkerHusk, opts: &WlPdfOpts) -> Self
    {
        let mut rng = Pcg64::seed_from_u64(husk.rng_seed);

        Self { 
            noise: Noise::new(opts.time_limit_of_sample, &mut rng, 1.0), 
            rng, 
            x_pos: 0.0, 
            exp_lambda: husk.exp_lambda, 
            time_steps_performed: 0,
            target_pos: husk.target_pos,
            step_size: husk.step_size, 
            sqrt_step_size: husk.step_size.sqrt(), 
            reset_times: ResetTimes::new(husk.exp_lambda, husk.step_size)
        }
    }

    pub fn reset(&mut self)
    {
        self.x_pos = 0.0;
        self.time_steps_performed = 0;
        self.reset_times.current_idx = 0;
    }


    pub fn calc_resets(&mut self) -> Option<usize>
    {
        self.reset();
        assert!(self.x_pos < self.target_pos);
        let mut idx = 0;
        let mut resets = 0;
        'outer: loop {
            for _ in 0..self.reset_times.next(&mut self.rng)
            {
                if idx > self.noise.actual_noise.len(){
                    return None;
                }
                let gaus = self.noise.get(idx);
                
                let change = gaus * self.sqrt_step_size;
                //println!("{change} {} {resets}", self.x_pos);
                self.x_pos += change;
                idx += 1;
                if self.x_pos >= self.target_pos{
                    break 'outer;
                }
            }
            resets += 1;
        }
        Some(resets)
    }
}

pub struct NoiseChange{
    idx: usize,
    change: [f64;2]
}

pub enum MarkovStep{
    ResetMove(usize, usize),
    NoiseMove(Vec<NoiseChange>)
}

impl MarkovChain<MarkovStep, ()> for LdResettingWalker
{
    fn m_step(&mut self) -> MarkovStep {
        let p: f64 = self.rng.gen();
        if p < 0.01 {
            let uni = Uniform::new(0, self.reset_times.resets.len());
            let idx = uni.sample(&mut self.rng);
            let old = self.reset_times.resets[idx];
            let new_reset_time = self.reset_times.distr.sample(&mut self.rng);
            self.reset_times.resets[idx] = (new_reset_time / self.step_size).floor() as usize;
            MarkovStep::ResetMove(idx, old)

        }else if p < 0.1 {
            let uni = Uniform::new(0, self.noise.actual_noise.len());
            let mut noise_changes = Vec::new();
            for _ in 0..3000 {
                let idx = uni.sample(&mut self.rng);
                let old = self.noise.draw_new(idx, &mut self.rng, 1.0);
                noise_changes.push(NoiseChange{idx, change: old});
            }
            MarkovStep::NoiseMove(noise_changes)
        } else {
            let uni = Uniform::new(0, self.noise.actual_noise.len());
            let mut noise_changes = Vec::new();
            for _ in 0..3000 {
                let idx = uni.sample(&mut self.rng);
                let eps = EPS.choose(&mut self.rng).unwrap();
                let old = self.noise.slight_change(idx, &mut self.rng, 1.0, *eps);
                noise_changes.push(NoiseChange{idx, change: old});
            }
            MarkovStep::NoiseMove(noise_changes)
        }
    }

    fn undo_step(&mut self, step: &MarkovStep) {
        match step{
            MarkovStep::ResetMove(index, old_val) => {
                self.reset_times.resets[*index] = *old_val;
            }, 
            MarkovStep::NoiseMove(noise_move) => {
                for n_move in noise_move.iter().rev(){
                    self.noise.undo(n_move.idx, n_move.change, 1.0);
                }
            }
        }
    }

    fn undo_step_quiet(&mut self, step: &MarkovStep) {
        self.undo_step(step)
    }
}


pub fn execute_wl_reset_pdf(opts: WlPdfOpts)
{
    let start = Instant::now();
    let (husk, json): (ResettingWalkerHusk, _) = parse(opts.json.clone());

    let file = File::create("pdf.dat").unwrap();
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#resets pdf").unwrap();

    let mut walker = LdResettingWalker::new(husk, &opts);

    let hist = HistUsizeFast::new_inclusive(0, opts.max_resets as usize)
        .unwrap();

    let rng = Pcg64::from_rng(&mut walker.rng).unwrap();

    let mut wl = WangLandau1T::new(
        1e-6, 
        walker, 
        rng, 
        1, 
        hist, 
        1000
    ).unwrap();

    wl.init_greedy_heuristic(LdResettingWalker::calc_resets, None).unwrap();

    let time = opts.max_time_in_minutes as u64 * 60;
   
    unsafe{
        wl.wang_landau_while_unsafe(LdResettingWalker::calc_resets, |_| start.elapsed().as_secs() < time);
    }


    let mut density = wl.log_density_base10();
    sampling::glue::norm_log10_sum_to_1(&mut density);
    
    let file = File::create("wl_pdf.dat")
        .unwrap();

    let mut buf = BufWriter::new(file);

    writeln!(buf, "#bin prob").unwrap();
    write!(buf, "#").unwrap();
    serde_json::to_writer(&mut buf, &json).unwrap();
    write_commands(&mut buf).unwrap();

    for (index, p) in density.iter().enumerate()
    {
        writeln!(buf, "{index} {p}").unwrap()
    }

    let time_passed = humantime::format_duration(start.elapsed());
    let log_f = wl.log_f();
    println!("time passed: {time_passed} log_f: {log_f}");
    writeln!(buf, "#time passed: {time_passed} log_f: {log_f}").unwrap();

}

pub fn write_commands<W: Write>(mut w: W) -> std::io::Result<()>
{
    write!(w, "#")?;
    for arg in std::env::args()
    {
        write!(w, " {arg}")?;
    }
    writeln!(w)
}