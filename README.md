<div align="center">

# Fast Horizon Launcher

**A lightweight, offline-first Rust launcher for Minecraft Horizon.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)]()
[![Minecraft](https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F?logo=minecraft)](https://minecraft.net)

</div>

---

## ⚡ Overview

**Fast Horizon Launcher** is a standalone, blazingly fast launcher designed to prepare and run native **[Horizon](https://github.com/horizon-rust/horizon)** (Minecraft rewritten in Rust) completely offline without Microsoft account requirements.

It automatically discovers and integrates with your existing Minecraft assets from **FastClient**, **TLauncher**, or standard `.minecraft` directories, automatically resolves **JDK 25+**, and bootstraps the Horizon native client engine.

---

## ✨ Features

- **Zero-Login Offline Play**: Start the game with your custom nickname—no Microsoft authentication or telemetry.
- **Smart Directory Detection**: Auto-detects Minecraft files in:
  - FastClient (`%APPDATA%\.fastclient`)
  - Standard Minecraft / TLauncher (`%APPDATA%\.minecraft`)
  - Custom directories via CLI or interactive menu.
- **Automated JDK 25+ Resolution**: Scans FastClient runtimes, system `JAVA_HOME`, and standard JDK paths without manual environment variable setup.
- **Native Horizon Integration**: Coordinates `horizon-launcher.exe`, executes resource extraction for Minecraft 26.3, and directly launches `horizon-client.exe`.
- **Hybrid Interface**: Supports both an interactive colored TUI terminal menu and headless CLI flags for automation and batch scripts.

---

## 🚀 Quick Start

### 1. Requirements
- Existing Minecraft Java 26.3 files (from FastClient, TLauncher, or standard launcher).
- OpenJDK 25+ (automatically detected if installed or bundled in FastClient).
- `horizon-launcher.exe` placed in the same folder or `bin/`.

### 2. Run Interactively
Simply run the executable:
```bash
fast-horizon-launcher
```

### 3. CLI Mode
```bash
# Check detected sources and status
fast-horizon-launcher --check

# Launch game directly with custom nickname
fast-horizon-launcher --launch --name Steve

# Run asset and registry preparation
fast-horizon-launcher --prepare

# Override Minecraft source and Java path
fast-horizon-launcher --minecraft "D:\Games\.minecraft" --java "C:\Program Files\Java\jdk-25" --launch
```

---

## 🛠️ Building From Source

Make sure you have Rust 1.80+ installed:

```bash
git clone https://github.com/AN0N/fast-horizon-launcher.git
cd fast-horizon-launcher
cargo build --release
```

The optimized binary will be located in `target/release/fast-horizon-launcher.exe`.

---

## ⚙️ Configuration

Configuration is automatically maintained in `launcher-config.json` next to the binary:

```json
{
  "player_name": "Player",
  "minecraft_dir": null,
  "java_home": null,
  "offline_mode": true,
  "horizon_data_dir": null
}
```

---

## 📜 License

Licensed under the [MIT License](LICENSE).
