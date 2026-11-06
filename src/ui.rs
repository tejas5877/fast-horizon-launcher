use std::io::{self, Write};
use crate::config::LauncherConfig;
use crate::detector::{DetectedJava, DetectedMinecraft};
use crate::horizon::HorizonEnvironment;

pub struct UI;

impl UI {
    pub fn banner() {
        println!("\x1b[38;2;147;51;234m");
        println!(r#"  ███████╗ █████╗ ███████╗████████╗    ██╗  ██╗ ██████╗ ██████╗ ██╗███████╗ ██████╗ ███╗   ██╗"#);
        println!(r#"  ██╔════╝██╔══██╗██╔════╝╚══██╔══╝    ██║  ██║██╔═══██╗██╔══██╗██║╚══███╔╝██╔═══██╗████╗  ██║"#);
        println!(r#"  █████╗  ███████║███████╗   ██║       ███████║██║   ██║██████╔╝██║  ███╔╝ ██║   ██║██╔██╗ ██║"#);
        println!(r#"  ██╔══╝  ██╔══██║╚════██║   ██║       ██╔══██║██║   ██║██╔══██╗██║ ███╔╝  ██║   ██║██║╚██╗██║"#);
        println!(r#"  ██║     ██║  ██║███████║   ██║       ██║  ██║╚██████╔╝██║  ██║██║███████╗╚██████╔╝██║ ╚████║"#);
        println!(r#"  ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝"#);
        println!("\x1b[0m");
        println!("\x1b[1;36m  Fast Horizon Launcher v1.0.0 \x1b[0m· \x1b[90mOffline-First Rust Minecraft Launcher\x1b[0m\n");
    }

    pub fn print_status(
        config: &LauncherConfig,
        mc: Option<&DetectedMinecraft>,
        java: Option<&DetectedJava>,
        horizon: Option<&HorizonEnvironment>,
    ) {
        println!("\x1b[1mStatus Overview:\x1b[0m");
        println!("  ─────────────────────────────────────────────────────────────");

        println!("  Player Name:       \x1b[32m{}\x1b[0m", config.player_name);

        match mc {
            Some(m) => {
                let badge = if m.has_version_26_3 {
                    "\x1b[32m[26.3 Ready]\x1b[0m"
                } else {
                    "\x1b[33m[26.3 Missing]\x1b[0m"
                };
                println!("  Minecraft Source:  \x1b[36m{}\x1b[0m ({}) {}", m.path.display(), m.source_name, badge);
            }
            None => {
                println!("  Minecraft Source:  \x1b[31mNot Detected\x1b[0m");
            }
        }

        match java {
            Some(j) => {
                println!("  Java Runtime:      \x1b[36m{}\x1b[0m ({}, {})", j.java_executable.display(), j.source_name, j.version_hint);
            }
            None => {
                println!("  Java Runtime:      \x1b[31mNot Found (JDK 25 required)\x1b[0m");
            }
        }

        match horizon {
            Some(h) => {
                let client_status = if h.is_client_installed() {
                    "\x1b[32mInstalled\x1b[0m"
                } else {
                    "\x1b[33mNot Installed\x1b[0m"
                };
                let res_status = if h.are_resources_ready() {
                    "\x1b[32mReady\x1b[0m"
                } else {
                    "\x1b[33mNeeds Preparation\x1b[0m"
                };
                println!("  Horizon Launcher:  \x1b[36m{}\x1b[0m", h.launcher_exe.display());
                println!("  Native Client:     {}", client_status);
                println!("  Game Resources:    {}", res_status);
            }
            None => {
                println!("  Horizon Engine:    \x1b[31mhorizon-launcher.exe not found\x1b[0m");
            }
        }

        println!("  ─────────────────────────────────────────────────────────────\n");
    }

    pub fn read_input(prompt: &str) -> String {
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input.trim().to_string()
    }
}
