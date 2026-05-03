use habu::{parse::parse_program_from_source, run::run};
use std::{env, fs::read_to_string};

fn main() {
    let path = env::args().skip(1).next();
    if let Some(path) = path {
        let source_contents = read_to_string(path).unwrap();
        match parse_program_from_source(&source_contents) {
            Ok(program) => {
                run(program).unwrap();
            }
            Err(err) => {
                println!("{err}");
            }
        }
    } else {
        println!("[ERR!] You must specify a source file to run.");
    }
}
