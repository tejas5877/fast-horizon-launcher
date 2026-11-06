use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use serde_json::{json, Value};

use crate::config::LauncherConfig;
use crate::detector::{DetectedJava, DetectedMinecraft};

#[derive(Debug, Clone)]
pub struct HorizonEnvironment {
    pub launcher_exe: PathBuf,
    pub data_dir: PathBuf,
    pub game_home: PathBuf,
}

impl HorizonEnvironment {
    pub fn discover(config: &LauncherConfig) -> Option<Self> {
        let exe_dir = env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let candidates = [
            exe_dir.join("horizon-launcher.exe"),
            exe_dir.join("bin").join("horizon-launcher.exe"),
            PathBuf::from(r"C:\Users\niraj\Downloads\horizon-launcher-windows-x64\horizon-launcher.exe"),
            PathBuf::from(r"C:\Users\niraj\Downloads\Horizon-Offline-0.8.0\horizon-launcher.exe"),
        ];

        let launcher_exe = candidates.into_iter().find(|p| p.is_file())?;

        let data_dir = config.horizon_data_dir.clone().unwrap_or_else(|| {
            if let Ok(appdata) = env::var("APPDATA") {
                PathBuf::from(appdata).join("horizon-launcher")
            } else {
                exe_dir.join("Horizon Offline Data")
            }
        });
        let game_home = data_dir.join("game");

        Some(Self {
            launcher_exe,
            data_dir,
            game_home,
        })
    }

    /// Automatically sets dev_offline=true and configures offline player in settings.json & accounts.json
    pub fn sync_offline_bypass(&self, config: &LauncherConfig, mc: Option<&DetectedMinecraft>) {
        let appdata_dir = env::var("APPDATA")
            .ok()
            .map(|a| PathBuf::from(a).join("horizon-launcher"));

        let target_dirs: Vec<PathBuf> = [Some(self.data_dir.clone()), appdata_dir]
            .into_iter()
            .flatten()
            .collect();

        let mc_path_str = mc.map(|m| m.path.to_string_lossy().into_owned());

        for dir in target_dirs {
            let _ = fs::create_dir_all(&dir);

            // 1. Update settings.json with dev_offline = true
            let settings_file = dir.join("settings.json");
            let mut settings: Value = fs::read_to_string(&settings_file)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_else(|| json!({}));

            if let Some(obj) = settings.as_object_mut() {
                obj.insert("dev_offline".to_string(), json!(true));
                obj.insert("selected".to_string(), json!("nightly-2026-11-05-66f3981"));
                if let Some(ref p) = mc_path_str {
                    obj.insert("minecraft_dir".to_string(), json!(p));
                }
            }
            if let Ok(pretty) = serde_json::to_string_pretty(&settings) {
                let _ = fs::write(settings_file, pretty);
            }

            // 2. Update accounts.json with offline player name
            let accounts_file = dir.join("accounts.json");
            let accounts_json = json!({
                "accounts": [],
                "selected": null,
                "offline": [config.player_name],
                "offline_selected": config.player_name
            });
            if let Ok(pretty) = serde_json::to_string_pretty(&accounts_json) {
                let _ = fs::write(accounts_file, pretty);
            }
        }
    }

    pub fn is_client_installed(&self) -> bool {
        let versions_dir = self.data_dir.join("versions");
        if let Ok(entries) = fs::read_dir(versions_dir) {
            for entry in entries.flatten() {
                if entry.path().join("horizon-client.exe").is_file() {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_client_executable(&self) -> Option<PathBuf> {
        let versions_dir = self.data_dir.join("versions");
        if let Ok(entries) = fs::read_dir(versions_dir) {
            let mut clients: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path().join("horizon-client.exe"))
                .filter(|p| p.is_file())
                .collect();
            clients.sort();
            return clients.pop();
        }
        None
    }

    pub fn are_resources_ready(&self) -> bool {
        let versions_dir = self.data_dir.join("versions");
        if let Ok(entries) = fs::read_dir(versions_dir) {
            for entry in entries.flatten() {
                let resources = entry.path().join(".local").join("horizon-client").join("26.3");
                if resources.join("horizon-resources.json").is_file() 
                    || resources.join("resource-manifest.json").is_file()
                    || resources.join("registries.json").is_file() {
                    return true;
                }
            }
        }
        false
    }

    pub fn prepare_resources(
        &self,
        mc: &DetectedMinecraft,
        java: &DetectedJava,
    ) -> std::io::Result<ExitStatus> {
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.game_home)?;

        let mut cmd = Command::new(&self.launcher_exe);
        cmd.arg("prepare");
        cmd.arg("--minecraft");
        cmd.arg(&mc.path);

        self.apply_environment(&mut cmd, java);

        println!("Running resource preparation via Horizon launcher...");
        println!("Using Minecraft files from: {}", mc.path.display());
        println!("Using Java from: {}", java.java_home.display());

        cmd.status()
    }

    /// Launch the official Horizon GUI window with Microsoft account requirement completely disabled
    pub fn launch_official_gui(
        &self,
        config: &LauncherConfig,
        mc: Option<&DetectedMinecraft>,
        java: &DetectedJava,
    ) -> std::io::Result<ExitStatus> {
        self.sync_offline_bypass(config, mc);

        println!("\x1b[32mLaunching official Horizon Launcher (Offline Mode Enabled)...\x1b[0m");
        let mut cmd = Command::new(&self.launcher_exe);
        self.apply_environment(&mut cmd, java);
        cmd.status()
    }

    pub fn launch_native(
        &self,
        config: &LauncherConfig,
        java: &DetectedJava,
    ) -> std::io::Result<ExitStatus> {
        fs::create_dir_all(&self.game_home)?;

        if let Some(client_exe) = self.get_client_executable() {
            println!("Launching native Horizon client: {}", client_exe.display());
            println!("Player Name: {}", config.player_name);

            let mut cmd = Command::new(&client_exe);
            cmd.current_dir(&self.game_home);
            cmd.arg("--name");
            cmd.arg(&config.player_name);

            self.apply_environment(&mut cmd, java);
            cmd.status()
        } else {
            // Fallback to official launcher play with offline bypass
            println!("Starting Horizon native client via launcher backend...");
            let mut cmd = Command::new(&self.launcher_exe);
            cmd.arg("play");

            self.apply_environment(&mut cmd, java);
            cmd.status()
        }
    }

    fn apply_environment(&self, cmd: &mut Command, java: &DetectedJava) {
        cmd.env("HORIZON_LAUNCHER_HOME", &self.data_dir);
        cmd.env("HORIZON_HOME", &self.game_home);
        cmd.env("HORIZON_LAUNCHER_NO_SELF_UPDATE", "1");
        cmd.env("JAVA_HOME", &java.java_home);

        let current_path = env::var("PATH").unwrap_or_default();
        let java_bin = java.java_home.join("bin");
        let new_path = format!("{};{}", java_bin.display(), current_path);
        cmd.env("PATH", new_path);
    }
}
