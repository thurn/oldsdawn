// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generates initialize.rs. This used to be automatic via the 'linkme' crate,
//! but new versions of Rust have caused a bunch of crazy problems with it.

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

fn main() -> Result<()> {
    println!("cargo:warning=Generating initialize.rs");
    let mut functions = HashMap::new();
    for e in WalkDir::new("src") {
        let entry = e?;
        let file_name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            _ => continue,
        }
        .replace(".rs", "");

        if entry.file_type().is_file() {
            functions.insert(file_name, find_functions(entry.path())?);
        }
    }

    let out_path = Path::new("src/initialize.rs");
    if out_path.exists() {
        fs::remove_file(out_path)?;
    }
    let mut file = LineWriter::new(File::create(out_path)?);
    writeln!(file, "//! GENERATED CODE - DO NOT MODIFY\n")?;

    writeln!(file, "use rules::DEFINITIONS;\n")?;
    writeln!(file, "use crate::{{")?;
    let mut modules = functions
        .iter()
        .filter(|(_, list)| !list.is_empty())
        .map(|(module, _)| module.clone())
        .collect::<Vec<_>>();
    modules.sort();
    writeln!(file, "    {},", modules.join(", "))?;
    writeln!(file, "}};")?;

    writeln!(file, "\npub fn run() {{")?;
    for module in &modules {
        if let Some(list) = functions.get(module) {
            for function in list {
                writeln!(file, "    DEFINITIONS.insert({}::{});", module, function)?;
            }
        }
    }
    writeln!(file, "}}")?;

    Ok(())
}

fn find_functions(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut result = vec![];
    let re = Regex::new(r"pub fn (?P<name>\w+)\(\) -> CardDefinition")?;
    for l in BufReader::new(File::open(path)?).lines() {
        let line = l?;
        if let Some(captures) = re.captures(&line) {
            result.push(captures["name"].to_string());
        }
    }
    Ok(result)
}
