use clap::Parser;

mod walker;
mod parse;
mod misc;
mod sync_queue;
mod analytical;

mod config;
pub use config::*;
use parse::parse_and_add_to_global;


fn main() {
    
    let opts = Exec::parse();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::ScanMirrorResetting(opts) => walker::execute_mirror(opts),
        Exec::ScanUniMirrorReset(opts) => walker::execute_uni(opts),
        Exec::ScanUniMirror(opts) => 
        {
            walker::execute_uni_only_mirror(opts)
        },
        Exec::ScanUniMirrorAdaptive(opts) => 
        {
            walker::execute_uni_only_mirror_adaptive(opts)
        },
        Exec::MirrorHists(opt) => {
            walker::exec_mirroring_hists(opt.json_path.as_ref())
        },
        Exec::ScanLUniMirror(opt) => {
            walker::execute_pos_scan_uni_only_mirror(opt)
        },
        Exec::ScanLUniMirrorReset(opt) => walker::execute_pos_scan_uni(opt),
        Exec::TestEffRandWalk => walker::test_eff_rand_walker(),
        Exec::EffRandWalkLambda(opt) => {
            let opts: walker::MeasureMfptOpt = parse_and_add_to_global(opt.json);
            walker::eff_measure_mfpt_lambda(
                opts, 
                opt.out.unwrap()
            );
        },
        Exec::EffRandWalkTarget(opt) => {
            let opts: walker::MeasureMfptLOpt = parse_and_add_to_global(opt.json);
            walker::eff_measure_mfpt_target(
                opts, 
                opt.out.unwrap()
            );
        },
        Exec::EffRandWalkBeta(opt) => {
            let mut opts: walker::MeasureMfptBetaOpt = parse_and_add_to_global(opt.json);
            if let Some(a) = opt.a{
                opts.settimgs.a = a;
            }
            walker::eff_measure_mfpt_beta(
                opts, 
                opt.out.unwrap()
            );
        },
        Exec::Eff2RandWalkBeta(opt) => {
            let mut opts: walker::MeasureMfptBetaOpt = parse_and_add_to_global(opt.json);
            if let Some(a) = opt.a{
                opts.settimgs.a = a;
            }
            walker::even_more_efficient_mirror::eff_measure_mfpt_beta(
                opts, 
                opt.out.unwrap()
            );
        },
        Exec::EffBetaCreateJob(opt) => {
            walker::job_creator(opt)
        },
        Exec::Eq23(opt) => {
            analytical::exec_eq_23(opt)
        },
        Exec::Fig4(opt) => {
            analytical::fig4(opt)
        }
    }
    
}