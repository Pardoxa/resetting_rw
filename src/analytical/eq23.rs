use clap::Parser;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io::Write;
use crate::misc::*;



fn naive_product(a: f64, n: i32) -> Vec<f64>
{
    let mut prod = 1.0;
    let mut v = Vec::with_capacity(n as usize);
    for i in 1..=n{
        let exp = a.powi(-2*i);
        prod *= 1.0 - exp;
        v.push(prod);
    }
    v
}

fn left(a: f64, n: i32, r: f64) -> f64
{
    let products = naive_product(a, n);
    let sum: f64 = products.iter()
        .map(|val| val.recip())
        .sum();

    0.5 * r.sqrt() / (1.0 + sum)
}

#[derive(Clone, Debug, Serialize, Deserialize, Parser)]
pub struct Eq23Opt{
    /// a
    #[arg(long, short)]
    a: f64,
    #[arg(long, short)]
    /// r
    r: f64,
    #[arg(long, short)]
    /// cutoff for the sums, Something like 1000 should be good
    cutoff: i32,
    #[arg(long)]
    /// Left x value
    x_start: f64,
    #[arg(long)]
    ///right x value
    x_end: f64,
    #[arg(long, short)]
    /// Number of samples
    samples: i64,
    /// filename for output
    filename: String
}

pub fn exec_eq_23(opt: Eq23Opt)
{
    let x_arr = RatioIter::get_ratio_iter(opt.x_start, opt.x_end, opt.samples)
        .float_iter()
        .collect_vec();
    write_res(
        &x_arr, 
        opt.a.abs(), 
        opt.cutoff, 
        opt.r, 
        &opt.filename
    )
}

fn write_res(
    x_arr: &[f64], 
    a: f64, 
    cutoff: i32, 
    r: f64,
    filename: &str
)
{
    assert!(
        a >= 0.0,
        "Negative a not allowed here"
    );
    let left = left(a, cutoff, r);

    let products = naive_product(a, cutoff);

    let header = [
        "x",
        "P(x)"
    ];
    let mut buf = create_buf_with_command_and_version_and_header(filename, header);

    let r_root = r.sqrt();
    for x in x_arr{
        let inner_left = (-x.abs() * r_root).exp();
        let mut sum = 0.0;
        for (prod, i) in products.iter().zip(1..){
            let a_term = a.powi(-i);
            let factor = a_term * (-a_term * r_root * x.abs()).exp();
            sum += factor / prod;
        }
        let result = left * (inner_left + sum);
        writeln!(buf, "{x} {result}").unwrap();
    }

}