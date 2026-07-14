![](data/icons/com.parchlinux.gamehub.svg)

# Parch Linux Gamehub

a single tool that gamers need

### Features

- Dashboard with system status (GPU, driver, Vulkan, Chaotic AUR) and quick actions
- Launcher grid with local application icons for Steam, Lutris, Heroic, Bottles, Minigalaxy
- Install, launch, and remove game launchers directly from the app
- Compatibility tool management for GE-Proton, Proton-CachyOS, DXVK, VKD3D-Proton
- Dedicated Steam Gaming Session setup with auto-start and gaming optimizations
- 11 gaming tools across Wine/Compatibility, Performance, and Additional categories
- Chaotic AUR setup and mirrorman launcher for Parch Repository Manager
- GSettings-backed preferences for auto-update, notifications, CPU governor, kernel parameters, Esync/Fsync
- Local icon cache for launcher and dashboard cards
- Polkit-based privilege escalation (no password prompts in the terminal)

### Install

**From Parch Linux repository**

```
sudo pacman -S gamehub
```

**From source**

Requires Rust 1.75+ and GTK4/libadwaita development packages.

```
git clone https://github.com/parchlinux/gamehub-rs
cd gamehub-rs
./run.sh
```

To compile the GSettings schema and run without the script:

```
glib-compile-schemas data/
GSETTINGS_SCHEMA_DIR=$PWD/data cargo run
```

### Build

```
cargo build --release
```

The binary is at `target/release/gamehub`.

### License

AGPL-3.0
