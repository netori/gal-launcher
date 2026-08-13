//! 平台相关的文件系统辅助：移动端路径规范化、授权根目录判断。

use std::path::Path;

/// Android 上把分隔符统一为 `/`（SAF 风格路径避免反斜杠）；其它平台原样返回。
#[allow(dead_code)] // M3 导出路径规范化时使用
pub fn normalize(path: &str) -> String {
    #[cfg(target_os = "android")]
    {
        path.replace('\\', "/")
    }
    #[cfg(not(target_os = "android"))]
    {
        path.to_string()
    }
}

/// 判断 `path` 是否位于任一已授权根目录下。空列表表示不限制（桌面端）。
#[allow(dead_code)] // M3 导出/写入校验时使用
pub fn is_under_any_root(path: &str, roots: &[String]) -> bool {
    if roots.is_empty() {
        return true;
    }
    let p = Path::new(path);
    roots.iter().any(|r| p.starts_with(Path::new(r)))
}
