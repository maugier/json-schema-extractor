use std::io::{stdin, BufRead};

use clap::Parser;
use serde_json::{Value, json};
use anyhow::{Result, anyhow};

pub const OK: Result<()> = Ok(());

#[derive(Parser)]
#[command(author, about, long_about)]
struct Cmd {
    #[arg(short,long)]
    lines: bool,
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

    let data = stdin().lock();

    let mut errors = 0;
    let mut ok = 0;
    let mut schema = Schema(json!({}));

    let source: Box<dyn Iterator<Item=Result<Value>>> = if cmd.lines {
        Box::new(data.lines().map(|l| {
            Ok(serde_json::from_str(&l?)?)
        }))
    } else {
        let ds = serde_json::Deserializer::from_reader(data)
            .into_iter()
            .map(|r| Ok(r?));
        Box::new(ds)
    };

    for value in source {
        let Ok(value) = value else { errors += 1; continue };
        let Ok(()) = schema.add(&value) else { errors += 1; continue };
        ok += 1;
    }

    eprintln!("{} ok, {} errors", ok, errors);

    println!("{}", serde_json::to_string_pretty(&schema.0)?);

    Ok(())

}
