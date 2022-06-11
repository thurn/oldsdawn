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

use std::{fs, process};

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let content = fs::read_to_string(path).expect("path contents");
    let doc = roxmltree::Document::parse(&content).expect("parse error");
    let mut error = false;
    for node in doc.descendants().filter(|node| node.has_tag_name("test-case")) {
        let result = node.attribute("result").expect("test result");
        println!(
            "\n{}: {} in {}\n",
            node.attribute("methodname").expect("method name"),
            result,
            node.attribute("duration").expect("test duration")
        );
        if result != "Passed" {
            error = true;
        }
    }

    process::exit(if error { 1 } else { 0 })
}
