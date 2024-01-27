use std::io::{BufWriter, Write};
use std::sync::RwLock;
use std::fs::File;
use std::path::Path;
use std::fmt::Display;
use serde_json::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
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
    let file = File::create(path)
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
    writeln!(w, "# Git Hash: {} Compile-time: {}", env!("GIT_HASH"), env!("BUILD_TIME_CHRONO"))?;
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