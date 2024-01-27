mod walker;
mod parse;
mod misc;

use clap::Parser;
use std::path::PathBuf;



fn main() {
    
    let opts = Exec::parse();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::SimpleResetPdf(opts) => walker::execute_simple_reset_pdf(opts),
        Exec::WlResets(opts) => walker::execute_wl_reset_pdf(opts),
        Exec::ScanMirrorResetting(opts) => walker::execute_mirror(opts),
        Exec::SimpleMirrorResetPdf(opts) => walker::execute_simple_mirror_reset_pdf(opts),
        Exec::ScanUniMirrorReset(opts) => walker::execute_uni(opts)
    }
    
}

#[derive(Parser)]
#[command(author, version, about)]
pub enum Exec{
    ScanResetting(ScanOpts),
    ScanMirrorResetting(MirrorScanOpts),
    SimpleResetPdf(ResetPdfOpts),
    SimpleMirrorResetPdf(MirrorResetPdfOpts),
    WlResets(WlPdfOpts),
    #[clap(visible_alias="uni")]
    ScanUniMirrorReset(UniScanOpts)
}

#[derive(Parser)]
pub struct WlPdfOpts{
    #[arg(long, short)]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub max_time_in_minutes: usize,

    #[arg(long, short)]
    pub time_limit_of_sample: usize,

    #[arg(long, short)]
    pub max_resets: u32
}

#[derive(Parser)]
pub struct ResetPdfOpts{
    #[arg(long, short)]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub samples: usize,

    #[arg(long, short)]
    pub threads: usize,

    #[arg(long, short)]
    pub max_resets: u32
}

#[derive(Parser)]
pub struct MirrorResetPdfOpts{
    #[arg(long, short)]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub samples: usize,

    #[arg(long, short)]
    pub threads: usize,

    #[arg(long, short)]
    pub max_resets: u32,

    #[arg(long, short)]
    pub mirror_prob: f64
}

#[derive(Parser)]
pub struct ScanOpts{
    #[arg(long, short)]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub samples: usize,

    #[arg(long, short)]
    pub threads: usize,

    #[arg(long, short)]
    pub lambda_start: f64,

    #[arg(long, short)]
    pub lambda_end: f64,

    #[arg(long, short)]
    pub lambda_samples: usize
}

#[derive(Parser)]
pub struct UniScanOpts{
    #[arg(long, short, requires("out"))]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub samples: usize,

    #[arg(long, short)]
    pub threads: usize,

    #[arg(long, short)]
    pub lambda_start: f64,

    #[arg(long, short)]
    pub lambda_end: f64,

    #[arg(long, short)]
    pub lambda_samples: usize,

    #[arg(long, short)]
    pub out: Option<PathBuf>
}

#[derive(Parser)]
pub struct MirrorScanOpts{
    #[arg(long, short)]
    pub json: Option<PathBuf>,

    #[arg(long, short)]
    pub samples: usize,

    #[arg(long, short)]
    pub threads: usize,

    #[arg(long, short)]
    pub lambda_start: f64,

    #[arg(long, short)]
    pub lambda_end: f64,

    #[arg(long, short)]
    pub lambda_samples: usize,

    #[arg(long, short)]
    pub mirror_prob: f64
}