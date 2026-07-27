use std::path::{Path, PathBuf};

const LEGACY_INSTALL_DIRECTORY: &str = "my-todos";
const CANONICAL_INSTALL_DIRECTORY: &str = "Todoz";
const MAIN_EXECUTABLE: &str = "my-todos.exe";

fn paths_equal_ignoring_windows_case(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

fn is_legacy_installed_executable(current_exe: &Path, local_data_dir: &Path) -> bool {
    let legacy_directory = local_data_dir.join(LEGACY_INSTALL_DIRECTORY);
    current_exe
        .parent()
        .is_some_and(|parent| paths_equal_ignoring_windows_case(parent, &legacy_directory))
}

fn canonical_executable(local_data_dir: &Path) -> PathBuf {
    local_data_dir
        .join(CANONICAL_INSTALL_DIRECTORY)
        .join(MAIN_EXECUTABLE)
}

/// Redirects the compatibility executable left in the pre-Todoz installation
/// directory to the canonical Todoz installation.
///
/// The v0.1.66 NSIS product-name change installed Todoz beside my-todos. The
/// migration installer replaces the old executable with the current build so
/// an update initiated by the legacy app can still complete its relaunch.
#[cfg(target_os = "windows")]
pub fn redirect_legacy_install() -> bool {
    let Some(local_data_dir) = dirs::data_local_dir() else {
        return false;
    };
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };

    if !is_legacy_installed_executable(&current_exe, &local_data_dir) {
        return false;
    }

    let canonical_exe = canonical_executable(&local_data_dir);
    if !canonical_exe.is_file() {
        return false;
    }

    match std::process::Command::new(&canonical_exe)
        .args(std::env::args_os().skip(1))
        .spawn()
    {
        Ok(_) => true,
        Err(error) => {
            eprintln!(
                "Failed to redirect legacy Todoz installation to {}: {error}",
                canonical_exe.display()
            );
            false
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn redirect_legacy_install() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_legacy_install_path_case_insensitively() {
        let local_data = Path::new(r"C:\Users\Test\AppData\Local");
        let current = Path::new(r"c:\users\test\appdata\local\MY-TODOS\my-todos.exe");

        assert!(is_legacy_installed_executable(current, local_data));
    }

    #[test]
    fn does_not_redirect_canonical_install_path() {
        let local_data = Path::new(r"C:\Users\Test\AppData\Local");
        let current = canonical_executable(local_data);

        assert!(!is_legacy_installed_executable(&current, local_data));
    }
}
