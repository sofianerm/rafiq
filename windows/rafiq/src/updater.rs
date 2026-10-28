//! Keeping the Windows app up to date, the way Sparkle keeps the Mac one up to date.
//!
//! Until this existed there was no update path on Windows at all: someone who installed
//! `Rafiq-Setup.exe` stayed on that build for good, because nothing in the app ever
//! mentioned that a newer one had been cut. Every release reached them only if they
//! happened to look at the repository again.
//!
//! The feed is `latest.json` on the newest GitHub release, written by the Windows Package
//! workflow beside the installer it describes — the same `releases/latest/download/` URL the
//! README links to, so there is one place a release is published and one place it is read.
//!
//! What it does *not* do is nag. A check that fails — no network, a feed that has not been
//! written yet, a signature that does not verify — leaves the app exactly as it was and says
//! so only in the log. The Mac's updater was hanging on "Checking…" as recently as 1.14.0;
//! the lesson taken from that is that an update check is never worth blocking on.
//!
//! Rafiq has no releases of its own yet, so `plugins.updater.endpoints` is empty and the
//! pubkey is the placeholder: the updater is switched off (no check, no install, no row in
//! Settings). It used to point at upstream vinzdg/codenotch, whose builds must never replace
//! Rafiq. To turn it back on, put our own `latest.json` URL in `endpoints` and our public key
//! in `pubkey` (see the README) — nothing else needs to change.

use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// What the Settings page shows next to the version.
///
/// `checking` is deliberately not a state the page can get stuck in: every path that sets it
/// also sets something else before it returns.
#[derive(Clone, Serialize, Default)]
pub struct UpdateState {
    /// False when no feed is configured: Settings then hides the Updates row.
    pub enabled: bool,
    /// The version on offer, when one is newer than this build.
    pub available: Option<String>,
    /// True only between a check starting and finishing.
    pub checking: bool,
    /// True while the download and install are running.
    pub installing: bool,
    /// Set when the last check or install failed, for the page to show quietly.
    pub message: Option<String>,
}

static STATE: Mutex<Option<UpdateState>> = Mutex::new(None);

/// The status lock guards one small struct. Poisoning it would take the About pane down with
/// `unwrap`, which is a steep price for a version string, so take it back and carry on.
fn state() -> std::sync::MutexGuard<'static, Option<UpdateState>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Only ever called once `configured` has passed, so every state it broadcasts is enabled.
fn set(app: &AppHandle, mut next: UpdateState) {
    next.enabled = true;
    *state() = Some(next.clone());
    let _ = app.emit("update_state", &next);
}

#[tauri::command]
pub fn get_update_state(app: AppHandle) -> UpdateState {
    let mut st = state().clone().unwrap_or_default();
    st.enabled = configured(&app);
    st
}

/// The placeholder that ships in `tauri.conf.json` until a signing key exists.
const UNSET_PUBKEY: &str = "REPLACE_WITH_TAURI_PUBLIC_KEY";

/// Whether a feed and a real signing key have both been configured.
///
/// A build made before the key was generated would otherwise check a feed it can never
/// verify, and report a failure every time for a reason the user can do nothing about.
/// Silence is the right answer there — and with no endpoint there is nothing to check.
fn configured(app: &AppHandle) -> bool {
    configured_in(app.config().plugins.0.get("updater"))
}

fn configured_in(updater: Option<&serde_json::Value>) -> bool {
    let Some(u) = updater else { return false };
    let key_ok = u
        .get("pubkey")
        .and_then(|k| k.as_str())
        .is_some_and(|k| !k.is_empty() && k != UNSET_PUBKEY);
    let feed_ok = u
        .get("endpoints")
        .and_then(|e| e.as_array())
        .is_some_and(|e| e.iter().any(|x| x.as_str().is_some_and(|s| !s.trim().is_empty())));
    key_ok && feed_ok
}

/// Looks for a newer release. Answers immediately; the result arrives as `update_state`.
///
/// Called once a few seconds after launch, and again whenever someone opens Settings and
/// presses Check. Nothing here touches the notch.
#[tauri::command]
pub fn check_for_update(app: AppHandle) {
    if !configured(&app) {
        return;
    }
    if state().as_ref().is_some_and(|s| s.checking || s.installing) {
        return;
    }
    set(&app, UpdateState { checking: true, ..Default::default() });
    std::thread::spawn(move || {
        let result = tauri::async_runtime::block_on(async {
            app.updater()?.check().await
        });
        match result {
            Ok(Some(update)) => {
                crate::applog(&format!("updater: {} is available", update.version));
                set(&app, UpdateState { available: Some(update.version.clone()), ..Default::default() });
            }
            Ok(None) => {
                crate::applog("updater: this is the newest release");
                set(&app, UpdateState::default());
            }
            // Never a dialogue and never a badge: a check that could not be made says nothing
            // about whether an update exists, and the app it is running in works perfectly well.
            Err(e) => {
                crate::applog(&format!("updater: check failed ({e})"));
                set(&app, UpdateState { message: Some("Could not check for updates".into()), ..Default::default() });
            }
        }
    });
}

/// Downloads the newer installer and runs it. The app is replaced and restarted by NSIS.
#[tauri::command]
pub fn install_update(app: AppHandle) {
    if !configured(&app) {
        return;
    }
    if state().as_ref().is_some_and(|s| s.installing) {
        return;
    }
    set(&app, UpdateState { installing: true, ..Default::default() });
    std::thread::spawn(move || {
        let outcome = tauri::async_runtime::block_on(async {
            let Some(update) = app.updater()?.check().await? else {
                return Ok::<bool, tauri_plugin_updater::Error>(false);
            };
            // The signature is checked against the public key in tauri.conf.json before a
            // single byte is run: an unsigned or altered installer never reaches the disk.
            update.download_and_install(|_, _| {}, || {}).await?;
            Ok(true)
        });
        match outcome {
            Ok(true) => crate::applog("updater: installed, restarting"),
            Ok(false) => set(&app, UpdateState::default()),
            Err(e) => {
                crate::applog(&format!("updater: install failed ({e})"));
                set(&app, UpdateState {
                    available: state().as_ref().and_then(|s| s.available.clone()),
                    message: Some("Could not install the update".into()),
                    ..Default::default()
                });
            }
        }
    });
}

/// One check shortly after launch.
///
/// Delayed rather than immediate: the first seconds belong to reading usage and drawing the
/// notch, and an update that has waited since the last release can wait twenty more seconds.
pub fn check_on_launch(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(20));
        check_for_update(app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn needs_both_a_feed_and_a_key() {
        assert!(!configured_in(None));
        assert!(!configured_in(Some(&json!({"endpoints": [], "pubkey": "abc"}))));
        assert!(!configured_in(Some(&json!({"endpoints": ["https://x/latest.json"], "pubkey": UNSET_PUBKEY}))));
        assert!(configured_in(Some(&json!({"endpoints": ["https://x/latest.json"], "pubkey": "abc"}))));
    }

    #[test]
    fn shipped_config_never_points_at_upstream() {
        let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let updater = conf.pointer("/plugins/updater");
        let text = updater.map(|u| u.to_string()).unwrap_or_default();
        assert!(!text.contains("vinzdg"), "the updater must not offer upstream Codenotch builds: {text}");
        assert!(!configured_in(updater), "no Rafiq release feed exists yet: the updater stays off");
    }
}
