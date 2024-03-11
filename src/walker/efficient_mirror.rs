use std::{
    collections::{BTreeMap, BinaryHeap},
    f64::consts::SQRT_2
};
use itertools::*;
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, StandardNormal};
use rand_pcg::Pcg64Mcg;


pub fn test_eff_rand_walker()
{
    let rng = Pcg64Mcg::seed_from_u64(0xff00abc);
    let mut walker = EffRandWalk::new_test(
        4, 
        1.0,
        rng
    );
    for _ in 0..100{
        walker.bisection_step();
    }
    dbg!(walker);
}

#[derive(Debug)]
pub struct EffRandWalk<R>
{
    // Later I should check if HashMap is faster!
    walk: Vec<BTreeMap<OrderedFloat<f64>, Delta>>,
    prob: BinaryHeap<NextProb>,
    fpt: f64,
    target: f64,
    rng: R,
    a: f64
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

impl<R> EffRandWalk<R>
where R: Rng
{
    pub fn new(
        max_depth: usize,
        target: f64,
        rough_step_size: f64,
        lambda_mirror: f64,
        mut rng: R,
        a: f64
    ) -> Self
    {
        let mirror_dist = Exp::new(lambda_mirror)
            .unwrap();
        let mut next_mirror_time = mirror_dist.sample(&mut rng);
        let sqrt_step_size = rough_step_size.sqrt();
        let sq = sqrt_step_size * SQRT_2;
        let mut current_pos = 0.0;
        let mut current_time = 0.0;
        let mut map = BTreeMap::new();
        let fpt = 'outer: loop {
            let div = next_mirror_time / rough_step_size;
            let floored = div.floor();
            let rest = div - floored;
            let steps = floored as usize;
            let time_before_loop = current_time;
            for i in 0..steps{
                let left_time = rough_step_size.mul_add(i as f64, time_before_loop);
                let left_pos = current_pos;
                current_pos += rng.sample::<f64, _>(StandardNormal) * sq;
                let delta = Delta{
                    left_pos,
                    right_pos: current_pos,
                    delta_t: rough_step_size
                };
                map.insert(OrderedFloat(left_time), delta);
                if (left_pos..=current_pos).contains(&target)
                {
                    // TODO in here I could linerarly interpolate to get a more accurate result
                    let fpt = rough_step_size.mul_add((i + 1) as f64, time_before_loop);
                    break 'outer fpt;
                }
            }
            current_time += rough_step_size.mul_add(steps as f64, time_before_loop);
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
            map.insert(OrderedFloat(left_time), delta);
            if (left_pos..=current_pos).contains(&target)
            {
                // TODO in here I could linerarly interpolate to get a more accurate result
                let fpt = current_time;
                break 'outer fpt;
            }
            current_pos *= a;
            next_mirror_time = mirror_dist.sample(&mut rng);
        };
        let heap: BinaryHeap<_> = map
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
            ).collect();
        let mut walk = vec![map];
        walk.extend(
            (1..max_depth).map(|_| BTreeMap::new())
        );
        Self {
            walk, 
            prob: heap, 
            fpt, 
            target, 
            rng, 
            a
        }
    }

    pub fn new_test(max_depth: usize, target: f64, rng: R) -> Self
    {
        let walk = [1.0, 2.0, -1.0, 10.0];
        let queue: BTreeMap<_, _> = walk.iter()
            .tuple_windows()
            .enumerate()
            .map(
                |(i, (left, right))|
                {
                    let time = i as f64;
                    let d = Delta{
                        left_pos: *left,
                        right_pos: *right,
                        delta_t: 1.0
                    };
                    (OrderedFloat(time), d)
                }
            ).collect();
        let heap: BinaryHeap<_> = queue.iter()
            .map(
                |(key, val)|
                {
                    let prob = val.calc_prob(target);
                    NextProb{
                        which_vec: 0,
                        time: *key,
                        prob: OrderedFloat(prob)
                    }
                }
            ).collect();
        let mut walk = vec![queue];
        walk.extend(
            (1..max_depth).map(|_| BTreeMap::new())
        );
        Self{
            walk,
            prob: heap,
            fpt: 3.0,
            target,
            rng,
            a: 1.0
        }
    }

    fn bisection_step(&mut self)
    {
        while let Some(val) = self.prob.pop(){
            let next_vec_id = val.which_vec + 1;
            // TODO: Check if this is correct or if there is a delta t missing for val.time
            if val.time.into_inner() > self.fpt || next_vec_id == self.walk.len(){
                continue;
            }

            let delta = self.walk[val.which_vec].get(&val.time)
                .expect("Has to exist!");

            let (left, right) = delta.bisect();
            // TODO: CHECK if passage occured! and update self.mfpt if nessessary
            let prob_left = left.calc_prob(self.target);
            let prob_right = right.calc_prob(self.target);
            let time_right = val.time + left.delta_t;
            self.walk[next_vec_id].insert(val.time, left);
            self.walk[next_vec_id].insert(time_right, right);
            self.prob.push(
                NextProb { which_vec: next_vec_id, time: val.time, prob: OrderedFloat(prob_left) }
            );
            self.prob.push(
                NextProb { which_vec: next_vec_id, time: time_right, prob: OrderedFloat(prob_right) }
            );

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
        (self.right_pos - target).abs()
    }

    pub fn bisect(&self) -> (Delta, Delta)
    {
        // also needs to be done with proper equation,
        // just a fill in for now to create the husk of the program!
        // TODO
        let mid = self.left_pos + 0.5* (self.right_pos - self.left_pos);
        let delta_t = self.delta_t * 0.5;
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

