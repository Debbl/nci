use std::path::Path;

use crate::utils::get_package_json;

/// Bash completion script. Source via `nr --completion-bash >> ~/.bashrc`.
pub const RAW_BASH_COMPLETION_SCRIPT: &str = r#"
###-begin-nr-completion-###

if type complete &>/dev/null; then
  _nr_completion() {
    local words
    local cur
    local cword
    _get_comp_words_by_ref -n =: cur words cword
    IFS=$'\n'
    COMPREPLY=($(COMP_CWORD=$cword COMP_LINE=$cur nr --completion ${words[@]}))
  }
  complete -F _nr_completion nr
fi

###-end-nr-completion-###
"#;

/// Zsh completion script. Source via your `~/.zshrc` or place under
/// `fpath`-loaded directories.
pub const RAW_ZSH_COMPLETION_SCRIPT: &str = r#"
#compdef nr

_nr_completion() {
  local -a completions
  completions=("${(f)$(nr --completion $words[2,-1])}")

  compadd -a completions
}

_nr_completion
"#;

/// Fish completion script. Write to `~/.config/fish/completions/nr.fish`.
pub const RAW_FISH_COMPLETION_SCRIPT: &str = r#"
function _nr_completion
  set -l tokens (commandline -xpc)
  if test (count $tokens) -ge 1
    set tokens $tokens[2..-1]
  end
  nr --completion $tokens 2>/dev/null
end

complete -c nr -f -a '(_nr_completion)' -d 'package.json scripts'
"#;

/// Return script-name suggestions for the prefix being typed, in
/// `package.json` order. `prefix=""` means "show all".
pub fn completion_suggestions(cwd: &Path, prefix: &str) -> Vec<String> {
    let pkg_path = cwd.join("package.json");
    let pkg = get_package_json(&pkg_path.to_string_lossy());
    let scripts = pkg.scripts.unwrap_or_default();

    let keys: Vec<String> = scripts
        .into_iter()
        .filter(|(k, _)| !k.starts_with('?'))
        .map(|(k, _)| k)
        .collect();

    if prefix.is_empty() {
        return keys;
    }
    let lower = prefix.to_lowercase();
    keys.into_iter()
        .filter(|k| k.to_lowercase().contains(&lower))
        .collect()
}
