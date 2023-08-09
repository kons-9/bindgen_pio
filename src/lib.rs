use bindgen;
use serde_json::{self, Value};
use std::path::PathBuf;
use std::process::Command;

pub trait PioExtention {
    fn pio(self, pio_path: Option<PathBuf>, cpp: bool, target: &str) -> Self;
    fn search_pio(pio_path: Option<PathBuf>) -> Result<Command, std::io::Error> {
        let mut command = if let Some(pio_path) = pio_path {
            Command::new(pio_path)
        } else {
            println!("searching pio in default path");
            Command::new("pio")
        };
        command.status().expect("failed to execute process");
        Ok(command)
    }
}

impl PioExtention for bindgen::Builder {
    fn pio(mut self, pio_path: Option<PathBuf>, cpp: bool, target: &str) -> Self {
        if cpp {
            self = self.clang_arg("-x").clang_arg("c++");
        }
        self = self.clang_arg("-target").clang_arg(target);

        // get pio command
        let mut command = Self::search_pio(pio_path).unwrap();

        // get json
        let output = command
            .arg("project")
            .arg("metadata")
            .arg("--json-output")
            .output()
            .expect("failed to execute process")
            .stdout;
        let stdout = String::from_utf8(output).unwrap();
        let std_json: Value = serde_json::from_str(&stdout).unwrap();

        // get computer name
        let micro_com_name = match &std_json {
            Value::Object(obj) => {
                // get first key
                obj.keys().next().unwrap()
            }
            _ => {
                panic!("invalid platformio.ini");
            }
        };
        println!("target computer's name: {}", micro_com_name);

        // get metadata
        let _build_type = &std_json[micro_com_name]["build_type"];
        let _env_name = &std_json[micro_com_name]["env_name"];
        let libsource_dirs = &std_json[micro_com_name]["libsource_dirs"];
        let defines = &std_json[micro_com_name]["defines"];
        let includes = &std_json[micro_com_name]["includes"];
        let cc_flags = &std_json[micro_com_name]["cc_flags"];
        let cxx_flags = &std_json[micro_com_name]["cxx_flags"];
        // maybe target can be guessed from cc_path or cxx_path
        let _cc_path = &std_json[micro_com_name]["cc_path"];
        let _cxx_path = &std_json[micro_com_name]["cxx_path"];

        // add libsource_dirs
        for libsource_dir in libsource_dirs.as_array().unwrap() {
            self = self.clang_arg(&("-I".to_string() + libsource_dir.as_str().unwrap()));
        }
        // add defines
        for define in defines.as_array().unwrap() {
            self = self.clang_arg(&("-D".to_string() + define.as_str().unwrap()));
        }
        // add includes
        let include_keys = includes.as_object().unwrap().keys();
        for include_key in include_keys {
            let include = &includes[include_key];
            for include_dir in include.as_array().unwrap() {
                self = self.clang_arg(&("-I".to_string() + include_dir.as_str().unwrap()));
            }
        }

        // if cpp {
        //     // add cxx_flags
        //     for cxx_flag in cxx_flags.as_array().unwrap() {
        //         self = self.clang_arg(cxx_flag.as_str().unwrap());
        //     }
        //     // add cxx_path
        // } else {
        //     // add cc_flags
        //     for cc_flag in cc_flags.as_array().unwrap() {
        //         self = self.clang_arg(cc_flag.as_str().unwrap());
        //     }
        //     // add cc_path
        //     // self = self.clang_arg(cc_path.as_str().unwrap());
        // }

        self
    }
}
