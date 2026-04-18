use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let skill_dir = manifest_dir.join(".claude/skills/senko");
    let dod_verifier = manifest_dir.join("src/dod_verifier_agent.md");

    // Collect skill files
    let mut entries: Vec<SkillEntry> = Vec::new();
    scan_dir(&skill_dir, &skill_dir, &mut entries);
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    // Add hardcoded dod_verifier_agent.md
    entries.push(SkillEntry {
        absolute_path: dod_verifier.clone(),
        relative_path: String::new(), // special case
        segments: vec!["agents".to_string(), "dod-verifier.md".to_string()],
        const_name: "DOD_VERIFIER_AGENT_MD".to_string(),
    });

    // Generate code
    let mut code = String::new();

    // Constants
    for entry in &entries {
        let abs = entry.absolute_path.display();
        code.push_str(&format!(
            "pub const {}: &str = include_str!(\"{}\");\n",
            entry.const_name, abs
        ));
    }
    code.push('\n');

    // Aliases for backward compatibility
    code.push_str("pub const SKILL_MD_CONTENT: &str = SKILLS_SENKO_SKILL_MD;\n");
    code.push_str("pub const DOD_VERIFIER_AGENT_CONTENT: &str = DOD_VERIFIER_AGENT_MD;\n");
    code.push('\n');

    // INSTALLABLE_FILES array
    code.push_str("pub const INSTALLABLE_FILES: &[InstallableFile] = &[\n");
    for entry in &entries {
        let segments: Vec<String> = entry
            .segments
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect();
        code.push_str(&format!(
            "    InstallableFile {{ segments: &[{}], content: {} }},\n",
            segments.join(", "),
            entry.const_name
        ));
    }
    code.push_str("];\n");

    fs::write(out_dir.join("installable_files.rs"), code).unwrap();

    // rerun-if-changed
    println!("cargo:rerun-if-changed=.claude/skills/senko");
    println!("cargo:rerun-if-changed=src/dod_verifier_agent.md");
    // Also watch individual files for changes
    for entry in &entries {
        println!("cargo:rerun-if-changed={}", entry.absolute_path.display());
    }
}

struct SkillEntry {
    absolute_path: PathBuf,
    relative_path: String,
    segments: Vec<String>,
    const_name: String,
}

fn scan_dir(base: &Path, dir: &Path, entries: &mut Vec<SkillEntry>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            scan_dir(base, &path, entries);
        } else if path.is_file() {
            let relative = path.strip_prefix(base).unwrap();
            let relative_str = relative.to_str().unwrap().to_string();

            // Segments: ["skills", "senko", ...relative components]
            let mut segments = vec!["skills".to_string(), "senko".to_string()];
            for component in relative.components() {
                segments.push(component.as_os_str().to_str().unwrap().to_string());
            }

            // Const name: SKILLS_SENKO_ + relative path parts joined by _
            let const_name = format!(
                "SKILLS_SENKO_{}",
                relative_str.replace(['/', '-', '.'], "_").to_uppercase()
            );

            entries.push(SkillEntry {
                absolute_path: path,
                relative_path: relative_str,
                segments,
                const_name,
            });
        }
    }
}
