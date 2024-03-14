use clap::Parser;

mod walker;
mod parse;
mod misc;
mod sync_queue;

mod config;
pub use config::*;
use parse::parse_and_add_to_global;


fn main() {
    
    let opts = Exec::parse();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::SimpleResetPdf(opts) => walker::execute_simple_reset_pdf(opts),
        Exec::WlResets(opts) => walker::execute_wl_reset_pdf(opts),
        Exec::ScanMirrorResetting(opts) => walker::execute_mirror(opts),
        Exec::SimpleMirrorResetPdf(opts) => walker::execute_simple_mirror_reset_pdf(opts),
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
        }
    }
    
}