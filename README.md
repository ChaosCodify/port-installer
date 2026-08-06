# Portable Installer

A portable Windows application that adds programs to the Start Menu without traditional installation.

## Features

- **No Installation Required** - Add apps to Start Menu without modifying system files
- **Portable** - Run from anywhere, carry your app registry on a USB drive
- **Taskbar Pinning** - Optionally pin apps to the taskbar
- **Folder Scanning** - Scan folders to find installable apps
- **Dual Storage Modes** - Store registry in AppData (recommended) or next to the .exe

## How It Works

1. **Install** - Creates a .lnk shortcut in your Start Menu under "Portable Apps"
2. **Uninstall** - Removes the shortcut and registry entry
3. **Launch** - Run apps directly from the Start Menu or the installer

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- [Tauri CLI](https://tauri.app/)

### Build Steps

```bash
# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Usage

### Adding Apps

1. Click "Install New" in the sidebar
2. Click "Browse" to select an .exe, .bat, .cmd, or .ps1 file
3. Customize the name, icon, and arguments if needed
4. Check "Pin to Taskbar" if desired
5. Click "Install"

### Scanning Folders

1. Click "Install New" in the sidebar
2. Click "Select Folder to Scan"
3. Browse for apps in the results
4. Click "Install" on any app you want to add

### Removing Apps

1. Find the app in the "Installed Apps" list
2. Click "Uninstall"
3. Confirm the removal

## Storage Modes

- **AppData (Recommended)** - Registry stored in `%LOCALAPPDATA%\PortableInstaller\registry.json`
- **Portable** - Registry stored next to the .exe as `portable-installer-registry.json`

## Notes

- Shortcuts are created in `%LOCALAPPDATA%\Microsoft\Windows\Start Menu\Programs\Portable Apps\`
- Taskbar pinning is best-effort and may not work for all app types
- The installer is self-contained and can be moved anywhere

## License

MIT
