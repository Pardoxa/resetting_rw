use clap::Parser;

mod walker;
mod parse;
mod misc;


mod config;
pub use config::*;


fn main() {
    
    let opts = Exec::parse();
    match opts{
        Exec::ScanResetting(opts) => walker::execute(opts),
        Exec::SimpleResetPdf(opts) => walker::execute_simple_reset_pdf(opts),
        Exec::WlResets(opts) => walker::execute_wl_reset_pdf(opts),
        Exec::ScanMirrorResetting(opts) => walker::execute_mirror(opts),
        Exec::SimpleMirrorResetPdf(opts) => walker::execute_simple_mirror_reset_pdf(opts),
        Exec::ScanUniMirrorReset(opts) => walker::execute_uni(opts),
        Exec::ScanUniMirror(opts) => walker::execute_uni_only_mirror(opts),
        Exec::MirrorHists(opt) => {
            walker::exec_mirroring_hists(opt.json_path.as_ref())
        }
    }
    
}
