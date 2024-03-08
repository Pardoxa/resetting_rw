use std::collections::{BTreeMap, BinaryHeap};
use itertools::*;
use ordered_float::OrderedFloat;


pub fn test_eff_rand_walker()
{
    let mut walker = EffRandWalk::new_test(4, 1.0);
    for _ in 0..100{
        walker.bisection_step();
    }
    dbg!(walker);
}

#[derive(Debug)]
pub struct EffRandWalk
{
    // Later I should check if HashMap is faster!
    walk: Vec<BTreeMap<OrderedFloat<f64>, Delta>>,
    prob: BinaryHeap<NextProb>,
    mfpt: f64,
    target: f64
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

impl EffRandWalk{
    pub fn new_test(max_depth: usize, target: f64) -> Self
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
            mfpt: 3.0,
            target
        }
    }

    fn bisection_step(&mut self)
    {
        while let Some(val) = self.prob.pop(){
            let next_vec_id = val.which_vec + 1;
            // TODO: Check if this is correct or if there is a delta t missing for val.time
            if val.time.into_inner() > self.mfpt || next_vec_id == self.walk.len(){
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

