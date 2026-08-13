//! Android 专属的文件访问能力（当前为无 JNI 桥的降级实现）。
//!
//! - 权限检测：用文件系统探测——能在 `/storage/emulated/0` 下创建/删除探针目录，
//!   说明已获得「所有文件访问」（Android 11+ 未授权时原始路径写入会被拒；API<30 无此权限模型，天然可写）。
//! - 跳转系统设置页需要 Intent（走 tauri 官方插件系统的 Kotlin Plugin 桥，工作量大），
//!   当前返回引导文案由前端展示；TODO: 移植接入插件后改为一键直达。

/// 用户是否已授予「所有文件访问」。
pub fn is_all_files_granted(_app: &tauri::AppHandle) -> bool {
    let probe = "/storage/emulated/0/.gal-launcher-perm-probe";
    match std::fs::create_dir_all(probe) {
        Ok(()) => {
            let _ = std::fs::remove_dir(probe);
            true
        }
        Err(_) => false,
    }
}

/// 请求「所有文件访问」：当前无 JNI 桥，返回引导提示由前端展示。
pub fn request_all_files_access(_app: &tauri::AppHandle) -> Result<(), String> {
    Err("请在系统设置中手动开启：设置 → 应用 → GAL 启动器 → 权限 → 允许管理所有文件".into())
}
