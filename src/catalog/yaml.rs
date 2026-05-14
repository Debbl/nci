//! Surgical edits to `pnpm-workspace.yaml`: insert a new package entry into
//! a catalog block while leaving everything else (comments, key ordering,
//! blank lines, trailing newline) alone.
//!
//! The strategy is line-based:
//! 1. Find the matching top-level block (`catalog:` or `catalogs:`).
//! 2. Within `catalogs:`, find the named child block.
//! 3. Detect the indent used by existing children; fall back to 2 / 4 spaces.
//! 4. Insert `<indent><pkg>: <version>` right after the last non-blank line
//!    of that block.
//!
//! When a section doesn't exist yet, it's appended at the end of the file.

/// Insert `<pkg>: <version>` into the catalog named `catalog_name` in `content`
/// and return the new file body. `catalog_name == "default"` targets the
/// top-level `catalog:` block; anything else targets `catalogs.<name>`.
pub fn insert_catalog_entry(
    content: &str,
    catalog_name: &str,
    pkg: &str,
    version: &str,
) -> String {
    let mut lines: Vec<String> = content.split('\n').map(String::from).collect();
    let trailing_newline = content.ends_with('\n');
    // `split` on a trailing '\n' yields an extra empty string; pop it so we
    // edit the real content, then restore it before returning.
    if trailing_newline {
        lines.pop();
    }

    if catalog_name == "default" {
        insert_default(&mut lines, pkg, version);
    } else {
        insert_named(&mut lines, catalog_name, pkg, version);
    }

    let mut out = lines.join("\n");
    if trailing_newline || !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Find a top-level key line (`key:` at indent 0) in `lines` and return its
/// index. Quoted keys and trailing comments are tolerated.
fn find_top_level_block(lines: &[String], key: &str) -> Option<usize> {
    let needle = format!("{}:", key);
    lines.iter().position(|l| {
        if indent_of(l) != 0 {
            return false;
        }
        let trimmed = l.trim_end();
        trimmed == needle || trimmed.starts_with(&format!("{} ", needle))
            || trimmed.starts_with(&format!("{}#", needle))
    })
}

/// Find a child key inside a parent block (`<indent><name>:`).
fn find_child(lines: &[String], parent_start: usize, name: &str) -> Option<(usize, usize)> {
    let end = block_end(lines, parent_start);
    let needle = format!("{}:", name);
    for i in (parent_start + 1)..end {
        let line = &lines[i];
        let indent = indent_of(line);
        if indent == 0 {
            return None;
        }
        let trimmed = line.trim_start();
        let trimmed = trimmed.trim_end();
        if trimmed == needle || trimmed.starts_with(&format!("{} ", needle)) {
            return Some((i, indent));
        }
    }
    None
}

/// Find the line index where the block starting at `start` ends. The block
/// ends at the first non-blank line whose indent is ≤ the parent's indent.
fn block_end(lines: &[String], start: usize) -> usize {
    let parent_indent = indent_of(&lines[start]);
    for i in (start + 1)..lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }
        if indent_of(line) <= parent_indent {
            return i;
        }
    }
    lines.len()
}

/// Inspect children of `parent_start` for their indent. Returns the first
/// indent value found, or `default` if the block is empty.
fn detect_child_indent(lines: &[String], parent_start: usize, default: usize) -> usize {
    let end = block_end(lines, parent_start);
    for i in (parent_start + 1)..end {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }
        return indent_of(line);
    }
    default
}

/// Index just past the last non-blank line of the block. Insertion before
/// this position keeps the new entry adjacent to existing entries instead of
/// after a trailing blank line.
fn insert_position(lines: &[String], parent_start: usize) -> usize {
    let end = block_end(lines, parent_start);
    for i in (parent_start + 1..end).rev() {
        if !lines[i].trim().is_empty() {
            return i + 1;
        }
    }
    parent_start + 1
}

fn insert_default(lines: &mut Vec<String>, pkg: &str, version: &str) {
    if let Some(start) = find_top_level_block(lines, "catalog") {
        let indent = detect_child_indent(lines, start, 2);
        let pos = insert_position(lines, start);
        lines.insert(pos, format!("{}{}: {}", " ".repeat(indent), pkg, version));
    } else {
        ensure_trailing_blank(lines);
        lines.push("catalog:".to_string());
        lines.push(format!("  {}: {}", pkg, version));
    }
}

fn insert_named(lines: &mut Vec<String>, name: &str, pkg: &str, version: &str) {
    if let Some(catalogs_start) = find_top_level_block(lines, "catalogs") {
        if let Some((child_start, _child_indent)) = find_child(lines, catalogs_start, name) {
            let inner_indent = detect_child_indent(lines, child_start, 4);
            let pos = insert_position(lines, child_start);
            lines.insert(pos, format!("{}{}: {}", " ".repeat(inner_indent), pkg, version));
        } else {
            // Existing `catalogs:` block, but no entry for `name` yet.
            let pos = insert_position(lines, catalogs_start);
            lines.insert(pos, format!("    {}: {}", pkg, version));
            lines.insert(pos, format!("  {}:", name));
        }
    } else {
        ensure_trailing_blank(lines);
        lines.push("catalogs:".to_string());
        lines.push(format!("  {}:", name));
        lines.push(format!("    {}: {}", pkg, version));
    }
}

fn ensure_trailing_blank(lines: &mut Vec<String>) {
    if let Some(last) = lines.last() {
        if !last.is_empty() {
            lines.push(String::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_into_existing_default_catalog() {
        let input = "catalog:\n  react: ^18.0.0\n";
        let out = insert_catalog_entry(input, "default", "lodash", "^4.17.21");
        assert_eq!(
            out,
            "catalog:\n  react: ^18.0.0\n  lodash: ^4.17.21\n"
        );
    }

    #[test]
    fn insert_default_when_section_missing() {
        let input = "packages:\n  - 'packages/*'\n";
        let out = insert_catalog_entry(input, "default", "react", "^18.0.0");
        assert_eq!(
            out,
            "packages:\n  - 'packages/*'\n\ncatalog:\n  react: ^18.0.0\n"
        );
    }

    #[test]
    fn insert_into_existing_named_catalog() {
        let input = "catalogs:\n  prod:\n    react: ^18.0.0\n";
        let out = insert_catalog_entry(input, "prod", "lodash", "^4.17.21");
        assert_eq!(
            out,
            "catalogs:\n  prod:\n    react: ^18.0.0\n    lodash: ^4.17.21\n"
        );
    }

    #[test]
    fn add_named_catalog_under_existing_catalogs() {
        let input = "catalogs:\n  prod:\n    react: ^18.0.0\n";
        let out = insert_catalog_entry(input, "dev", "typescript", "^5.0.0");
        // The new "dev:" + entry get inserted at the end of the `catalogs:`
        // block, adjacent to prod's last line.
        let expected = "catalogs:\n  prod:\n    react: ^18.0.0\n  dev:\n    typescript: ^5.0.0\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn create_catalogs_section_when_missing() {
        let input = "packages:\n  - 'packages/*'\n";
        let out = insert_catalog_entry(input, "prod", "react", "^18.0.0");
        assert_eq!(
            out,
            "packages:\n  - 'packages/*'\n\ncatalogs:\n  prod:\n    react: ^18.0.0\n"
        );
    }

    #[test]
    fn preserves_comments_and_blank_lines() {
        let input = "# my workspace\n\ncatalog:\n  # frontend\n  react: ^18.0.0\n  vue: ^3.0.0\n\nnames:\n  - a\n";
        let out = insert_catalog_entry(input, "default", "lodash", "^4.17.21");
        // The new entry slots in right after `vue:` — before the blank line
        // that separates `catalog:` from `names:`.
        let expected = "# my workspace\n\ncatalog:\n  # frontend\n  react: ^18.0.0\n  vue: ^3.0.0\n  lodash: ^4.17.21\n\nnames:\n  - a\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn respects_existing_4space_indent() {
        let input = "catalog:\n    react: ^18.0.0\n";
        let out = insert_catalog_entry(input, "default", "lodash", "^4.0.0");
        assert_eq!(out, "catalog:\n    react: ^18.0.0\n    lodash: ^4.0.0\n");
    }
}
