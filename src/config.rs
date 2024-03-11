use std::num::NonZeroUsize;

use clap::Parser;
use camino::Utf8PathBuf;


#[derive(Parser)]
#[command(author, version, about)]
/// ChatGPTs guess of what this program does:
/// Provides various executable commands for statistical analysis and simulations.
///
/// This module defines executable commands that perform statistical analysis,
/// simulations, and data processing tasks. Each command corresponds to a specific
/// operation or functionality.
pub enum Exec{
    /// Performs scanning with resetting.
    ScanResetting(ScanOpts),
    /// Performs scanning with mirror and normal resetting (?).
    ScanMirrorResetting(MirrorScanOpts),
    /// Computes the probability density function for resetting.
    SimpleResetPdf(ResetPdfOpts),
    /// Computes the probability density function for mirror (+?) resetting.
    SimpleMirrorResetPdf(MirrorResetPdfOpts),
    /// Uses the Wang-Landau algorithm to compute the density of states.
    WlResets(WlPdfOpts),
    #[clap(visible_alias="uni")]
    /// Performs scanning with uniform mirror probability distribution. Also uses resets
    ScanUniMirrorReset(UniScanOpts),
    #[clap(visible_alias="unim")]
    /// Performs scanning with uniform mirroring probability distribution. No resets.
    ScanUniMirror(UniScanOpts),
    #[clap(visible_alias="scanl")]
    /// Scan L only mirroring
    ScanLUniMirror(LUniScanOpts),
    #[clap(visible_alias="scanlb")]
    /// Scan L mirroring and resets
    ScanLUniMirrorReset(LUniScanOpts),
    /// Performs scanning with uniform mirroring probability distribution. No resets.
    #[clap(visible_alias="unima")]
    ScanUniMirrorAdaptive(UniScanOpts),
    /// Create histograms only mirroring
    MirrorHists(MirrorHists),
    ///
    TestEffRandWalk,
    #[clap(visible_alias="effrm")]
    EffRandWalkLambda(JsonPathOpt)
}

#[derive(Parser)]
pub struct JsonPathOpt{
    #[arg(long, short, requires("out"))]
    /// Path to json file
    pub json: Option<Utf8PathBuf>,

    #[arg(long, short)]
    /// Name of output file
    pub out: Option<Utf8PathBuf>
}

#[derive(Parser)]
pub struct MirrorHists{
    #[arg(long, short)]
    /// Path to json
    pub json_path: Option<Utf8PathBuf>
}

#[derive(Parser)]
pub struct WlPdfOpts{
    /// Path to the input JSON file. If not provided, a default config file will be printed.
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

    /// Maximum time allowed for the simulation in minutes.
    #[arg(long, short)]
    pub max_time_in_minutes: usize,

    /// Time limit for each sample in the simulation (Maybe, have to check still).
    #[arg(long, short)]
    pub time_limit_of_sample: usize,

    /// Maximum number of resets allowed.
    #[arg(long, short)]
    pub max_resets: u32
}

#[derive(Parser)]
pub struct ResetPdfOpts{
    /// Path to the input JSON file. If not provided, a default config file will be printed.
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

    /// Number of samples to take.
    #[arg(long, short)]
    pub samples: usize,

    /// Number of threads to use.
    #[arg(long, short)]
    pub threads: usize,

    /// Maximum number of resets allowed.
    #[arg(long, short)]
    pub max_resets: u32
}

#[derive(Parser)]
pub struct MirrorResetPdfOpts{
    /// Path to the input JSON file. If not given, it will print out a default config file.
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

    /// Number of samples to take.
    #[arg(long, short)]
    pub samples: usize,

    /// Number of threads to use.
    #[arg(long, short)]
    pub threads: usize,

    /// Maximum number of resets allowed.
    #[arg(long, short)]
    pub max_resets: u32,

    /// Probability of mirror reset.
    #[arg(long, short)]
    pub mirror_prob: f64
}

#[derive(Parser)]
pub struct ScanOpts{
    /// Path to the input JSON file. If not given, it will print out a default config file.
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

    /// Number of samples to take.
    #[arg(long, short)]
    pub samples: usize,

    /// Number of threads to use.
    #[arg(long, short)]
    pub threads: usize,

    /// Start value of lambda.
    #[arg(long, short)]
    pub lambda_start: f64,

    /// End value of lambda.
    #[arg(long, short)]
    pub lambda_end: f64,

    /// Number of samples to take for lambda.
    #[arg(long, short)]
    pub lambda_samples: usize
}

#[derive(Parser)]
pub struct UniScanOpts{
    /// Path to the input JSON file. If not given, it will print out a default config file
    #[arg(long, short, requires("out"))]
    pub json: Option<Utf8PathBuf>,

    #[arg(long, short)]
    /// Number of samples to take.
    pub samples: usize,

    #[arg(long, short)]
    /// Number of threads to use.
    pub threads: NonZeroUsize,

    /// Start value of lambda.
    #[arg(long, short)]
    pub lambda_start: f64,

    /// End value of lambda.
    #[arg(long, short)]
    pub lambda_end: f64,

    /// Number of samples to take for lambda.
    #[arg(long, short)]
    pub lambda_samples: usize,

    /// Path to the output file
    #[arg(long, short)]
    pub out: Option<Utf8PathBuf>
}

#[derive(Parser)]
pub struct LUniScanOpts{
    /// Path to the input JSON file. If not given, it will print out a default config file
    #[arg(long, short, requires("out"))]
    pub json: Option<Utf8PathBuf>,

    #[arg(long, short)]
    /// Number of samples to take.
    pub samples: usize,

    #[arg(long, short)]
    /// Number of threads to use.
    pub threads: NonZeroUsize,

    /// Start value of L.
    #[arg(long, short)]
    pub l_start: f64,

    /// End value of L.
    #[arg(long, short)]
    pub l_end: f64,

    /// Number of samples to take for L.
    #[arg(long, short)]
    pub l_samples: usize,

    /// Path to the output file
    #[arg(long, short)]
    pub out: Option<Utf8PathBuf>
}

#[derive(Parser)]
/// Options for scanning with mirror resetting.
///
/// Defines parameters for scanning with mirror resetting, including JSON file input,
/// sample count, thread count, lambda range, lambda sample count, and mirror reset probability.
pub struct MirrorScanOpts{
    /// Path to the input JSON file. If not provided, a default config file will be printed.
    #[arg(long, short)]
    pub json: Option<Utf8PathBuf>,

    /// Number of samples to take.
    #[arg(long, short)]
    pub samples: usize,

    /// Number of threads to use.
    #[arg(long, short)]
    pub threads: usize,

    /// Start value of lambda.
    #[arg(long, short)]
    pub lambda_start: f64,

    /// End value of lambda.
    #[arg(long, short)]
    pub lambda_end: f64,

    /// Number of samples to take for lambda.
    #[arg(long, short)]
    pub lambda_samples: usize,

    /// Probability of mirror reset.
    #[arg(long, short)]
    pub mirror_prob: f64
}
