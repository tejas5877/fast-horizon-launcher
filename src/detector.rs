use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DetectedMinecraft {
    pub path: PathBuf,
    pub source_name: &'static str,
    pub has_version_26_3: bool,
    pub available_versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedJava {
    pub java_home: PathBuf,
    pub java_executable: PathBuf,
    pub source_name: &'static str,
    pub version_hint: String,
}

pub fn detect_minecraft(custom_path: Option<&Path>) -> Option<DetectedMinecraft> {
    if let Some(custom) = custom_path {
        if custom.is_dir() {
            return Some(probe_minecraft_dir(custom, "Custom Configuration"));
        }
    }

    // 1. Probe FastClient (%APPDATA%\.fastclient)
    if let Ok(appdata) = env::var("APPDATA") {
        let fastclient_path = PathBuf::from(&appdata).join(".fastclient");
        if fastclient_path.is_dir() {
            let probe = probe_minecraft_dir(&fastclient_path, "FastClient");
            if probe.has_version_26_3 || !probe.available_versions.is_empty() {
                return Some(probe);
            }
        }
    }

    // 2. Probe Standard Minecraft / TLauncher (%APPDATA%\.minecraft)
    if let Ok(appdata) = env::var("APPDATA") {
        let mc_path = PathBuf::from(&appdata).join(".minecraft");
        if mc_path.is_dir() {
            return Some(probe_minecraft_dir(&mc_path, "Standard Minecraft / TLauncher"));
        }
    }

    // 3. Unix fallback paths (~/.minecraft)
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let home_mc = PathBuf::from(&home).join(".minecraft");
        if home_mc.is_dir() {
            return Some(probe_minecraft_dir(&home_mc, "Home Directory (.minecraft)"));
        }
    }

    None
}

fn probe_minecraft_dir(dir: &Path, source: &'static str) -> DetectedMinecraft {
    let versions_dir = dir.join("versions");
    let mut available_versions = Vec::new();
    let mut has_version_26_3 = false;

    if let Ok(entries) = std::fs::read_dir(versions_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let name_str = name.to_string();
                    if name_str.contains("26.3") {
                        has_version_26_3 = true;
                    }
                    available_versions.push(name_str);
                }
            }
        }
    }

    DetectedMinecraft {
        path: dir.to_path_buf(),
        source_name: source,
        has_version_26_3,
        available_versions,
    }
}

pub fn detect_java(custom_home: Option<&Path>) -> Option<DetectedJava> {
    // 1. Check custom configured path
    if let Some(custom) = custom_home {
        if let Some(detected) = verify_java_home(custom, "User Configuration") {
            return Some(detected);
        }
    }

    // 2. Check FastClient JDK 25 runtime
    if let Ok(appdata) = env::var("APPDATA") {
        let fc_runtime_root = PathBuf::from(&appdata).join("FastClient").join("runtimes");
        let fc_java25 = fc_runtime_root.join("java-25");
        if fc_java25.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&fc_java25) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(detected) = verify_java_home(&entry.path(), "FastClient Bundled Java 25") {
                            return Some(detected);
                        }
                    }
                }
            }
        }

        // Check FastClient JDK 21 fallback
        let fc_java21 = fc_runtime_root.join("java-21");
        if fc_java21.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&fc_java21) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(detected) = verify_java_home(&entry.path(), "FastClient Bundled Java 21") {
                            return Some(detected);
                        }
                    }
                }
            }
        }
    }

    // 3. Check system JAVA_HOME
    if let Ok(jh) = env::var("JAVA_HOME") {
        let jh_path = PathBuf::from(jh);
        if let Some(detected) = verify_java_home(&jh_path, "System JAVA_HOME") {
            return Some(detected);
        }
    }

    // 4. Common standard Windows paths
    let standard_roots = [
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files\Java",
        r"C:\Program Files\Microsoft",
    ];

    for root in standard_roots {
        let path = PathBuf::from(root);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(detected) = verify_java_home(&entry.path(), "Standard Installation") {
                            return Some(detected);
                        }
                    }
                }
            }
        }
    }

    None
}

fn verify_java_home(dir: &Path, source: &'static str) -> Option<DetectedJava> {
    let exe_name = if cfg!(windows) { "java.exe" } else { "java" };
    let candidate_exe = dir.join("bin").join(exe_name);
    if candidate_exe.is_file() {
        let version_hint = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Some(DetectedJava {
            java_home: dir.to_path_buf(),
            java_executable: candidate_exe,
            source_name: source,
            version_hint,
        })
    } else {
        None
    }
}
