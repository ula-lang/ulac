use rayon::{ThreadPool, ThreadPoolBuilder};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use ula::compilation;
use walkdir::WalkDir;

pub struct Compiler {
    thread_pool: ThreadPool
}


impl Compiler {
    pub fn new(num_threads: Option<usize>) -> Self {
        let mut thread_pool_builder = ThreadPoolBuilder::new();

        if let Some(num_threads) = num_threads {
            thread_pool_builder = thread_pool_builder.num_threads(num_threads);
        }

        Self {
            thread_pool: thread_pool_builder.build().unwrap()
        }
    }

    pub fn compile_dir<P>(&self, dir: P) -> Result<HashMap<PathBuf, Result<String, Vec<String>>>, Error>
        where P: AsRef<Path> {
        let compiled_map = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.file_name().to_str().unwrap().ends_with(".ula"))
            .map(|e| {
                let path = e.path().to_owned();

                let compile_result = self.compile_file(&path);

                (path, compile_result)
            })
            .collect();

        Ok(compiled_map)
    }

    pub fn compile_file<P>(&self, src: P) -> Result<String, Vec<String>> where P: AsRef<Path> {
        let src_code = {
            let mut buf = String::new();
            let mut file = File::open(&src).unwrap();

            file.read_to_string(&mut buf).unwrap();

            buf
        };

        match compilation::compile_str(&src_code) {
            Ok(lua) => Ok(lua),

            Err(mut errors) => {
                // Append file name to errors
                for error in errors.iter_mut() {
                    error.push_str(&format!(" in <{}>", src.as_ref().to_str().unwrap_or("unknown")))
                }

                Err(errors)
            }
        }
    }
}
