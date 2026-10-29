//! Start at sign-in: an HKCU\...\Run registry value (per user, no administrator needed).
//! The command carries --silent: wait in the background, show no bar without sessions, appear when one starts.
//! Implemented with reg.exe, so no new dependency.

//! Sur macOS : un LaunchAgent (~/Library/LaunchAgents/com.sofianerm.rafiq.plist), voir `mac` plus bas.

#[cfg(target_os = "macos")]
pub use mac::{disable, enable, is_enabled};
#[cfg(not(target_os = "macos"))]
pub use win::{disable, enable, is_enabled};

#[cfg(not(target_os = "macos"))]
mod win {
use std::process::Command;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "Rafiq";
/// Le nom de la valeur avant que l'app devienne Rafiq : retirée à chaque changement, pour ne pas lancer l'ancien exe.
const OLD_NAME: &str = "Codenotch";

fn reg(args: &[&str]) -> Option<(bool, String)> {
    let mut c = Command::new("reg");
    c.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c.output().ok().map(|o| {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.success(), text)
    })
}

pub fn is_enabled() -> bool {
    reg(&["query", RUN_KEY, "/v", NAME])
        .map(|(ok, out)| ok && out.contains(NAME))
        .unwrap_or(false)
}

pub fn enable() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let _ = reg(&["delete", RUN_KEY, "/v", OLD_NAME, "/f"]);
    let val = format!("\"{}\" --silent", exe.display());
    match reg(&["add", RUN_KEY, "/v", NAME, "/t", "REG_SZ", "/d", &val, "/f"]) {
        Some((true, _)) => Ok("start at sign-in enabled (silent until a session appears)".into()),
        Some((false, out)) => Err(out),
        None => Err("reg.exe failed to run".into()),
    }
}

pub fn disable() -> Result<String, String> {
    let _ = reg(&["delete", RUN_KEY, "/v", OLD_NAME, "/f"]);
    match reg(&["delete", RUN_KEY, "/v", NAME, "/f"]) {
        Some((true, _)) => Ok("start at sign-in disabled".into()),
        Some((false, out)) => {
            if out.to_lowercase().contains("unable to find") || out.contains("找不到") { // reg.exe answers in the OS language; "找不到" is the Chinese "unable to find"
                Ok("start at sign-in was not enabled".into())
            } else {
                Err(out)
            }
        }
        None => Err("reg.exe failed to run".into()),
    }
}
}

#[cfg(target_os = "macos")]
mod mac {
    use std::path::PathBuf;

    const LABEL: &str = "com.sofianerm.rafiq";

    fn plist_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join("Library").join("LaunchAgents").join(format!("{LABEL}.plist")))
    }

    /// Caractères spéciaux du XML, pour un chemin d'app qui en contiendrait
    fn xml(s: &str) -> String {
        s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    }

    pub fn is_enabled() -> bool {
        plist_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn enable() -> Result<String, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let path = plist_path().ok_or("cannot find the home directory")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array><string>{}</string><string>--silent</string></array>
  <key>RunAtLoad</key><true/>
</dict>
</plist>
"#,
            xml(&exe.display().to_string())
        );
        std::fs::write(&path, plist).map_err(|e| e.to_string())?;
        Ok("start at sign-in enabled".into())
    }

    pub fn disable() -> Result<String, String> {
        match plist_path() {
            Some(p) if p.exists() => std::fs::remove_file(p).map(|_| "start at sign-in disabled".into()).map_err(|e| e.to_string()),
            _ => Ok("start at sign-in was not enabled".into()),
        }
    }
}