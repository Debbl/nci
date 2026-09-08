use std::{fs, path::Path};

use serde_json::Value;

use super::{config_from_fields, package_json::write_json, CatalogConfig};

fn nested(json: &Value) -> bool {
    json.get("workspaces")
        .filter(|w| w.is_object())
        .is_some_and(|w| {
            ["catalog", "catalogs"]
                .iter()
                .any(|key| w.get(key).is_some_and(|v| !v.is_null()))
        })
}

pub fn detect_bun_catalogs(cwd: &Path) -> Option<CatalogConfig> {
    for dir in cwd.ancestors() {
        let path = dir.join("package.json");
        let Some(json) = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            continue;
        };
        if json.get("workspaces").is_none_or(|w| w.is_null()) {
            continue;
        }
        // The closest workspace is the boundary, even if it has no catalogs.
        let fields = if nested(&json) {
            &json["workspaces"]
        } else {
            &json
        };
        return config_from_fields(path, serde_json::from_value(fields.clone()).ok()?);
    }
    None
}

pub(super) fn add_package(
    path: &Path,
    content: &str,
    catalog: &str,
    pkg: &str,
    version: &str,
) -> std::io::Result<()> {
    let mut json: Value = serde_json::from_str(content)?;
    let target = if nested(&json) {
        &mut json["workspaces"]
    } else {
        &mut json
    };
    let entries = if catalog == "default" {
        &mut target["catalog"]
    } else {
        &mut target["catalogs"][catalog]
    };
    if entries.is_null() {
        *entries = serde_json::json!({});
    }
    let map = entries.as_object_mut().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "catalog must be an object")
    })?;
    map.insert(pkg.into(), Value::String(version.into()));
    write_json(path, content, &json)
}
