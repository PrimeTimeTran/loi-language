use std::{fs, path::PathBuf};

use crate::daemon::initialize::loid_dir;

fn explain_json_path() -> PathBuf {
    loid_dir().join("daemon").join("data").join("explain.json")
}

// fn explain_md_path() -> PathBuf {
//     loid_dir().join("explain.md")
// }
fn explain_md_path() -> PathBuf {
    std::env::current_dir().unwrap().join("explain.md")
}

pub fn generate_explain_doc() -> std::io::Result<()> {
    let path = explain_json_path();

    if !path.exists() {
        return Ok(()); // nothing to render yet
    }

    let raw = fs::read_to_string(path)?;
    let json: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));

    let md = render_explain(&json);

    fs::write(explain_md_path(), md)?;

    Ok(())
}
/// Converts explain.json → markdown
fn render_explain(json: &serde_json::Value) -> String {
    let mut out = String::new();

    out.push_str("# Loid Explain\n\n");

    // -------------------------
    // SUMMARY
    // -------------------------
    if let Some(summary) = json.get("summary") {
        out.push_str("## Summary\n\n");
        out.push_str(summary.as_str().unwrap_or("N/A"));
        out.push_str("\n\n");
    }

    // -------------------------
    // MANIFEST SECTION
    // -------------------------
    if let Some(manifest) = json.get("manifest") {
        out.push_str("## Manifest (Root Config)\n\n");

        if let Some(obj) = manifest.as_object() {
            for (k, v) in obj {
                out.push_str(&format!("- **{}**: {}\n", k, v));
            }
        }

        out.push_str("\n");
    }

    // -------------------------
    // SYMBOLS SECTION
    // -------------------------
    if let Some(symbols) = json.get("symbols") {
        out.push_str("## Symbols (Navigation Roots)\n\n");

        if let Some(arr) = symbols.as_array() {
            for s in arr {
                let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                let path = s.get("path").and_then(|v| v.as_str()).unwrap_or("");

                out.push_str(&format!("- `{}` → `{}`\n", id, path));
            }
        }

        out.push_str("\n");
    }

    // -------------------------
    // STATE SECTION
    // -------------------------
    if let Some(state) = json.get("state") {
        out.push_str("## Runtime State\n\n");

        if let Some(obj) = state.as_object() {
            for (k, v) in obj {
                out.push_str(&format!("- **{}**: {}\n", k, v));
            }
        }

        out.push_str("\n");
    }

    out
}

pub fn generate_runtime_views() {
    generate_explain_doc();
}
