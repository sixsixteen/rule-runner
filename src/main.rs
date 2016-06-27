#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;
extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde_json;
extern crate tempfile;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use std::io::Write;
use std::thread;
use std::fs::File;
use std::process::Command;
use std::collections::HashMap;

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize,Hash)]
struct Rule {
    code: String,
    name: String
}

fn create_temp_files(x: usize) -> Vec<tempfile::NamedTempFile> {
    let mut temp_files = Vec::new();
    for _ in 0..x {
        let file = tempfile::NamedTempFile::new().unwrap();
        temp_files.push(file);
    }
    return temp_files;
}

fn main() {

    let mut router = Router::new();

    // Define routes and corresponding handlers
    router.post("/rules", run_rules);

    let router_opt = Iron::new(router).http("localhost:5511");
    if router_opt.is_err() {
        std::process::exit(0);
    }
    router_opt.unwrap();

    fn run_rules(req: &mut Request) -> IronResult<Response> {
        let mut body_str = String::new();
        let _ = req.body.read_to_string(&mut body_str);
        let rules: Vec<Rule> = serde_json::from_str(&body_str).unwrap();
        let mut code_files = create_temp_files(rules.len());
        let mut output_files = create_temp_files(rules.len());

        let mut rule_to_output = HashMap::new();
        let mut root_file = tempfile::NamedTempFile::new().unwrap();
        let fs_require = "var fs = require('fs');\n".to_string();
        let _ = root_file.write(fs_require.as_bytes());

        for (i, rule) in rules.iter().enumerate() {
            let ref mut code_file = code_files[i];
            let _ = code_file.write(rule.code.as_bytes());
            let ref mut output_file = output_files[i];
            let output_file_name = output_file.path();
            let code_file_name = code_file.path();
            let require_statement = format!(
                "fs.writeFile({:?}, JSON.stringify(require({:?})), function(){{}});",
                output_file_name,
                code_file_name
            );
            let _ = root_file.write(require_statement.as_bytes());
            rule_to_output.insert(rule, output_file_name.to_owned());
        }
        let root_file_name = root_file.path();
        let _ = Command::new("node")
            .arg(root_file_name)
            .output()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

        // super-hack incoming
        let mut return_json = String::new();
        let mut json_outputs = Vec::new();
        for (rule, output_file) in &rule_to_output {
            let mut f = File::open(output_file).unwrap();
            let mut s = String::new();
            let _ = f.read_to_string(&mut s);
            json_outputs.push(
                format!(
                    "{{\"name\": {}, \"result\": {}}}",
                    rule.name,
                    s
                    )
                );
        }
        let elements = json_outputs.join(",");
        return_json = format!("[{}]", elements);

        Ok(Response::with((status::Ok, return_json)))
    }

}
