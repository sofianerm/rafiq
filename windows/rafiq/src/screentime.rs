//! Screen time for the notch's screen-time module: seconds per app per day, all local.
//!
//! Every few seconds the foreground window's process gets the time since the last sample, unless
//! nobody has touched the mouse or keyboard for a while (a video left playing past that stops
//! counting) or the session is locked. Apps are keyed by their exe name and shown under the
//! description the exe carries ("Google Chrome"). A week is kept in screentime.json. Only exe
//! names and durations are stored: no window titles, no URLs.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const TICK_SECS: u64 = 5;
/// Past this long without input, time stops counting
const IDLE_SECS: u64 = 300;
const KEEP_DAYS: usize = 8;
/// Processes that are not an app anyone is using
const IGNORED: [&str; 5] = ["lockapp", "searchhost", "shellexperiencehost", "startmenuexperiencehost", "rafiq"];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenTime {
    /// "YYYY-MM-DD" → exe stem → seconds
    pub days: BTreeMap<String, BTreeMap<String, u64>>,
    /// exe stem → the name to show
    pub names: BTreeMap<String, String>,
}

static DATA: Mutex<Option<ScreenTime>> = Mutex::new(None);

fn store_path() -> PathBuf {
    crate::config::config_path().with_file_name("screentime.json")
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn snapshot() -> ScreenTime {
    DATA.lock().unwrap().clone().unwrap_or_default()
}

fn save(d: &ScreenTime) {
    if let Ok(t) = serde_json::to_string(d) {
        let _ = std::fs::write(store_path(), t);
    }
}

/// Adds `secs` to `app` on `day`, and forgets days past the week
fn add(d: &mut ScreenTime, day: &str, app: &str, name: Option<String>, secs: u64) {
    *d.days.entry(day.to_string()).or_default().entry(app.to_string()).or_default() += secs;
    if let Some(n) = name {
        d.names.entry(app.to_string()).or_insert(n);
    }
    while d.days.len() > KEEP_DAYS {
        let oldest = d.days.keys().next().cloned().unwrap();
        d.days.remove(&oldest);
    }
}

#[cfg(windows)]
mod win {
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    pub fn idle_secs() -> u64 {
        let mut li = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        unsafe {
            if !GetLastInputInfo(&mut li).as_bool() {
                return 0;
            }
            (GetTickCount().wrapping_sub(li.dwTime) / 1000) as u64
        }
    }

    pub fn exe_path(pid: u32) -> Option<String> {
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
            let _ = CloseHandle(h);
            ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    /// The FileDescription in the exe's version resource, in its first listed language
    pub fn description(path: &str) -> Option<String> {
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let size = GetFileVersionInfoSizeW(PCWSTR(wide.as_ptr()), None);
            if size == 0 {
                return None;
            }
            let mut data = vec![0u8; size as usize];
            GetFileVersionInfoW(PCWSTR(wide.as_ptr()), 0, size, data.as_mut_ptr().cast()).ok()?;
            let query = |key: &str| -> Option<(*const u8, u32)> {
                let k: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
                let mut ptr = std::ptr::null_mut();
                let mut len = 0u32;
                VerQueryValueW(data.as_ptr().cast(), PCWSTR(k.as_ptr()), &mut ptr, &mut len)
                    .as_bool()
                    .then_some((ptr as *const u8, len))
                    .filter(|(p, l)| !p.is_null() && *l > 0)
            };
            let (tr, _) = query("\\VarFileInfo\\Translation")?;
            let lang = *(tr as *const u16);
            let cp = *(tr as *const u16).add(1);
            let (p, l) = query(&format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription"))?;
            let s = std::slice::from_raw_parts(p as *const u16, l as usize);
            let s = String::from_utf16_lossy(s).trim_end_matches('\0').trim().to_string();
            (!s.is_empty()).then_some(s)
        }
    }
}

/// macOS : l'app au premier plan vient de NSWorkspace, l'inactivité de CoreGraphics.
#[cfg(target_os = "macos")]
#[allow(unused_unsafe)]
mod mac {
    use objc2_app_kit::NSWorkspace;

    pub fn idle_secs() -> u64 {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
        }
        // kCGEventSourceStateCombinedSessionState, kCGAnyInputEventType
        let s = unsafe { CGEventSourceSecondsSinceLastEventType(0, u32::MAX) };
        if s.is_finite() && s > 0.0 { s as u64 } else { 0 }
    }

    /// (nom court de l'exécutable, nom affiché) de l'app au premier plan
    pub fn front() -> Option<(String, String)> {
        unsafe {
            let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
            let name = app.localizedName().map(|s| s.to_string());
            let exe = app
                .executableURL()
                .and_then(|u| u.lastPathComponent())
                .map(|s| s.to_string().to_lowercase());
            let stem = exe.or_else(|| name.as_ref().map(|n| n.to_lowercase()))?;
            Some((stem.clone(), name.unwrap_or(stem)))
        }
    }
}

/// L'app au premier plan : son nom court (la clé) et de quoi trouver son nom lisible
#[cfg(windows)]
fn front() -> Option<(String, String)> {
    let path = win::exe_path(crate::focus::fg_pid())?;
    let stem = std::path::Path::new(&path).file_stem()?.to_string_lossy().to_lowercase();
    Some((stem, path))
}
#[cfg(target_os = "macos")]
use mac::front;

#[cfg(windows)]
fn display_name(source: &str, stem: &str) -> String {
    win::description(source).unwrap_or_else(|| stem.to_string())
}
#[cfg(target_os = "macos")]
fn display_name(source: &str, _stem: &str) -> String {
    source.to_string()
}

#[cfg(windows)]
use win::idle_secs;
#[cfg(target_os = "macos")]
use mac::idle_secs;

pub fn start(app: AppHandle) {
    *DATA.lock().unwrap() =
        Some(std::fs::read_to_string(store_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default());
    #[cfg(any(windows, target_os = "macos"))]
    std::thread::spawn(move || {
        crate::activity::lower_thread_priority();
        let mut ticks = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(TICK_SECS));
            ticks += 1;
            if idle_secs() < IDLE_SECS {
                if let Some((stem, source)) = front() {
                    if !IGNORED.contains(&stem.as_str()) {
                        let mut guard = DATA.lock().unwrap();
                        let d = guard.get_or_insert_with(Default::default);
                        // The description is looked up once per app, the first time it is seen
                        let name = if d.names.contains_key(&stem) {
                            None
                        } else {
                            Some(display_name(&source, &stem))
                        };
                        add(d, &today(), &stem, name, TICK_SECS);
                    }
                }
            }
            // To disk every minute, to the notch every half minute
            if ticks % 12 == 0 {
                save(&snapshot());
            }
            if ticks % 6 == 0 {
                let _ = app.emit("screentime", snapshot());
            }
        }
    });
}

#[tauri::command]
pub fn get_screentime() -> ScreenTime {
    snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_per_day_and_keeps_a_week() {
        let mut d = ScreenTime::default();
        add(&mut d, "2026-09-01", "chrome", Some("Google Chrome".into()), 5);
        add(&mut d, "2026-09-01", "chrome", Some("ignored".into()), 5);
        assert_eq!(d.days["2026-09-01"]["chrome"], 10);
        assert_eq!(d.names["chrome"], "Google Chrome"); // the first name sticks
        for day in 2..=12 {
            add(&mut d, &format!("2026-09-{day:02}"), "code", None, 1);
        }
        assert_eq!(d.days.len(), KEEP_DAYS);
        assert!(!d.days.contains_key("2026-09-01"));
        assert!(d.days.contains_key("2026-09-12"));
    }
}
