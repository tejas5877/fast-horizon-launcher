mod config;
mod detector;
mod horizon;
mod ui;

use clap::Parser;
use std::path::PathBuf;

use crate::config::LauncherConfig;
use crate::detector::{detect_java, detect_minecraft};
use crate::horizon::HorizonEnvironment;
use crate::ui::UI;

#[derive(Parser, Debug)]
#[command(
    name = "fast-horizon-launcher",
    author = "AN0N",
    version = "1.0.0",
    about = "Lightning-fast, offline-first Rust launcher for Horizon Minecraft with automatic FastClient & Java detection."
)]
struct Args {
    /// Start the Minecraft Horizon client directly
    #[arg(short, long)]
    launch: bool,

    /// Run resource preparation from Minecraft files
    #[arg(short, long)]
    prepare: bool,

    /// Only perform diagnostics check and print status
    #[arg(short, long)]
    check: bool,

    /// Player nickname to use for offline play
    #[arg(short, long)]
    name: Option<String>,

    /// Override path to Minecraft installation (e.g. .fastclient or .minecraft)
    #[arg(short, long)]
    minecraft: Option<PathBuf>,

    /// Override path to Java JDK home directory (JDK 25 required)
    #[arg(short, long)]
    java: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut config = LauncherConfig::load();

    // Apply CLI overrides if provided
    if let Some(ref name) = args.name {
        let _ = config.set_player_name(name);
    }
    if let Some(ref mc) = args.minecraft {
        let _ = config.set_minecraft_dir(Some(mc));
    }
    if let Some(ref java) = args.java {
        let _ = config.set_java_home(Some(java));
    }

    let mc_detected = detect_minecraft(config.minecraft_dir.as_deref());
    let java_detected = detect_java(config.java_home.as_deref());
    let horizon_env = HorizonEnvironment::discover(&config);

    if args.check {
        UI::banner();
        UI::print_status(&config, mc_detected.as_ref(), java_detected.as_ref(), horizon_env.as_ref());
        return;
    }

    if args.prepare {
        UI::banner();
        if let (Some(mc), Some(java), Some(horizon)) = (mc_detected.as_ref(), java_detected.as_ref(), horizon_env.as_ref()) {
            match horizon.prepare_resources(mc, java) {
                Ok(status) if status.success() => {
                    println!("\x1b[32mSuccessfully prepared Minecraft resources!\x1b[0m");
                }
                Ok(status) => {
                    eprintln!("\x1b[31mPreparation process failed with exit code: {:?}\x1b[0m", status.code());
                }
                Err(e) => {
                    eprintln!("\x1b[31mError executing preparation: {}\x1b[0m", e);
                }
            }
        } else {
            eprintln!("\x1b[31mError: Cannot prepare resources. Ensure Minecraft, Java, and Horizon Launcher are present.\x1b[0m");
        }
        return;
    }

    if args.launch {
        UI::banner();
        if let (Some(java), Some(horizon)) = (java_detected.as_ref(), horizon_env.as_ref()) {
            match horizon.launch(&config, java) {
                Ok(status) => {
                    println!("Client process completed with status: {:?}", status.code());
                }
                Err(e) => {
                    eprintln!("\x1b[31mLaunch failed: {}\x1b[0m", e);
                }
            }
        } else {
            eprintln!("\x1b[31mCannot launch. Java or Horizon Environment missing.\x1b[0m");
        }
        return;
    }

    // Interactive Terminal UI loop
    interactive_loop(config);
}

fn interactive_loop(mut config: LauncherConfig) {
    loop {
        let mc_detected = detect_minecraft(config.minecraft_dir.as_deref());
        let java_detected = detect_java(config.java_home.as_deref());
        let horizon_env = HorizonEnvironment::discover(&config);

        UI::banner();
        UI::print_status(&config, mc_detected.as_ref(), java_detected.as_ref(), horizon_env.as_ref());

        println!("\x1b[1mOptions:\x1b[0m");
        println!("  \x1b[35m[1]\x1b[0m \x1b[1mLaunch Minecraft Horizon\x1b[0m");
        println!("  \x1b[35m[2]\x1b[0m Set Player Nickname");
        println!("  \x1b[35m[3]\x1b[0m Set Custom Minecraft Directory (e.g. FastClient)");
        println!("  \x1b[35m[4]\x1b[0m Set Custom Java JDK Home");
        println!("  \x1b[35m[5]\x1b[0m Run Resource Preparation");
        println!("  \x1b[35m[6]\x1b[0m Open Horizon Data Folder");
        println!("  \x1b[35m[0]\x1b[0m Exit");

        let choice = UI::read_input("\nSelect an option [1-6, 0]: ");
        match choice.as_str() {
            "1" | "" => {
                if let (Some(java), Some(horizon)) = (java_detected.as_ref(), horizon_env.as_ref()) {
                    println!("\n\x1b[32mStarting game...\x1b[0m");
                    let _ = horizon.launch(&config, java);
                } else {
                    println!("\n\x1b[31mMissing requirements: Ensure JDK 25 and horizon-launcher.exe are present.\x1b[0m");
                    let _ = UI::read_input("Press Enter to continue...");
                }
            }
            "2" => {
                let name = UI::read_input("\nEnter new player nickname: ");
                if !name.is_empty() {
                    let _ = config.set_player_name(&name);
                    println!("\x1b[32mPlayer name updated to: {}\x1b[0m", name);
                }
            }
            "3" => {
                let dir_str = UI::read_input("\nEnter full Minecraft path (leave blank to auto-detect): ");
                if dir_str.is_empty() {
                    let _ = config.set_minecraft_dir::<PathBuf>(None);
                    println!("\x1b[32mReset to automatic detection.\x1b[0m");
                } else {
                    let path = PathBuf::from(&dir_str);
                    if path.is_dir() {
                        let _ = config.set_minecraft_dir(Some(&path));
                        println!("\x1b[32mMinecraft path saved: {}\x1b[0m", path.display());
                    } else {
                        println!("\x1b[31mSpecified folder does not exist!\x1b[0m");
                        let _ = UI::read_input("Press Enter to continue...");
                    }
                }
            }
            "4" => {
                let java_str = UI::read_input("\nEnter full JDK home path (leave blank to auto-detect): ");
                if java_str.is_empty() {
                    let _ = config.set_java_home::<PathBuf>(None);
                    println!("\x1b[32mReset to automatic Java detection.\x1b[0m");
                } else {
                    let path = PathBuf::from(&java_str);
                    if path.is_dir() {
                        let _ = config.set_java_home(Some(&path));
                        println!("\x1b[32mJava home saved: {}\x1b[0m", path.display());
                    } else {
                        println!("\x1b[31mSpecified path does not exist!\x1b[0m");
                        let _ = UI::read_input("Press Enter to continue...");
                    }
                }
            }
            "5" => {
                if let (Some(mc), Some(java), Some(horizon)) = (mc_detected.as_ref(), java_detected.as_ref(), horizon_env.as_ref()) {
                    let _ = horizon.prepare_resources(mc, java);
                    let _ = UI::read_input("\nPress Enter to continue...");
                } else {
                    println!("\n\x1b[31mCannot prepare: Minecraft, Java, or Horizon Launcher missing.\x1b[0m");
                    let _ = UI::read_input("Press Enter to continue...");
                }
            }
            "6" => {
                if let Some(horizon) = horizon_env.as_ref() {
                    let _ = std::process::Command::new("explorer")
                        .arg(&horizon.data_dir)
                        .spawn();
                } else {
                    println!("\n\x1b[31mData folder not initialized.\x1b[0m");
                    let _ = UI::read_input("Press Enter to continue...");
                }
            }
            "0" | "exit" | "q" => {
                println!("\nExiting. Steady on.");
                break;
            }
            _ => {
                println!("\x1b[33mInvalid option selected.\x1b[0m");
            }
        }
    }
}
