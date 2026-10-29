//! Clipboard history for the notch's clipboard module: the last texts copied, newest first.
//!
//! Windows bumps a sequence number on every clipboard change, so a cheap poll of it is enough; the
//! text is only read when it moves. Anything a password manager marks as not for history or for
//! monitors (the formats Windows' own Win+V history honours) is skipped. Only pinned items are
//! written to disk: the rest lives in memory and is gone when the app quits.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

/// Unpinned items kept
const KEEP: usize = 30;
/// Longer texts are cut: this is a history to glance at, not a store
const MAX_CHARS: usize = 20_000;
const POLL_MS: u64 = 400;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: u64,
    pub text: String,
    /// When it was (last) copied, ms since the epoch
    pub at: u64,
    pub pinned: bool,
}

static ITEMS: Mutex<Vec<ClipItem>> = Mutex::new(Vec::new());

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn store_path() -> PathBuf {
    crate::config::config_path().with_file_name("clipboard-pinned.json")
}

fn save_pinned(items: &[ClipItem]) {
    let pinned: Vec<&ClipItem> = items.iter().filter(|i| i.pinned).collect();
    if let Ok(t) = serde_json::to_string_pretty(&pinned) {
        let _ = std::fs::write(store_path(), t);
    }
}

/// Puts a newly copied text on top. The same text copied again moves up rather than repeating,
/// and keeps its pin. Returns whether anything changed.
fn push(items: &mut Vec<ClipItem>, text: String, at: u64) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    let text: String = if text.chars().count() > MAX_CHARS { text.chars().take(MAX_CHARS).collect() } else { text };
    if items.first().is_some_and(|i| i.text == text) {
        return false;
    }
    // Copied again (from the notch, say): the same item moves up, keeping its id and its pin
    let (id, pinned) = match items.iter().position(|i| i.text == text) {
        Some(p) => {
            let old = items.remove(p);
            (old.id, old.pinned)
        }
        None => (items.iter().map(|i| i.id).max().unwrap_or(0) + 1, false),
    };
    items.insert(0, ClipItem { id, text, at, pinned });
    // Cap the unpinned ones only, dropping the oldest
    let mut unpinned = 0;
    items.retain(|i| {
        if i.pinned {
            return true;
        }
        unpinned += 1;
        unpinned <= KEEP
    });
    true
}

#[cfg(windows)]
mod win {
    use windows::core::w;
    use windows::Win32::System::DataExchange::{GetClipboardSequenceNumber, IsClipboardFormatAvailable, RegisterClipboardFormatW};

    pub fn sequence() -> u32 {
        unsafe { GetClipboardSequenceNumber() }
    }

    /// The formats password managers add to keep a secret out of clipboard history and monitors
    pub fn is_private() -> bool {
        unsafe {
            [
                w!("ExcludeClipboardContentFromMonitorProcessing"),
                w!("CanIncludeInClipboardHistory"),
                w!("Clipboard Viewer Ignore"),
            ]
            .into_iter()
            .any(|name| {
                let f = RegisterClipboardFormatW(name);
                f != 0 && IsClipboardFormatAvailable(f).is_ok()
            })
        }
    }
}

/// macOS : NSPasteboard a son propre compteur de changements ; les gestionnaires de mots de passe
/// marquent leurs secrets avec les types de nspasteboard.org.
#[cfg(target_os = "macos")]
#[allow(unused_unsafe)]
mod mac {
    use objc2_app_kit::NSPasteboard;

    pub fn sequence() -> u32 {
        unsafe { NSPasteboard::generalPasteboard().changeCount() as u32 }
    }

    pub fn is_private() -> bool {
        unsafe {
            let Some(types) = NSPasteboard::generalPasteboard().types() else { return false };
            (0..types.count()).any(|i| {
                let t = types.objectAtIndex(i).to_string();
                t == "org.nspasteboard.ConcealedType" || t == "org.nspasteboard.TransientType"
            })
        }
    }
}

#[cfg(windows)]
use win as sys;
#[cfg(target_os = "macos")]
use mac as sys;

fn broadcast(app: &AppHandle) {
    let items = ITEMS.lock().unwrap().clone();
    let _ = app.emit("clipboard", &items);
}

pub fn start(app: AppHandle) {
    *ITEMS.lock().unwrap() = std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Vec<ClipItem>>(&t).ok())
        .unwrap_or_default();
    #[cfg(any(windows, target_os = "macos"))]
    std::thread::spawn(move || {
        let mut seen = 0u32;
        loop {
            let seq = sys::sequence();
            if seq != seen {
                seen = seq;
                if !sys::is_private() {
                    // A clipboard another app still holds open refuses the read; the next change tries again
                    let text = arboard::Clipboard::new().and_then(|mut c| c.get_text());
                    if let Ok(text) = text {
                        let changed = push(&mut ITEMS.lock().unwrap(), text, now_ms());
                        if changed {
                            broadcast(&app);
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(POLL_MS));
        }
    });
}

#[tauri::command]
pub fn get_clipboard() -> Vec<ClipItem> {
    ITEMS.lock().unwrap().clone()
}

/// Puts an item back on the clipboard; the watcher then sees it and moves it to the top
#[tauri::command]
pub fn clipboard_copy(id: u64) -> Result<(), String> {
    let text = ITEMS.lock().unwrap().iter().find(|i| i.id == id).map(|i| i.text.clone()).ok_or("no such item")?;
    arboard::Clipboard::new().and_then(|mut c| c.set_text(text)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clipboard_pin(app: AppHandle, id: u64, pinned: bool) {
    {
        let mut items = ITEMS.lock().unwrap();
        if let Some(i) = items.iter_mut().find(|i| i.id == id) {
            i.pinned = pinned;
        }
        save_pinned(&items);
    }
    broadcast(&app);
}

#[tauri::command]
pub fn clipboard_delete(app: AppHandle, id: u64) {
    {
        let mut items = ITEMS.lock().unwrap();
        items.retain(|i| i.id != id);
        save_pinned(&items);
    }
    broadcast(&app);
}

/// Empties the history; pinned items stay
#[tauri::command]
pub fn clipboard_clear(app: AppHandle) {
    ITEMS.lock().unwrap().retain(|i| i.pinned);
    broadcast(&app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_no_repeats_pins_survive() {
        let mut v = Vec::new();
        assert!(push(&mut v, "a".into(), 1));
        assert!(push(&mut v, "b".into(), 2));
        assert!(!push(&mut v, "b".into(), 3)); // already on top
        assert!(!push(&mut v, "  \n".into(), 4)); // blank
        v.iter_mut().find(|i| i.text == "a").unwrap().pinned = true;
        assert!(push(&mut v, "a".into(), 5)); // copied again: moves up, keeps its pin
        assert_eq!(v.iter().map(|i| i.text.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        assert!(v[0].pinned);
        for n in 0..KEEP + 5 {
            push(&mut v, format!("t{n}"), 10 + n as u64);
        }
        assert_eq!(v.iter().filter(|i| !i.pinned).count(), KEEP);
        assert!(v.iter().any(|i| i.text == "a" && i.pinned)); // the pin outlives the cap
    }
}
