//! Rust-side (tray menu) strings. The page has its own dictionary; keys are kept identical on both sides.

use std::sync::atomic::{AtomicBool, Ordering};

/// La langue de l'app côté Rust (notifications, messages de Clippy) : posée au lancement et à chaque
/// changement dans les Settings, avec la langue résolue ("fr" ou "en").
static FR: AtomicBool = AtomicBool::new(true);

pub fn set_lang(resolved: &str) {
    FR.store(resolved == "fr", Ordering::Relaxed);
}

pub fn is_fr() -> bool {
    FR.load(Ordering::Relaxed)
}

/// Le texte de la langue de l'app
pub fn l<'a>(fr: &'a str, en: &'a str) -> &'a str {
    if is_fr() { fr } else { en }
}

pub fn resolve_auto() -> &'static str {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Globalization::GetUserDefaultLocaleName;
        let mut buf = [0u16; 85];
        let n = GetUserDefaultLocaleName(&mut buf);
        if n > 0 {
            let name = String::from_utf16_lossy(&buf[..(n as usize - 1)]).to_lowercase();
            if let Some(lang) = language_from_windows_locale(&name) {
                return lang;
            }
        }
    }
    "en"
}

/// Rafiq parle français ou anglais : une locale française donne "fr", tout le reste l'anglais.
fn language_from_windows_locale(name: &str) -> Option<&'static str> {
    name.to_ascii_lowercase().starts_with("fr").then_some("fr")
}

/// Whether the region settings write times on a 24-hour clock. The page can't tell: WebView2's
/// locale follows the browser language, not the regional format.
pub fn clock_24h() -> bool {
    time_format().is_some_and(|pattern| is_24h_pattern(&pattern))
}

fn time_format() -> Option<String> {
    #[cfg(windows)]
    unsafe {
        use windows::core::PCWSTR;
        use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_SSHORTTIME, LOCALE_STIMEFORMAT};
        // The taskbar clock shows the short time, or the long time once it shows seconds; the two are set separately
        let kind = if taskbar_shows_seconds() { LOCALE_STIMEFORMAT } else { LOCALE_SSHORTTIME };
        let mut buf = [0u16; 80];
        let n = GetLocaleInfoEx(PCWSTR::null(), kind, Some(&mut buf));
        if n > 0 {
            return Some(String::from_utf16_lossy(&buf[..(n as usize - 1)]));
        }
    }
    None
}

#[cfg(windows)]
fn taskbar_shows_seconds() -> bool {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let mut value = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"),
            w!("ShowSecondsInSystemClock"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    status.is_ok() && value != 0
}

/// "HH:mm" against "hh:mm tt"; text between single quotes is literal.
fn is_24h_pattern(pattern: &str) -> bool {
    pattern.split('\'').step_by(2).any(|part| part.contains('H'))
}

pub fn tr(lang: &str, key: &str) -> &'static str {
    let l = if lang == "auto" { resolve_auto() } else { lang };
    match (l, key) {
        ("fr", "open_data") => "Ouvrir le dossier des données (journaux / icônes)",
        ("fr", "install") => "Installer les hooks de Claude Code",
        ("fr", "uninstall") => "Désinstaller les hooks",
        ("fr", "language") => "Langue",
        ("fr", "lang_auto") => "Comme le système",
        ("fr", "reset_pos") => "Replacer la barre",
        ("fr", "quit") => "Quitter",
        ("fr", "hooks_missing") => "Hooks non installés : clic droit sur l'icône → Installer les hooks de Claude Code (l'app de bureau a déjà une solution de secours)",
        ("fr", "autostart") => "Lancer avec Windows (en silence)",
        ("fr", "refresh_all") => "Tout actualiser",
        ("fr", "waiting") => "En attente de la première lecture…",
        ("fr", "quit_app") => "Quitter Rafiq",
        ("fr", "settings") => "Réglages…",
        ("fr", "refresh_now") => "Actualiser",
        ("fr", "open_host") => "Ouvrir %@",
        ("fr", "keep_open") => "Garder ouvert",
        (_, "open_data") => "Open data folder (logs / icons)",
        (_, "install") => "Install Claude Code hooks",
        (_, "uninstall") => "Uninstall hooks",
        (_, "language") => "Language",
        (_, "lang_auto") => "Follow system",
        (_, "reset_pos") => "Reset bar position",
        (_, "quit") => "Quit",
        (_, "hooks_missing") => "Hooks not installed: tray right-click → Install Claude Code hooks (desktop app auto-fallback active)",
        (_, "autostart") => "Start with Windows (silent)",
        (_, "refresh_all") => "Refresh all",
        (_, "waiting") => "Waiting for the first reading…",
        (_, "quit_app") => "Quit Rafiq",
        (_, "settings") => "Settings…",
        (_, "refresh_now") => "Refresh now",
        (_, "open_host") => "Open %@",
        (_, "keep_open") => "Keep open",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::tr;

    const KEYS: [&str; 16] = [
        "open_data", "install", "uninstall", "language", "lang_auto", "reset_pos", "quit", "hooks_missing",
        "autostart", "refresh_all", "waiting", "quit_app", "settings", "refresh_now", "open_host", "keep_open",
    ];

    #[test]
    fn french_translates_every_known_key() {
        for key in KEYS {
            assert_ne!(tr("fr", key), "?", "unknown key {key}");
            assert_ne!(tr("fr", key), tr("en", key), "{key} has no French");
        }
        assert_eq!(tr("fr", "settings"), "Réglages…");
    }

    #[test]
    fn only_french_locales_resolve_to_french() {
        assert_eq!(super::language_from_windows_locale("fr-CH"), Some("fr"));
        assert_eq!(super::language_from_windows_locale("fr-FR"), Some("fr"));
        assert_eq!(super::language_from_windows_locale("de-CH"), None);
        assert_eq!(super::language_from_windows_locale("ko-KR"), None);
    }

    #[test]
    fn unknown_language_keeps_the_english_fallback() {
        assert_eq!(tr("xx", "settings"), "Settings…");
        assert_eq!(tr("ko", "settings"), "Settings…");
    }

    #[test]
    fn the_hour_symbol_outside_quotes_decides_the_clock() {
        assert!(super::is_24h_pattern("HH:mm"));
        assert!(super::is_24h_pattern("H:mm"));
        assert!(super::is_24h_pattern("HH' h 'mm"));
        assert!(!super::is_24h_pattern("hh:mm tt"));
        assert!(!super::is_24h_pattern("tt hh:mm"));
        assert!(!super::is_24h_pattern("h:mm 'Hrs'"));
    }
}
