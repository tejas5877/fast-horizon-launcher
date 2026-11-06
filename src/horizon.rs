use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

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

        // Search for horizon-launcher.exe in:
        // 1. Current directory
        // 2. bin/
        // 3. Downloads / Horizon-Offline-0.8.0 folder if running nearby
        let candidates = [
            exe_dir.join("horizon-launcher.exe"),
            exe_dir.join("bin").join("horizon-launcher.exe"),
            PathBuf::from(r"C:\Users\niraj\Downloads\Horizon-Offline-0.8.0\horizon-launcher.exe"),
        ];

        let launcher_exe = candidates.into_iter().find(|p| p.is_file())?;

        let data_dir = config.horizon_data_dir.clone().unwrap_or_else(|| {
            exe_dir.join("Horizon Offline Data")
        });
        let game_home = data_dir.join("game");

        Some(Self {
            launcher_exe,
            data_dir,
            game_home,
        })
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
                if resources.join("resource-manifest.json").is_file() 
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

    pub fn launch(
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
            // Fallback to official launcher play
            println!("Starting Horizon through launcher launcher...");
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
