use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The Mac's notch sizes, as multiples of the designed size: Small, Medium, Large.
pub const SIZES: [f64; 3] = [0.8, 1.0, 1.25];

/// The nearest of `SIZES`, so a scale saved by the old 40–100 % slider still lands on a size that
/// exists. 0.9, halfway between Small and Medium, counts as Medium.
pub fn snap_scale(scale: f64) -> f64 {
    if scale < 0.9 {
        SIZES[0]
    } else if scale < 1.125 || !scale.is_finite() {
        SIZES[1]
    } else {
        SIZES[2]
    }
}

/// One ring on the notch: which provider. (A `window` key from older builds is ignored.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraySlot {
    pub provider: String,
}

/// Un module du notch (prière, météo…) et s'il est affiché. La liste donne l'ordre des cellules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSlot {
    pub id: String,
    #[serde(default = "yes")]
    pub on: bool,
}

/// La ville choisie à l'accueil (géocodage Open-Meteo) : météo et horaires de prière calculés.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct City {
    pub name: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub timezone: String,
}

/// Un arrêt du module transports, dans l'identifiant de sa source (`transit_source`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitStop {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    /// "auto" (la langue de Windows), "fr" ou "en"
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub bar_x: Option<i32>,
    #[serde(default)]
    pub bar_y: Option<i32>,
    /// Logical width of the bar (wheel-adjustable, 220-520); None = default 360
    #[serde(default)]
    pub bar_w: Option<u32>,
    /// Allow dragging + wheel resizing (tray toggle, off by default to prevent accidental drags)
    #[serde(default)]
    pub drag_enabled: bool,
    /// The one position every edge used to share, read once so `load` can hand it to the edge the
    /// notch is on and never written again. `notch_along` replaces it.
    #[serde(default = "default_notch_y", skip_serializing)]
    pub notch_y: f64,
    /// Where along each edge the notch sits: the window centre as a fraction of that edge's span in
    /// the work area, 0 = top/left, 1 = bottom/right, 0.5 = centred (the default for an edge with
    /// no entry). One per edge, as the Mac keeps one offset per edge: sliding it along the right
    /// edge should not also move it on the top.
    #[serde(default)]
    pub notch_along: BTreeMap<String, f64>,
    /// Which screen edge the notch is pinned to: "right" (the default), "left", "top" or "bottom".
    #[serde(default = "default_notch_edge")]
    pub notch_edge: String,
    /// Which monitor the notch lives on, by the system's device name (`\\.\DISPLAY2`). None, or a
    /// name no longer attached, means the primary monitor — so unplugging a screen cannot strand it.
    #[serde(default)]
    pub notch_monitor: Option<String>,
    /// Notch size as a multiple of the designed size, one of `SIZES`. The whole notch scales: the
    /// window grows and its WebView zooms, so the rings, text and hover card keep their proportions.
    #[serde(default = "default_scale")]
    pub scale: f64,
    /// Where the weekly limit gets a ring of its own: "off", "inside" or "outside".
    #[serde(default = "default_weekly_ring")]
    pub weekly_ring: String,
    /// Which providers the notch itself shows, in order. Empty means every provider that has
    /// something to report — the original behaviour, and the default. Superseded by `notch_slots`,
    /// kept so an existing config migrates cleanly.
    #[serde(default)]
    pub notch_providers: Vec<String>,
    /// Which providers get a ring on the notch, in order. An empty list means every provider.
    #[serde(default)]
    pub notch_slots: Vec<TraySlot>,
    /// Antigravity's lane on the ring, as the Mac app's "Notch reads": "automatic", "5h" or "weekly"
    #[serde(default = "default_antigravity_limit")]
    pub antigravity_limit: String,
    /// The model family that choice looks at, as the Mac app's "Model data": "gemini" or "3p"
    #[serde(default = "default_antigravity_model")]
    pub antigravity_model: String,
    /// One-shot migration flag: a config saved before the GLM ring existed gets GLM added to the
    /// notch once; an explicit later un-tick is respected and never overridden.
    #[serde(default)]
    pub glm_notch_fixed: bool,
    /// false = the pill is kept off the screen edge entirely; the tray icon is then the only way in
    #[serde(default = "yes")]
    pub notch_visible: bool,
    /// true = the Mac's Show on hover: the notch rests as a small pill at the edge and opens when the
    /// pointer reaches it. Only means anything while `notch_visible` is true. The Mac's default, and a
    /// fresh install's; a config written before this existed keeps the always-open notch it had (see
    /// `load`), so nobody's notch starts folding on an update.
    #[serde(default = "yes")]
    pub notch_on_hover: bool,
    /// false = the tray icon is hidden. Refused while the notch is also hidden, because that would
    /// leave the app running with no way to reach it.
    #[serde(default = "yes")]
    pub tray_visible: bool,
    /// false = no arc above the notch to carry it by. Nothing is lost: Appearance → Edge moves it too.
    #[serde(default = "yes")]
    pub show_move_handle: bool,
    /// The prayer module's mosque on masjidbox.com: its slug, or the whole page URL. Only read when
    /// `prayer_source` is "masjidbox".
    #[serde(default)]
    pub prayer_masjid: String,
    /// D'où viennent les horaires : "aladhan" (calculés pour `city`, sans clé) ou "masjidbox"
    /// (ceux d'une mosquée, `prayer_masjid`).
    #[serde(default = "default_prayer_source")]
    pub prayer_source: String,
    /// La méthode de calcul d'Aladhan (3 = Ligue islamique mondiale, 12 = UOIF…).
    #[serde(default = "default_prayer_method")]
    pub prayer_method: u32,
    /// Silent toasts at each adhan, and five minutes before a prayer's time runs out
    #[serde(default = "yes")]
    pub prayer_notify: bool,
    /// Modules du notch choisis dans les Settings, dans l'ordre. Vide = tous, dans l'ordre des
    /// `<script src>` de notch.html ; un module absent de la liste (ajouté depuis) s'affiche à la fin.
    #[serde(default)]
    pub notch_modules: Vec<ModuleSlot>,
    /// false tant que l'accueil n'a pas été fait : les Settings s'ouvrent dessus au lancement.
    #[serde(default)]
    pub onboarded: bool,
    /// Le prénom donné à l'accueil.
    #[serde(default)]
    pub user_name: String,
    #[serde(default)]
    pub city: Option<City>,
    /// "transitous" (MOTIS, monde entier) ou "opendata" (transport.opendata.ch, Suisse).
    #[serde(default = "default_transit_source")]
    pub transit_source: String,
    #[serde(default)]
    pub transit_stops: Vec<TransitStop>,
}

fn default_prayer_source() -> String {
    "aladhan".into()
}
fn default_prayer_method() -> u32 {
    3
}
fn default_transit_source() -> String {
    "transitous".into()
}


fn default_notch_y() -> f64 {
    0.5
}
fn default_notch_edge() -> String {
    "right".into()
}

/// The four edges, in the order Settings lists them.
pub const EDGES: [&str; 4] = ["left", "right", "top", "bottom"];

/// An unreadable edge means the right-hand one, the layout every earlier build used.
pub fn edge_or_right(value: &str) -> String {
    if EDGES.contains(&value) {
        value.to_string()
    } else {
        "right".into()
    }
}

/// True for the edges the notch stands upright on (the pill is a column); false for top and bottom,
/// where it lies flat (the pill is a row) and the window's width and height swap.
pub fn edge_is_vertical(edge: &str) -> bool {
    matches!(edge, "left" | "right")
}

/// Show on hover is the default for a fresh install, as on the Mac, but a config saved before the
/// setting existed was saved by a notch that was always open, and it stays that way: its owner never
/// chose a folding notch, so an update is not where they should meet one.
fn keep_open_on_upgrade(cfg: &mut Config, raw: Option<&str>) {
    let saved_without_it = raw
        .and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok())
        .is_some_and(|v| v.get("notch_on_hover").is_none());
    if saved_without_it {
        cfg.notch_on_hover = false;
    }
}

/// Migration: the one position every edge used to share becomes the position for the edge the notch
/// was on, so an existing config keeps its place; the other edges start centred. `notch_y` is never
/// written back, so this runs once and a later `load` finds `notch_along` already filled in.
fn carry_shared_position(cfg: &mut Config) {
    if cfg.notch_along.is_empty() && (cfg.notch_y - 0.5).abs() > f64::EPSILON {
        let edge = edge_or_right(&cfg.notch_edge);
        let along = cfg.notch_y;
        cfg.set_along(&edge, along);
    }
}

impl Config {
    /// Where the notch sits along `edge`: centred until it has been slid somewhere on that edge.
    pub fn along(&self, edge: &str) -> f64 {
        self.notch_along.get(edge).copied().unwrap_or(0.5).clamp(0.0, 1.0)
    }
    pub fn set_along(&mut self, edge: &str, along: f64) {
        self.notch_along.insert(edge.to_string(), along.clamp(0.0, 1.0));
    }
}

fn default_scale() -> f64 {
    1.0
}
fn default_weekly_ring() -> String {
    "off".into()
}

/// A second arc changes how every reading looks, so an unreadable value means off rather than a
/// guess at what was meant.
pub fn weekly_ring_or_off(value: &str) -> String {
    match value {
        "inside" | "outside" => value.to_string(),
        _ => default_weekly_ring(),
    }
}
fn yes() -> bool {
    true
}
fn default_antigravity_limit() -> String {
    "automatic".into()
}
fn default_antigravity_model() -> String {
    "gemini".into()
}

fn default_port() -> u16 {
    48666
}
fn default_lang() -> String {
    "auto".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            lang: default_lang(),
            bar_x: None,
            bar_y: None,
            bar_w: None,
            drag_enabled: false,
            notch_y: default_notch_y(),
            notch_along: BTreeMap::new(),
            notch_edge: default_notch_edge(),
            notch_monitor: None,
            scale: default_scale(),
            weekly_ring: default_weekly_ring(),
            notch_providers: Vec::new(), // empty = show them all
            notch_slots: Vec::new(),     // filled in by load(), from notch_providers
            antigravity_limit: default_antigravity_limit(),
            antigravity_model: default_antigravity_model(),
            glm_notch_fixed: true, // a fresh install picks from the full list already
            notch_visible: true,
            notch_on_hover: true,
            tray_visible: true,
            show_move_handle: true,
            prayer_masjid: String::new(),
            prayer_source: default_prayer_source(),
            prayer_method: default_prayer_method(),
            prayer_notify: true,
            notch_modules: Vec::new(),
            onboarded: false,
            user_name: String::new(),
            city: None,
            transit_source: default_transit_source(),
            transit_stops: Vec::new(),
        }
    }
}

/// %APPDATA%\rafiq\config.json ; tous les fichiers de Rafiq vivent à côté. `RAFIQ_DATA_DIR` le
/// remplace par un autre dossier : de quoi essayer une vraie première ouverture (l'accueil) sans
/// toucher à la config de tous les jours.
pub fn config_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("RAFIQ_DATA_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir).join("config.json");
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rafiq")
        .join("config.json")
}

/// Ce que l'installeur perso (feature `perso`) intègre : une config toute faite (presets/sofiane.json,
/// sans clé ni secret : les clés d'API sont dans ai.json, jamais ici). Une copie sans config.json
/// démarre avec, sans accueil. Le fichier reste en local (.gitignore) : la version publique n'a pas
/// la feature, et passe toujours par l'accueil.
#[cfg(feature = "perso")]
const PERSO_PRESET: Option<&str> = Some(include_str!("../presets/sofiane.json"));
#[cfg(not(feature = "perso"))]
const PERSO_PRESET: Option<&str> = None;

fn preset() -> Option<String> {
    PERSO_PRESET.map(str::to_string)
}

pub fn load() -> Config {
    let path = config_path();
    let raw = std::fs::read_to_string(&path).ok().or_else(preset);
    let mut cfg: Config = raw
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    keep_open_on_upgrade(&mut cfg, raw.as_deref());

    // Migration: before slots existed the notch was a plain provider list, one ring each. That is
    // exactly a list of slots, so nobody's choice is lost and nobody has to reconfigure anything.
    if cfg.notch_slots.is_empty() {
        cfg.notch_slots = cfg
            .notch_providers
            .iter()
            .map(|p| TraySlot { provider: p.clone() })
            .collect();
    }

    carry_shared_position(&mut cfg);
    // A selection saved before GLM existed gets the GLM ring back exactly once.
    migrate_glm_notch(&mut cfg, &raw);

    // Both hidden would leave the app unreachable: no pill, no tray icon, no way to open settings.
    if !cfg.notch_visible && !cfg.tray_visible {
        cfg.tray_visible = true;
    }

    // The old slider's 40–100 %, or a hand-edited file, lands on one of the three sizes
    cfg.scale = snap_scale(cfg.scale);
    cfg.weekly_ring = weekly_ring_or_off(&cfg.weekly_ring);
    cfg
}

fn migrate_glm_notch(cfg: &mut Config, raw: &Option<String>) {
    let predates = raw
        .as_deref()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok())
        .map(|v| v.get("glm_notch_fixed").is_none())
        .unwrap_or(false);
    if !predates {
        return;
    }
    if !cfg.notch_slots.is_empty() && !cfg.notch_slots.iter().any(|s| s.provider == "glm") {
        cfg.notch_slots.push(TraySlot { provider: "glm".into() });
    }
    cfg.glm_notch_fixed = true;
}

pub fn save(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(txt) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, txt);
    }
}

#[cfg(test)]
mod tests {
    use super::{carry_shared_position, keep_open_on_upgrade, snap_scale, weekly_ring_or_off, Config};

    /// Le profil de l'installeur perso est fini : pas d'accueil, rien de secret.
    #[cfg(feature = "perso")]
    #[test]
    fn perso_preset_is_a_finished_profile() {
        let raw = super::PERSO_PRESET.unwrap();
        assert!(!raw.to_lowercase().contains("api_key") && !raw.contains("sk-"));
        let cfg: Config = serde_json::from_str(raw).unwrap();
        assert!(cfg.onboarded && !cfg.user_name.is_empty());
        assert!(!cfg.notch_modules.is_empty());
    }

    /// Une installation neuve (pas de fichier, ou un fichier sans `onboarded`) passe par l'accueil,
    /// sans rien de personnel.
    #[test]
    fn fresh_config_goes_through_the_welcome() {
        let cfg = Config::default();
        assert!(!cfg.onboarded);
        assert_eq!((cfg.prayer_source.as_str(), cfg.transit_source.as_str()), ("aladhan", "transitous"));
        assert!(cfg.prayer_masjid.is_empty() && cfg.city.is_none() && cfg.transit_stops.is_empty());
        let cfg: Config = serde_json::from_str(r#"{"lang":"auto"}"#).unwrap();
        assert!(!cfg.onboarded);
    }

    /// Une config d'avant le choix des modules les montre tous ; un module listé sans `on` est affiché.
    #[test]
    fn notch_modules_default_to_all_shown() {
        let old: Config = serde_json::from_str("{}").unwrap();
        assert!(old.notch_modules.is_empty());
        let c: Config =
            serde_json::from_str(r#"{"notch_modules":[{"id":"meteo"},{"id":"tpg","on":false}]}"#).unwrap();
        assert_eq!(c.notch_modules.len(), 2);
        assert!(c.notch_modules[0].on);
        assert!(!c.notch_modules[1].on);
    }

    /// Show on hover is the Mac's default, so a fresh install gets it — but an update must not start
    /// folding a notch whose owner has only ever known it open.
    #[test]
    fn only_a_fresh_install_starts_on_hover() {
        let mut fresh = Config::default();
        keep_open_on_upgrade(&mut fresh, None);
        assert!(fresh.notch_on_hover, "no config file: the Mac's default");

        let mut upgraded = Config::default();
        keep_open_on_upgrade(&mut upgraded, Some(r#"{"notch_visible":true}"#));
        assert!(!upgraded.notch_on_hover, "saved before the setting existed: stays open");

        for chosen in [true, false] {
            let mut c = Config { notch_on_hover: chosen, ..Default::default() };
            keep_open_on_upgrade(&mut c, Some(&format!(r#"{{"notch_on_hover":{chosen}}}"#)));
            assert_eq!(c.notch_on_hover, chosen, "a choice already made is kept");
        }
    }

    /// The Mac keeps one offset per edge; sliding the notch along one must not move it on another.
    #[test]
    fn each_edge_keeps_its_own_place() {
        let mut c = Config::default();
        assert_eq!(c.along("right"), 0.5, "an edge never slid along is centred");
        c.set_along("right", 0.2);
        assert_eq!(c.along("right"), 0.2);
        assert_eq!(c.along("top"), 0.5, "sliding it on the right left the top where it was");
        c.set_along("top", 7.0);
        assert_eq!(c.along("top"), 1.0, "and it can never be put past the end of an edge");
    }

    #[test]
    fn the_shared_position_moves_to_the_edge_the_notch_was_on() {
        let mut c = Config { notch_y: 0.3, notch_edge: "left".into(), ..Default::default() };
        carry_shared_position(&mut c);
        assert_eq!(c.along("left"), 0.3, "an existing config keeps its place");
        assert_eq!(c.along("right"), 0.5, "the edges it was not on start centred");
        // Once carried over, a later load leaves it alone even though notch_y still reads 0.3
        c.set_along("left", 0.8);
        carry_shared_position(&mut c);
        assert_eq!(c.along("left"), 0.8);
        // A centred config has nothing to carry, so nothing is written for it
        let mut centred = Config::default();
        carry_shared_position(&mut centred);
        assert!(centred.notch_along.is_empty());
    }

    #[test]
    fn the_shared_position_is_read_but_never_written_again() {
        let mut v = serde_json::to_value(Config { notch_y: 0.3, ..Default::default() }).unwrap();
        assert!(v.get("notch_y").is_none(), "{v}");
        v["notch_y"] = serde_json::json!(0.3);
        let back: Config = serde_json::from_value(v).unwrap();
        assert_eq!(back.notch_y, 0.3);
    }

    #[test]
    fn a_saved_scale_snaps_to_the_nearest_size() {
        assert_eq!(snap_scale(0.4), 0.8);
        assert_eq!(snap_scale(0.85), 0.8);
        assert_eq!(snap_scale(0.9), 1.0);
        assert_eq!(snap_scale(1.0), 1.0);
        assert_eq!(snap_scale(1.2), 1.25);
        assert_eq!(snap_scale(3.0), 1.25);
    }

    #[test]
    fn only_the_two_placements_are_kept() {
        assert_eq!(weekly_ring_or_off("inside"), "inside");
        assert_eq!(weekly_ring_or_off("outside"), "outside");
        assert_eq!(weekly_ring_or_off("Inside"), "off");
        assert_eq!(weekly_ring_or_off(""), "off");
    }
}
