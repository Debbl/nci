use std::{fs::File, io::Read, path::Path};
use which::which;

use crate::detect::Package;

pub fn exclude<T: PartialEq + Clone>(arr: &[T], values: &[T]) -> Vec<T> {
    arr.iter()
        .cloned()
        .filter(|item| !values.contains(item))
        .collect()
}

/// Merge `-w value` / `--workspace value` into `-w=value` /
/// `--workspace=value` so npm doesn't treat the flag as a boolean true.
pub fn merge_workspace_flag(args: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let is_ws = arg == "-w" || arg == "--workspace";
        if is_ws && i + 1 < args.len() && !args[i + 1].starts_with('-') {
            out.push(format!("{}={}", arg, args[i + 1]));
            i += 2;
        } else {
            out.push(arg.clone());
            i += 1;
        }
    }
    out
}

pub fn which_cmd(cmd: &str) -> bool {
    let b = which(cmd);
    match b {
        Ok(_) => true,
        Err(_) => false,
    }
}

// https://blog.volta.sh/2020/11/25/command-spotlight-volta-run/
pub fn get_volta_prefix() -> Result<(String, Vec<String>), ()> {
    let volta_prefix = ("volta".to_string(), vec!["run".to_string()]);

    let has_volta_command = which_cmd("volta");

    if has_volta_command {
        Ok(volta_prefix)
    } else {
        Err(())
    }
}

pub fn get_package_json(path: &str) -> Package {
    let path = Path::new(&path);
    if path.exists() && path.is_file() {
        let file = File::open(&path);
        if let Ok(mut file) = file {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return match serde_json::from_str::<Package>(&contents) {
                    Ok(v) => v,
                    Err(_) => Package::default(),
                };
            }
            return Package::default();
        }
        return Package::default();
    }
    Package::default()
}
