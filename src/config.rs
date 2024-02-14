use clap::Parser;
use camino::Utf8PathBuf;


#[derive(Parser)]
#[command(author, version, about)]
pub enum Exec{
    ScanResetting(ScanOpts),
    ScanMirrorResetting(MirrorScanOpts),
    SimpleResetPdf(ResetPdfOpts),
    SimpleMirrorResetPdf(MirrorResetPdfOpts),
    WlResets(WlPdfOpts),
    #[clap(visible_alias="uni")]
    ScanUniMirrorReset(UniScanOpts),
    #[clap(visible_alias="unim")]
    /// Scan mirror lambda, use only mirrors
    ScanUniMirror(UniScanOpts),
    /// Create histograms only mirroring
    MirrorHists(MirrorHists)
}

#[derive(Parser)]
pub struct MirrorHists{
    #[arg(long, short)]
    /// Path to json
    pub json_path: Option<Utf8PathBuf>
}

#[derive(Parser)]
pub struct WlPdfOpts{
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

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
    pub json: Option<Utf8PathBuf>,

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
    pub json: Option<Utf8PathBuf>,

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
    pub json: Option<Utf8PathBuf>,

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
    pub json: Option<Utf8PathBuf>,

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
    pub out: Option<Utf8PathBuf>
}

#[derive(Parser)]
pub struct MirrorScanOpts{
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

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
