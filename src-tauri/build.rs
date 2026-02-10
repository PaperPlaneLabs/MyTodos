fn main() {
    // Load .env from project root (parent of src-tauri) for local dev.
    // In CI, these are set as environment variables directly.
    if let Ok(contents) = std::fs::read_to_string("../.env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('\'').trim_matches('"');
                if key == "GOOGLE_CLIENT_ID" || key == "GOOGLE_CLIENT_SECRET" {
                    println!("cargo:rustc-env={}={}", key, value);
                }
            }
        }
    }

    // Re-run if .env changes
    println!("cargo:rerun-if-changed=../.env");

    tauri_build::build()
}
