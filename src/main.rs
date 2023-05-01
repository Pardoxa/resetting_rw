mod walker;
mod parse;

use std::path::PathBuf;

use structopt::StructOpt;


fn main() {
    
    let opts = Exec::from_args();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::SimpleResetPdf(opts) => walker::execute_simple_reset_pdf(opts),
        Exec::WlResets(opts) => walker::execute_wl_reset_pdf(opts)
    }
    
}

#[derive(StructOpt)]
pub enum Exec{
    ScanResetting(ScanOpts),
    SimpleResetPdf(ResetPdfOpts),
    WlResets(WlPdfOpts)
}

#[derive(StructOpt)]
pub struct WlPdfOpts{
    #[structopt(long, short)]
    pub json: Option<PathBuf>,

    #[structopt(long, short)]
    pub max_time_in_minutes: usize,

    #[structopt(long, short)]
    pub time_limit_of_sample: usize,

    #[structopt(long, short)]
    pub max_resets: u32
}

#[derive(StructOpt)]
pub struct ResetPdfOpts{
    #[structopt(long, short)]
    pub json: Option<PathBuf>,

    #[structopt(long, short)]
    pub samples: usize,

    #[structopt(long, short)]
    pub threads: usize,

    #[structopt(long, short)]
    pub max_resets: u32
}

#[derive(StructOpt)]
pub struct ScanOpts{
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