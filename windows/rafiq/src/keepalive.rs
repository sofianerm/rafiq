//! Garde le notch à l'écran.
//!
//! Le symptôme (« Rafiq est ouvert mais je ne le vois plus », aléatoire) : run.log montre le notch
//! bien placé sur son écran pendant qu'il est invisible, et des relances refusées par l'instance
//! unique. Ce n'est donc pas la position, mais l'ordre des fenêtres ou la page :
//! - le notch perd sa place en tête des fenêtres « topmost » (barre des tâches cliquée, app plein
//!   écran, Explorer relancé, retour de veille) et passe derrière d'autres fenêtres ;
//! - la WebView2 cesse de dessiner (processus de rendu planté) : la fenêtre est transparente, une page
//!   morte = un notch invisible ;
//! - un changement d'écran, de résolution ou d'échelle (écran externe, bureau à distance, veille)
//!   que le relevé de la zone de travail ne voit pas.
//!
//! D'où une boucle toutes les 5 s qui réaffirme le topmost, replace le notch quand les écrans
//! changent, qu'il est hors écran ou pas là où on l'a mis, après une veille, et recharge la page si
//! elle ne donne plus signe de vie. `reshow` fait tout d'un coup : menu « Réafficher le notch »,
//! relance de Rafiq alors qu'il tourne déjà.

use crate::{applog, AppState};
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

const CHECK_MS: u64 = 5_000;
/// Le topmost est réaffirmé une boucle sur trois (15 s).
const TOPMOST_EVERY: u32 = 3;
/// Une boucle qui a duré bien plus que prévu : la machine était en veille.
const SLEEP_GAP_MS: u64 = 30_000;
/// Sans battement de la page depuis ce délai, elle est considérée morte. Large : une WebView
/// masquée ou couverte peut ralentir ses minuteries jusqu'à une par minute.
const BEAT_STALE_MS: u64 = 180_000;
/// Pas plus d'un rechargement par période, au cas où la page ne repartirait pas.
const RELOAD_COOLDOWN_MS: u64 = 600_000;

/// Dernier battement de la page (ms epoch), 0 tant qu'elle n'en a envoyé aucun.
static LAST_BEAT: AtomicU64 = AtomicU64::new(0);
static LAST_RELOAD: AtomicU64 = AtomicU64::new(0);
/// Où `place_notch` a mis la fenêtre en dernier (x, y, largeur, hauteur, en pixels physiques).
pub static PLACED: Mutex<Option<(i32, i32, i32, i32)>> = Mutex::new(None);

/// La page dit qu'elle vit (toutes les 10 s, notch.html).
#[tauri::command]
pub fn notch_alive() {
    LAST_BEAT.store(crate::now_ms(), SeqCst);
}

fn notch_wanted(app: &AppHandle) -> bool {
    app.state::<AppState>().cfg.lock().unwrap().notch_visible
}

/// Une app plein écran (jeu, vidéo, présentation) : on ne repasse pas par-dessus.
#[cfg(windows)]
fn fullscreen_app() -> bool {
    use windows::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
    };
    match unsafe { SHQueryUserNotificationState() } {
        Ok(s) => s == QUNS_BUSY || s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE,
        Err(_) => false,
    }
}
#[cfg(not(windows))]
fn fullscreen_app() -> bool {
    false
}

/// Remet le notch en tête des fenêtres topmost, sans l'activer ni le déplacer. `set_always_on_top`
/// ne suffit pas sous Windows : déjà topmost, il ne refait rien (seul un changement de style est
/// appliqué), alors que c'est le rang dans la pile qui a été perdu.
pub fn raise(w: &tauri::WebviewWindow) {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
        };
        // Tauri a sa propre version de la crate windows : le HWND passe par son entier
        if let Ok(h) = w.hwnd() {
            let h = windows::Win32::Foundation::HWND(h.0 as isize as _);
            let _ = unsafe {
                SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER)
            };
        }
    }
    #[cfg(not(windows))]
    {
        let _ = w.set_always_on_top(true);
    }
}

/// Recharge la page du notch (WebView figée ou plantée).
fn reload(app: &AppHandle, why: &str) {
    let now = crate::now_ms();
    let last = LAST_RELOAD.load(SeqCst);
    if last != 0 && now.saturating_sub(last) < RELOAD_COOLDOWN_MS {
        return;
    }
    let Some(w) = app.get_webview_window("notch") else { return };
    LAST_RELOAD.store(now, SeqCst);
    // La page a le temps de repartir avant d'être jugée à nouveau
    LAST_BEAT.store(now, SeqCst);
    match w.reload() {
        Ok(()) => applog(&format!("keepalive: notch page reloaded ({why})")),
        Err(e) => applog(&format!("keepalive: notch page reload failed ({why}): {e}")),
    }
}

/// Signature des écrans : un écran branché, débranché, une résolution ou une échelle changée.
fn screens_key(app: &AppHandle) -> Vec<(Option<String>, i32, i32, i32, i32, u64)> {
    crate::screens(app)
        .into_iter()
        .map(|s| (s.name, s.x, s.y, s.w, s.h, (s.scale * 1000.0).round() as u64))
        .collect()
}

/// Le centre de la fenêtre est-il sur un écran ?
pub(crate) fn on_some_screen(list: &[crate::Screen], x: i32, y: i32, w: i32, h: i32) -> bool {
    let (cx, cy) = (x + w / 2, y + h / 2);
    list.iter().any(|s| cx >= s.x && cx < s.x + s.w && cy >= s.y && cy < s.y + s.h)
}

/// Pourquoi le notch n'est pas où il devrait, s'il ne l'est pas.
fn misplaced(app: &AppHandle, w: &tauri::WebviewWindow) -> Option<String> {
    let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) else { return None };
    let (x, y, ww, wh) = (pos.x, pos.y, size.width as i32, size.height as i32);
    if !on_some_screen(&crate::screens(app), x, y, ww, wh) {
        return Some(format!("off screen at ({x},{y}) {ww}x{wh}"));
    }
    match *PLACED.lock().unwrap() {
        Some(p) if (p.0 - x).abs() > 2 || (p.1 - y).abs() > 2 || (p.2 - ww).abs() > 2 || (p.3 - wh).abs() > 2 => {
            Some(format!("moved to ({x},{y}) {ww}x{wh}, placed at ({},{}) {}x{}", p.0, p.1, p.2, p.3))
        }
        _ => None,
    }
}

/// Tout remettre d'un coup : afficher, replacer, repasser devant, recharger la page si elle ne
/// répond plus. Pour le menu « Réafficher le notch » et une relance de Rafiq.
pub fn reshow(app: &AppHandle, why: &str) {
    {
        let st = app.state::<AppState>();
        let mut c = st.cfg.lock().unwrap();
        if !c.notch_visible {
            c.notch_visible = true;
            crate::config::save(&c);
        }
    }
    let Some(w) = app.get_webview_window("notch") else { return };
    applog(&format!(
        "keepalive: reshow ({why}) visible={:?} pos={:?} size={:?} last_beat_ago_ms={}",
        w.is_visible().ok(),
        w.outer_position().ok().map(|p| (p.x, p.y)),
        w.outer_size().ok().map(|s| (s.width, s.height)),
        crate::now_ms().saturating_sub(LAST_BEAT.load(SeqCst))
    ));
    crate::apply_visibility(app);
    raise(&w);
    // Une page restée vide après un passage d'écran (landing) se réaffiche
    if crate::LANDING.load(SeqCst) == 0 {
        let _ = app.emit_to("notch", "notch_reveal", ());
    }
    let beat = LAST_BEAT.load(SeqCst);
    if beat != 0 && crate::now_ms().saturating_sub(beat) > 25_000 {
        LAST_RELOAD.store(0, SeqCst);
        reload(app, "reshow, page silent");
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut screens = screens_key(&app);
        let mut tick: u32 = 0;
        let mut last = std::time::Instant::now();
        let started = crate::now_ms();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(CHECK_MS));
            let gap = last.elapsed().as_millis() as u64;
            last = std::time::Instant::now();
            tick = tick.wrapping_add(1);
            if !notch_wanted(&app) || crate::DRAGGING.load(SeqCst) {
                continue;
            }
            let Some(w) = app.get_webview_window("notch") else { continue };

            let woke = gap > CHECK_MS + SLEEP_GAP_MS;
            if woke {
                applog(&format!("keepalive: woke up after {} s, placing the notch again", gap / 1000));
                // Les minuteries de la page étaient arrêtées elles aussi
                LAST_BEAT.store(crate::now_ms(), SeqCst);
            }
            let now_screens = screens_key(&app);
            let changed = now_screens != screens;
            if changed {
                applog(&format!("keepalive: screens changed: {screens:?} -> {now_screens:?}"));
                screens = now_screens;
            }
            if w.is_visible().ok() == Some(false) {
                applog("keepalive: notch window was hidden, showing it again");
                let _ = w.show();
            }
            let wrong = if crate::LANDING.load(SeqCst) == 0 { misplaced(&app, &w) } else { None };
            if let Some(why) = &wrong {
                applog(&format!("keepalive: notch {why}, placing it again"));
            }
            if woke || changed || wrong.is_some() {
                crate::place_notch(&app);
                raise(&w);
            } else if tick % TOPMOST_EVERY == 0 && !fullscreen_app() {
                raise(&w);
            }

            let beat = LAST_BEAT.load(SeqCst);
            let now = crate::now_ms();
            if beat != 0 && now.saturating_sub(beat) > BEAT_STALE_MS && now.saturating_sub(started) > BEAT_STALE_MS {
                reload(&app, &format!("no heartbeat for {} s", now.saturating_sub(beat) / 1000));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::on_some_screen;
    use crate::Screen;

    fn screen(x: i32, y: i32, w: i32, h: i32) -> Screen {
        Screen { name: Some("s".into()), x, y, w, h, scale: 1.0, work: (x, y, w, h) }
    }

    #[test]
    fn a_notch_whose_centre_left_every_screen_is_off_screen() {
        let list = [screen(0, 0, 1920, 1200)];
        assert!(on_some_screen(&list, 0, 76, 360, 1000));
        assert!(on_some_screen(&list, 1560, 76, 360, 1000));
        // Écran externe débranché : la fenêtre est restée à sa droite
        assert!(!on_some_screen(&list, 2400, 76, 360, 1000));
        assert!(!on_some_screen(&list, -32000, -32000, 360, 1000));
    }

    #[test]
    fn a_second_screen_counts() {
        let list = [screen(0, 0, 1920, 1200), screen(1920, 0, 2560, 1440)];
        assert!(on_some_screen(&list, 3000, 100, 360, 1000));
    }
}
