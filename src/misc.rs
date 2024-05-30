use std::io::{BufWriter, Write};
use std::sync::RwLock;
use fs_err::File;
use std::path::Path;
use std::fmt::Display;
use std::num::*;
use serde_json::Value;
use num_rational::Rational64;
use num_traits::cast::ToPrimitive;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const BUILD_TIME_CHRONO: &str = env!("BUILD_TIME_CHRONO");

pub static GLOBAL_ADDITIONS: RwLock<Option<String>> = RwLock::new(None);

pub fn write_json<W: Write>(mut writer: W, json: &Value)
{
    write!(writer, "#").unwrap();
    serde_json::to_writer(&mut writer, json).unwrap();
    writeln!(writer).unwrap();
}


pub fn create_buf<P>(path: P) -> BufWriter<File>
where P: AsRef<Path>
{
    let file = File::create(path.as_ref())
        .expect("Unable to create file");
    BufWriter::new(file)
}

pub fn write_commands<W: Write>(mut w: W) -> std::io::Result<()>
{
    write!(w, "#")?;
    for arg in std::env::args()
    {
        write!(w, " {arg}")?;
    }
    writeln!(w)
}

pub fn write_commands_and_version<W: Write>(mut w: W) -> std::io::Result<()>
{
    writeln!(w, "# {VERSION}")?;
    writeln!(w, "# Git Hash: {GIT_HASH} Compile-time: {BUILD_TIME_CHRONO}")?;
    let l = GLOBAL_ADDITIONS.read().unwrap();
    if let Some(add) = l.as_deref(){
        writeln!(w, "# {add}")?;
    }
    drop(l);
    write_commands(w)
}

#[must_use]
pub fn create_buf_with_command_and_version<P>(path: P) -> BufWriter<File>
where P: AsRef<Path>
{
    let mut buf = create_buf(path);
    write_commands_and_version(&mut buf)
        .expect("Unable to write Version and Command in newly created file");
    buf
}

pub fn create_buf_with_command_and_version_and_header<P, S, D>(path: P, header: S) -> BufWriter<File>
where P: AsRef<Path>,
    S: IntoIterator<Item=D>,
    D: Display
{
    let mut buf = create_buf_with_command_and_version(path);
    write_slice_head(&mut buf, header)
        .expect("unable to write header");
    buf
}

pub fn write_slice_head<W, S, D>(mut w: W, slice: S) -> std::io::Result<()>
where W: std::io::Write,
    S: IntoIterator<Item=D>,
    D: Display
{
    write!(w, "#")?;
    for (s, i) in slice.into_iter().zip(1_u16..){
        write!(w, " {s}_{i}")?;
    }
    writeln!(w)
}


pub struct RatioIter{
    start: Rational64,
    end: Rational64,
    // number of samples minus 1
    num_samples_m1: NonZeroI64
}

impl RatioIter{
    pub fn float_iter(&self) -> impl Iterator<Item=f64>
    {
        self.ratio_iter()
            .map(|r| r.to_f64().unwrap())
    }

    pub fn ratio_iter(&self) -> impl Iterator<Item=Rational64>
    {
        let delta = (self.end - self.start) / self.num_samples_m1.get();
        let start = self.start;
        (0..=self.num_samples_m1.get())
            .map(
                move |i| 
                {
                    start + delta * i
                }
            )
    }

    pub fn get_ratio_iter(start: f64, end: f64, num_samples: i64) -> Self{
        let start = Rational64::approximate_float(start).unwrap();
        let end = Rational64::approximate_float(end).unwrap();
        let num_samples_m1 = NonZeroI64::new(num_samples - 1).unwrap();
        RatioIter{
            start,
            end,
            num_samples_m1
        }
    }
}