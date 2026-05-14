//! Write `catalog:` references into `package.json` `dependencies` /
//! `devDependencies` / `peerDependencies`, preserving the file's existing
//! indent and key ordering (with new entries inserted into the sorted set).

use std::{
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use serde_json::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DepType {
    Dependencies,
    DevDependencies,
    PeerDependencies,
}

impl DepType {
    pub fn as_key(self) -> &'static str {
        match self {
            DepType::Dependencies => "dependencies",
            DepType::DevDependencies => "devDependencies",
            DepType::PeerDependencies => "peerDependencies",
        }
    }
}

/// Walk up from `cwd` looking for the nearest `package.json`. Returns the
/// absolute path or `None`.
pub fn find_closest_package_json(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd.to_path_buf();
    loop {
        let candidate = dir.join("package.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Detect the indent prefix used by the file (tabs or N spaces). Falls back
/// to two spaces.
fn detect_indent(content: &str) -> String {
    for line in content.lines() {
        if line.starts_with('\t') {
            return "\t".to_string();
        }
        if line.starts_with(' ') {
            let spaces = line.chars().take_while(|c| *c == ' ').count();
            if spaces > 0 && line.trim_start().starts_with('"') {
                return " ".repeat(spaces);
            }
        }
    }
    "  ".to_string()
}

/// One `(name, ref)` pair to write under the chosen `DepType` block. `ref` is
/// the catalog reference (e.g. `catalog:` or `catalog:prod`).
pub struct Entry<'a> {
    pub name: &'a str,
    pub catalog_ref: &'a str,
}

/// Update `package.json` at `path`: ensure the chosen `dep_type` block
/// exists, insert/overwrite each `entries[i].name` with `entries[i].catalog_ref`,
/// then write back with stable alphabetical key order under that block. The
/// rest of the file is round-tripped through `serde_json` (which preserves
/// top-level key order thanks to `preserve_order`).
pub fn update_package_json_catalog_refs(
    path: &Path,
    entries: &[Entry<'_>],
    dep_type: DepType,
) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    let indent = detect_indent(&content);
    let mut data: Value = serde_json::from_str(&content)?;

    let key = dep_type.as_key();
    let obj = data
        .as_object_mut()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "package.json root is not an object"))?;
    let deps = obj
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let deps_map = deps
        .as_object_mut()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "deps field is not an object"))?;

    for entry in entries {
        deps_map.insert(entry.name.to_string(), Value::String(entry.catalog_ref.to_string()));
    }

    // Sort by key. serde_json's Map preserves insertion order with the
    // `preserve_order` feature, so rebuild the map in sorted order.
    let mut pairs: IndexMap<String, Value> = IndexMap::new();
    let mut keys: Vec<String> = deps_map.keys().cloned().collect();
    keys.sort();
    for k in keys {
        if let Some(v) = deps_map.remove(&k) {
            pairs.insert(k, v);
        }
    }
    let mut sorted = serde_json::Map::new();
    for (k, v) in pairs {
        sorted.insert(k, v);
    }
    obj.insert(key.to_string(), Value::Object(sorted));

    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    data.serialize(&mut ser).map_err(std::io::Error::other)?;
    buf.push(b'\n');
    fs::write(path, buf)?;
    Ok(())
}

// `Value::serialize` requires `serde::Serialize` in scope.
use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_pkg(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join("package.json");
        fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn writes_into_existing_dependencies() {
        let dir = TempDir::new().unwrap();
        let p = write_pkg(
            dir.path(),
            r#"{
  "name": "demo",
  "dependencies": {
    "vue": "^3.0.0"
  }
}
"#,
        );
        update_package_json_catalog_refs(
            &p,
            &[Entry {
                name: "react",
                catalog_ref: "catalog:",
            }],
            DepType::Dependencies,
        )
        .unwrap();
        let got = fs::read_to_string(&p).unwrap();
        // Keys sorted alphabetically; indent preserved (2 spaces).
        let expected = "{\n  \"name\": \"demo\",\n  \"dependencies\": {\n    \"react\": \"catalog:\",\n    \"vue\": \"^3.0.0\"\n  }\n}\n";
        assert_eq!(got, expected);
    }

    #[test]
    fn creates_block_if_missing() {
        let dir = TempDir::new().unwrap();
        let p = write_pkg(dir.path(), "{\n  \"name\": \"demo\"\n}\n");
        update_package_json_catalog_refs(
            &p,
            &[Entry {
                name: "typescript",
                catalog_ref: "catalog:dev",
            }],
            DepType::DevDependencies,
        )
        .unwrap();
        let got = fs::read_to_string(&p).unwrap();
        assert!(got.contains("\"devDependencies\""));
        assert!(got.contains("\"typescript\": \"catalog:dev\""));
    }

    #[test]
    fn preserves_4_space_indent() {
        let dir = TempDir::new().unwrap();
        let p = write_pkg(
            dir.path(),
            "{\n    \"name\": \"demo\",\n    \"dependencies\": {\n        \"vue\": \"^3.0.0\"\n    }\n}\n",
        );
        update_package_json_catalog_refs(
            &p,
            &[Entry {
                name: "react",
                catalog_ref: "catalog:",
            }],
            DepType::Dependencies,
        )
        .unwrap();
        let got = fs::read_to_string(&p).unwrap();
        assert!(got.contains("        \"react\": \"catalog:\""), "got:\n{got}");
    }
}
