use std::{fs::File, io::Read, path::Path};
use which::which;

use crate::detect::Package;

pub fn exclude<T: PartialEq + Clone>(arr: &[T], values: &[T]) -> Vec<T> {
    arr.iter().filter(|&item| !values.contains(item)).cloned()
        .collect()
}

/// True when the terminal probably understands OSC 8 hyperlinks. Exposed for
/// callers that want to bake their own link rendering.
pub fn supports_hyperlink() -> bool {
    let force = std::env::var("FORCE_HYPERLINK").is_ok();
    let disable = std::env::var("NO_HYPERLINK").is_ok();
    let is_tty = console::Term::stdout().features().is_attended();
    force || (is_tty && !disable)
}

/// OSC 8 hyperlink. Terminals that understand it render `text` as a
/// clickable link to `url`; everything else falls back to `text (url)`.
/// Honours `NO_HYPERLINK` (disable) and `FORCE_HYPERLINK` (always on).
pub fn terminal_link(text: &str, url: &str) -> String {
    let force = std::env::var("FORCE_HYPERLINK").is_ok();
    let disable = std::env::var("NO_HYPERLINK").is_ok();
    let is_tty = console::Term::stdout().features().is_attended();
    let supported = force || (is_tty && !disable);
    if supported {
        format!("\u{1b}]8;;{}\u{7}{}\u{1b}]8;;\u{7}", url, text)
    } else {
        format!("{} ({})", text, url)
    }
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
    b.is_ok()
}

/// When Volta is on PATH, every package-manager invocation gets prefixed
/// with `volta run` so the right toolchain version is picked.
/// See <https://blog.volta.sh/2020/11/25/command-spotlight-volta-run/>.
pub fn get_volta_prefix() -> Option<(String, Vec<String>)> {
    if which_cmd("volta") {
        Some(("volta".to_string(), vec!["run".to_string()]))
    } else {
        None
    }
}

pub fn get_package_json(path: &str) -> Package {
    let path = Path::new(&path);
    if path.exists() && path.is_file() {
        let file = File::open(path);
        if let Ok(mut file) = file {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return serde_json::from_str::<Package>(&contents).unwrap_or_default();
            }
            return Package::default();
        }
        return Package::default();
    }
    Package::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_link_force_emits_osc8() {
        // FORCE_HYPERLINK bypasses TTY detection — useful for tests.
        std::env::set_var("FORCE_HYPERLINK", "1");
        std::env::remove_var("NO_HYPERLINK");
        let out = terminal_link("npm", "https://npmjs.com");
        // OSC 8 escape: ESC ] 8 ; ; <url> BEL <text> ESC ] 8 ; ; BEL
        assert!(out.contains("\u{1b}]8;;https://npmjs.com\u{7}"));
        assert!(out.ends_with("\u{1b}]8;;\u{7}"));
        assert!(out.contains("npm"));
        std::env::remove_var("FORCE_HYPERLINK");
    }

    #[test]
    fn terminal_link_fallback_when_no_tty() {
        // Cargo tests don't have a TTY by default — and NO_HYPERLINK overrides
        // both ways. With NO_HYPERLINK set, we always fall back.
        std::env::set_var("NO_HYPERLINK", "1");
        std::env::remove_var("FORCE_HYPERLINK");
        let out = terminal_link("npm", "https://npmjs.com");
        assert_eq!(out, "npm (https://npmjs.com)");
        std::env::remove_var("NO_HYPERLINK");
    }
}
