mod walker;
mod parse;
mod misc;

use std::path::PathBuf;

use structopt::StructOpt;


fn main() {
    
    let opts = Exec::from_args();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::SimpleResetPdf(opts) => walker::execute_simple_reset_pdf(opts),
        Exec::WlResets(opts) => walker::execute_wl_reset_pdf(opts),
        Exec::ScanMirrorResetting(opts) => walker::execute_mirror(opts),
        Exec::SimpleMirrorResetPdf(opts) => walker::execute_simple_mirror_reset_pdf(opts)
    }
    
}

#[derive(StructOpt)]
pub enum Exec{
    ScanResetting(ScanOpts),
    ScanMirrorResetting(MirrorScanOpts),
    SimpleResetPdf(ResetPdfOpts),
    SimpleMirrorResetPdf(MirrorResetPdfOpts),
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
pub struct MirrorResetPdfOpts{
    #[structopt(long, short)]
    pub json: Option<PathBuf>,

    #[structopt(long, short)]
    pub samples: usize,

    #[structopt(long, short)]
    pub threads: usize,

    #[structopt(long, short)]
    pub max_resets: u32,

    #[structopt(long, short)]
    pub mirror_prob: f64
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

#[derive(StructOpt)]
pub struct MirrorScanOpts{
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
    pub lambda_samples: usize,

    #[structopt(long, short)]
    pub mirror_prob: f64
}