use serde::{Deserialize, Serialize};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub install_time: String,
    pub pinned_to_taskbar: bool,
    pub icon_path: Option<String>,
    pub args: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub apps: Vec<InstalledApp>,
    pub storage_mode: String, // "portable" or "appdata"
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            apps: Vec::new(),
            storage_mode: "appdata".to_string(),
        }
    }

    pub fn get_registry_path(storage_mode: &str) -> PathBuf {
        if storage_mode == "portable" {
            // Store next to the executable
            let exe_path = std::env::current_exe().unwrap_or_default();
            exe_path
                .parent()
                .unwrap_or(&std::path::Path::new("."))
                .join("portable-installer-registry.json")
        } else {
            // Store in AppData
            let app_data = dirs::data_local_dir().unwrap_or_default();
            app_data
                .join("PortableInstaller")
                .join("registry.json")
        }
    }

    pub fn load(storage_mode: &str) -> Self {
        let path = Self::get_registry_path(storage_mode);
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| Registry::new())
        } else {
            let mut reg = Registry::new();
            reg.storage_mode = storage_mode.to_string();
            reg
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_registry_path(&self.storage_mode);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn sanitize_filename(name: &str) -> String {
    let illegal = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    name.chars()
        .map(|c| if illegal.contains(&c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

fn get_start_menu_folder() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Portable Apps")
}

fn create_shortcut(
    target_path: &str,
    display_name: &str,
    icon_path: Option<&str>,
) -> Result<PathBuf, String> {
    let target = std::path::Path::new(target_path);
    if !target.exists() {
        return Err(format!("Target file does not exist: {}", target_path));
    }

    let folder = get_start_menu_folder();
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;

    let safe_name = sanitize_filename(display_name);
    let shortcut_path = folder.join(format!("{}.lnk", safe_name));

    let working_dir = target
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_string_lossy()
        .to_string();

    let icon_arg = match icon_path {
        Some(p) if !p.is_empty() => format!("{},0", p.replace('\'', "''")),
        _ => format!("{},0", target_path.replace('\'', "''")),
    };

    let ps_script = format!(
        "$s = New-Object -ComObject WScript.Shell; \
         $c = $s.CreateShortcut('{0}'); \
         $c.TargetPath = '{1}'; \
         $c.WorkingDirectory = '{2}'; \
         $c.IconLocation = '{3}'; \
         $c.Save()",
        shortcut_path.to_string_lossy().replace('\'', "''"),
        target_path.replace('\'', "''"),
        working_dir.replace('\'', "''"),
        icon_arg,
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to spawn powershell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell failed: {}", stderr));
    }

    Ok(shortcut_path)
}

fn remove_shortcut(display_name: &str) -> Result<(), String> {
    let folder = get_start_menu_folder();
    let safe_name = sanitize_filename(display_name);
    let shortcut_path = folder.join(format!("{}.lnk", safe_name));
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn install_app(
    name: String,
    executable_path: String,
    icon_path: Option<String>,
    args: Option<String>,
    storage_mode: String,
    _pin_to_taskbar: bool,
) -> Result<InstalledApp, String> {
    let id = uuid::Uuid::new_v4().to_string();

    let icon_ref = icon_path.as_deref();
    create_shortcut(&executable_path, &name, icon_ref)?;

    let app = InstalledApp {
        id: id.clone(),
        name: name.clone(),
        executable_path,
        install_time: chrono_now(),
        pinned_to_taskbar: false,
        icon_path,
        args,
    };

    let mut registry = Registry::load(&storage_mode);
    registry.storage_mode = storage_mode;
    registry.apps.push(app.clone());
    registry.save()?;

    Ok(app)
}

#[tauri::command]
fn uninstall_app(app_id: String, storage_mode: String) -> Result<(), String> {
    let mut registry = Registry::load(&storage_mode);

    if let Some(pos) = registry.apps.iter().position(|a| a.id == app_id) {
        let app = registry.apps.remove(pos);
        remove_shortcut(&app.name)?;
        registry.save()?;
        Ok(())
    } else {
        Err("App not found".to_string())
    }
}

#[tauri::command]
fn get_installed_apps(storage_mode: String) -> Vec<InstalledApp> {
    let registry = Registry::load(&storage_mode);
    registry.apps
}

#[tauri::command]
fn get_storage_mode() -> String {
    // Check if portable registry exists next to exe
    let exe_path = std::env::current_exe().unwrap_or_default();
    let portable_path = exe_path
        .parent()
        .unwrap_or(&std::path::Path::new("."))
        .join("portable-installer-registry.json");

    if portable_path.exists() {
        "portable".to_string()
    } else {
        "appdata".to_string()
    }
}

#[tauri::command]
fn set_storage_mode(mode: String) -> Result<(), String> {
    let old_mode = get_storage_mode();
    if old_mode == mode {
        return Ok(());
    }

    // Load from old location
    let old_registry = Registry::load(&old_mode);

    // Save to new location
    let mut new_registry = old_registry;
    new_registry.storage_mode = mode;
    new_registry.save()?;

    // Optionally delete old registry file
    let old_path = Registry::get_registry_path(&old_mode);
    if old_path.exists() {
        let _ = fs::remove_file(old_path);
    }

    Ok(())
}

#[tauri::command]
fn scan_folder_for_apps(folder_path: String) -> Result<Vec<ScannedApp>, String> {
    let mut apps = Vec::new();
    let path = std::path::Path::new(&folder_path);

    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    scan_directory(path, &mut apps, 2)?; // Limit depth to 2
    Ok(apps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedApp {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
}

fn scan_directory(
    dir: &std::path::Path,
    apps: &mut Vec<ScannedApp>,
    depth: usize,
) -> Result<(), String> {
    if depth == 0 {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;

        if metadata.is_file() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if ext_lower == "exe" || ext_lower == "bat" || ext_lower == "cmd" || ext_lower == "ps1" {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    apps.push(ScannedApp {
                        name,
                        path: path.to_string_lossy().to_string(),
                        is_directory: false,
                    });
                }
            }
        } else if metadata.is_dir() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            apps.push(ScannedApp {
                name: format!("{}/ (folder)", name),
                path: path.to_string_lossy().to_string(),
                is_directory: true,
            });
            scan_directory(&path, apps, depth - 1)?;
        }
    }
    Ok(())
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

#[tauri::command]
fn write_log(contents: String) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let log_dir = exe_path
        .parent()
        .unwrap_or(&std::path::Path::new("."))
        .join("logs");
    fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;

    let log_file = log_dir.join("debug.log");

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .map_err(|e| e.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn launch_app(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn extract_icon(exe_path: String) -> Result<String, String> {
    let target = std::path::Path::new(&exe_path);
    if !target.exists() {
        return Err(format!("File not found: {}", exe_path));
    }

    let icon_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("icons_cache");
    fs::create_dir_all(&icon_dir).map_err(|e| e.to_string())?;

    let stem = target.file_stem().unwrap_or_default().to_string_lossy();
    let meta = fs::metadata(target).map_err(|e| e.to_string())?;
    let mtime = meta
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let out_png = icon_dir.join(format!("{}_{}.png", stem, mtime));

    if out_png.exists() {
        let data = fs::read(&out_png).map_err(|e| e.to_string())?;
        return Ok(format!("data:image/png;base64,{}", base64_encode(&data)));
    }

    let ps = format!(
        "Add-Type -AssemblyName System.Drawing; \
         $icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{}'); \
         $bmp = $icon.ToBitmap(); \
         $bmp.Save('{}'); \
         $bmp.Dispose(); $icon.Dispose()",
        exe_path.replace('\'', "''"),
        out_png.to_string_lossy().replace('\'', "''"),
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Icon extraction failed: {}", stderr));
    }

    if out_png.exists() {
        let data = fs::read(&out_png).map_err(|e| e.to_string())?;
        Ok(format!("data:image/png;base64,{}", base64_encode(&data)))
    } else {
        Err("Icon file was not created".to_string())
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            install_app,
            uninstall_app,
            get_installed_apps,
            get_storage_mode,
            set_storage_mode,
            scan_folder_for_apps,
            write_log,
            launch_app,
            extract_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
