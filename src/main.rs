use std::{fs::File, io::{stdin, BufReader, BufRead}};

use clap::Parser;
use serde_json::{Value, json};
use anyhow::{Result, anyhow};

pub const OK: Result<()> = Ok(());

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cmd {
    file: Option<String>
}

struct Schema(Value);

impl Schema {

    fn add(&mut self, value: &Value) -> Result<()> {
        Self::add_sch(&mut self.0, value)
    }

    fn add_sch(sch: &mut Value, value: &Value) -> Result<()> {
        match value {
            Value::Object(obj) => {
                let sch = sch.as_object_mut().ok_or_else(|| anyhow!("bad schema"))?;
                for (k,v) in obj.iter() {
                    let sub = sch.entry(k).or_insert(json!({}));
                    Self::add_sch(sub, v)?
                }
                OK
            },
            Value::Bool(_) => OK,
            Value::Number(_) => OK,
            Value::String(_) => OK,
            Value::Array(a) => {
                for v in a {
                    Self::add_sch(sch, v)?;
                }
                OK
            },
            _ => OK,
        }
    }
}

fn main() -> Result<()> {
    let cmd = Cmd::parse();

    let filename = cmd.file.as_ref()
        .filter(|&f| f == "-");

    let file: Box<dyn BufRead> = if let Some(filename) = filename {
        let h = File::open(filename)?;
        Box::new(BufReader::new(h))
    } else {
        Box::new(stdin().lock())
    };

    let mut errors = 0;
    let mut ok = 0;
    let mut schema = Schema(json!({}));

    for line in file.lines() {
        let line = line?;
        let Ok(value) = serde_json::from_str(&line) else { errors += 1; continue };
        schema.add(&value)?;
        ok += 1;
    }

    eprintln!("{} ok, {} errors", ok, errors);

    println!("{}", schema.0);

    Ok(())

}
