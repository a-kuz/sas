use std::path::PathBuf;

pub fn get_resource_base_path() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from("q3-resources")
    }
    
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let app_support = PathBuf::from(home).join("Library/Application Support/SAS/q3-resources");
            if app_support.exists() {
                return app_support;
            }
        }
        PathBuf::from("q3-resources")
    }
    
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let sas_dir = PathBuf::from(appdata).join("SAS/q3-resources");
            if sas_dir.exists() {
                return sas_dir;
            }
        }
        PathBuf::from("q3-resources")
    }
    
    #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let config_dir = PathBuf::from(home).join(".local/share/sas/q3-resources");
            if config_dir.exists() {
                return config_dir;
            }
        }
        PathBuf::from("q3-resources")
    }
    
    #[cfg(all(not(target_arch = "wasm32"), not(any(target_os = "macos", target_os = "windows", target_os = "linux"))))]
    {
        PathBuf::from("q3-resources")
    }
}

pub fn get_resource_path(relative_path: &str) -> String {
    let base = get_resource_base_path();
    let full_path = base.join(relative_path);
    full_path.to_string_lossy().to_string()
}


