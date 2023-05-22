use std::io::Write;
use serde_json::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn write_json<W: Write>(mut writer: W, json: &Value)
{
    write!(writer, "#").unwrap();
    serde_json::to_writer(&mut writer, json).unwrap();
    writeln!(writer).unwrap();
}

pub fn write_commands<W: Write>(mut w: W) -> std::io::Result<()>
{
    writeln!(w, "# v{VERSION}").unwrap();
    write!(w, "#")?;
    for arg in std::env::args()
    {
        write!(w, " {arg}")?;
    }
    writeln!(w)
}