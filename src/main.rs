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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Rule {
    code: String,
    name: String
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
        let mut parent_file = tempfile::NamedTempFile::new().unwrap();
        for rule in rules {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            let _ = file.write(rule.code.as_bytes());
            let file_name = file.path();
            let require_statement = format!(
                "require({:?});",
                file_name
            );
            let _ = parent_file.write(require_statement.as_bytes());
        }
        let parent_file_name = parent_file.path();
        let output = Command::new("node")
            .arg(parent_file_name)
            .output()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
        println!("{:?}", String::from_utf8(output.stdout));

        Ok(Response::with((status::Ok, "Ran the rules")))
    }

}
