//! Ce qui diffère entre Windows et macOS et que plusieurs fichiers utilisent : ouvrir un lien ou un
//! dossier, lire l'état du bouton gauche de la souris, trouver rafiq-hook à côté de l'app.

use std::path::{Path, PathBuf};

/// Ouvre une page dans le navigateur par défaut.
pub fn open_url(url: &str) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(0x0800_0000)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Ouvre un dossier dans l'Explorateur (Windows) ou le Finder (macOS).
pub fn reveal_dir(dir: &Path) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("explorer").arg(dir.as_os_str()).creation_flags(0x0800_0000).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(dir.as_os_str()).spawn();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(dir.as_os_str()).spawn();
    }
}

#[cfg(windows)]
pub fn left_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(target_os = "macos")]
pub fn left_button_down() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    }
    // kCGEventSourceStateCombinedSessionState = 0, kCGMouseButtonLeft = 0
    unsafe { CGEventSourceButtonState(0, 0) }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn left_button_down() -> bool {
    false
}

/// rafiq-hook livré avec l'app : à côté de l'exe sous Windows, dans `Contents/Resources` du .app sur macOS.
pub fn hook_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    if cfg!(windows) {
        return Some(dir.join("rafiq-hook.exe"));
    }
    let bundled = dir.parent().map(|c| c.join("Resources").join("rafiq-hook"));
    match bundled {
        Some(p) if p.exists() => Some(p),
        _ => Some(dir.join("rafiq-hook")),
    }
}
