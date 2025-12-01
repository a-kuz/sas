use std::path::PathBuf;
use std::sync::OnceLock;

static RESOURCE_BASE: OnceLock<PathBuf> = OnceLock::new();

pub fn get_resource_base_path() -> PathBuf {
    RESOURCE_BASE
        .get_or_init(|| {
            #[cfg(target_arch = "wasm32")]
            {
                return PathBuf::from("q3-resources");
            }

            if std::path::Path::new("q3-resources").exists() {
                return PathBuf::from("q3-resources");
            }

            #[cfg(target_os = "macos")]
            {
                if let Some(home) = std::env::var_os("HOME") {
                    let app_support =
                        PathBuf::from(home).join("Library/Application Support/SAS/q3-resources");
                    if app_support.exists() {
                        return app_support;
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                if let Some(appdata) = std::env::var_os("APPDATA") {
                    let sas_dir = PathBuf::from(appdata).join("SAS\\q3-resources");
                    if sas_dir.exists() {
                        return sas_dir;
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                if let Some(home) = std::env::var_os("HOME") {
                    let config_dir = PathBuf::from(home).join(".local/share/sas/q3-resources");
                    if config_dir.exists() {
                        return config_dir;
                    }
                }
            }

            PathBuf::from("q3-resources")
        })
        .clone()
}

pub fn get_resource_path(relative_path: &str) -> String {
    let base = get_resource_base_path();
    let full_path = base.join(relative_path);
    full_path.to_string_lossy().to_string()
}
