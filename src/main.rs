mod walker;
mod parse;

use std::path::PathBuf;

use structopt::StructOpt;


fn main() {
    
    let opts = Opts::from_args();
    walker::execute(opts)
}


#[derive(StructOpt)]
pub struct Opts{
    #[structopt(long, short)]
    pub json: Option<PathBuf>,

    #[structopt(long, short)]
    pub samples: usize,

    #[structopt(long, short)]
    pub threads: usize,

    #[structopt(long, short)]
    pub lambda_start: f64,

    #[structopt(long, short)]
    pub lambda_end: f64,

    #[structopt(long, short)]
    pub lambda_samples: usize
}