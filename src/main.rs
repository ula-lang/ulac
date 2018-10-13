#![feature(duration_as_u128)]
#[macro_use]
extern crate clap;
extern crate rayon;
extern crate ula;
extern crate walkdir;

use clap::App;
use clap::ArgMatches;
use compiler::Compiler;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::fs;
use std::time::Instant;

mod error;
mod compiler;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
        match command_build(matches) {
            Ok(_) => (),

            Err(error) => {
                println!("{}", error);

                exit(1);
            }
        }
    }
}

fn command_build(matches: &ArgMatches) -> Result<(), String> {
    let start = Instant::now();
    let mut success = true;

    let in_path = Path::new(matches.value_of("in").unwrap());
    let out_path = Path::new(matches.value_of("out").unwrap());

    if !in_path.is_file() && !in_path.exists() {
        return Err(format!("Error: File or directory <{}> does not exist", in_path.to_str().unwrap()));
    }

    if !in_path.is_file() && !out_path.exists() {
        return Err(format!("Error: File or directory <{}> does not exist", out_path.to_str().unwrap()));
    }

    if in_path.is_file() && !out_path.exists() {
        File::create(out_path).unwrap();
    }

    let threads = matches.value_of("threads").map(|t| {
        usize::from_str(t).unwrap()
    });

    let compiler = Compiler::new(threads);

    if in_path.is_file() && !out_path.is_file() {
        println!("{}", out_path.to_str().unwrap());

        Ok(())
    } else if in_path.is_file() && out_path.is_file() {
        match compiler.compile_file(&in_path) {
            Ok(lua) => {
                let mut out_file = File::create(out_path).unwrap();

                write!(out_file, "{}", lua).unwrap();

                let elapsed = start.elapsed();

                println!("Compiled successfully in {}ms", elapsed.as_millis());

                Ok(())
            }

            Err(errors) => {
                success = false;

                println!("{}", errors.join("\r\n"));

                Ok(())
            }
        }
    } else if in_path.is_dir() {
        if !out_path.is_dir() {
            return Err("If the input path is a directory, the output path must also be a directory".to_owned());
        }

        match compiler.compile_dir(&in_path) {
            Ok(results) => {
                for (mut path, result) in results {
                    match result {
                        Ok(lua) => {
                            let out_file_path = {
                                let mut buf = out_path.to_owned();

                                buf.push(path.strip_prefix(&in_path).unwrap());

                                buf.set_extension("lua");

                                buf
                            };

                            let out_file_dir = {
                                let mut buf = out_file_path.clone();

                                buf.pop();

                                buf
                            };

                            fs::create_dir_all(out_file_dir).unwrap();

                            let mut out_file = File::create(&out_file_path).unwrap();

                            write!(out_file, "{}", lua).unwrap();
                        }

                        Err(errors) => {
                            success = false;

                            println!("{}", errors.join("\r\n"))
                        }
                    }
                }

                if success {
                    let elapsed = start.elapsed();

                    println!("Compiled successfully in {}ms", elapsed.as_millis());
                }

                Ok(())
            }

            Err(error) => {
                Err(format!("{}", error))
            }
        }
    } else {
        Err("Unsupported input / output type combination".to_owned())
    }
}
