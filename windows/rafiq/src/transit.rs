//! Transports en commun partout (ou presque) : l'API MOTIS 2 de Transitous, sans clé.
//!
//! Transitous est tenu par des bénévoles pour les apps libres et non commerciales. Ses conditions
//! (https://transitous.org/api/) : un User-Agent avec le nom de l'app, sa version et un contact,
//! un lien visible vers https://transitous.org/sources/ et l'attribution OpenStreetMap (la card
//! du module et les Settings les montrent), et peu de requêtes. D'où les appels faits ici plutôt
//! que depuis la page (un navigateur ne laisse pas choisir le User-Agent), et un cache par arrêt.
//!
//! transport.opendata.ch (Suisse) reste l'autre source ; le module le lit directement.

use crate::config::TransitStop;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const API: &str = "https://api.transitous.org/api/";
const USER_AGENT: &str = concat!("Rafiq/", env!("CARGO_PKG_VERSION"), " (+https://github.com/sofianerm/rafiq)");
/// Un même arrêt n'est pas redemandé avant ce délai, quel que soit le nombre de fenêtres qui le lisent
const CACHE: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Departure {
    pub line: String,
    /// #rrggbb quand la source donne la couleur de la ligne, sinon vide
    pub color: String,
    /// BUS, TRAM, SUBWAY, REGIONAL_RAIL…
    pub mode: String,
    pub to: String,
    /// Heure réelle (prévue + retard), en ms depuis 1970
    pub at: i64,
    pub delay: i64,
    pub track: String,
    pub cancelled: bool,
}

static DEPARTURES: Mutex<Option<HashMap<String, (Instant, Vec<Departure>)>>> = Mutex::new(None);

fn get(path: &str, query: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build();
    let mut req = agent.get(&format!("{API}{path}")).set("User-Agent", USER_AGENT);
    for (k, v) in query {
        req = req.query(k, v);
    }
    req.call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("HTTP {code}"),
            other => other.to_string(),
        })?
        .into_json()
        .map_err(|e| e.to_string())
}

fn ms(v: &serde_json::Value) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(v.as_str()?).ok().map(|d| d.timestamp_millis())
}

fn parse_stops(v: &serde_json::Value) -> Vec<TransitStop> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["type"] == "STOP")
        .filter_map(|m| {
            let id = m["id"].as_str()?.to_string();
            let name = m["name"].as_str()?.to_string();
            // La commune (zone « default ») puis le pays : de quoi distinguer deux arrêts homonymes
            let area = m["areas"]
                .as_array()
                .and_then(|a| a.iter().find(|x| x["default"] == true))
                .and_then(|x| x["name"].as_str())
                .unwrap_or("");
            let country = m["country"].as_str().unwrap_or("");
            let sub = [area, country].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" · ");
            Some(TransitStop { id, name, sub })
        })
        .take(8)
        .collect()
}

fn parse_departures(v: &serde_json::Value) -> Vec<Departure> {
    let mut out: Vec<Departure> = v["stopTimes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let p = &s["place"];
            let at = ms(&p["departure"]).or_else(|| ms(&p["scheduledDeparture"]))?;
            let planned = ms(&p["scheduledDeparture"]).unwrap_or(at);
            let line = [&s["routeShortName"], &s["displayName"], &s["tripShortName"]]
                .iter()
                .filter_map(|x| x.as_str())
                .find(|x| !x.is_empty())
                .unwrap_or("")
                .to_string();
            let to = [&s["headsign"], &s["tripTo"]["name"]]
                .iter()
                .filter_map(|x| x.as_str())
                .find(|x| !x.is_empty())
                .unwrap_or("")
                .to_string();
            let color = s["routeColor"].as_str().filter(|c| c.len() == 6).map(|c| format!("#{c}")).unwrap_or_default();
            Some(Departure {
                line,
                color,
                mode: s["mode"].as_str().unwrap_or("").to_string(),
                to,
                at,
                delay: (at - planned) / 60_000,
                track: p["track"].as_str().or_else(|| p["scheduledTrack"].as_str()).unwrap_or("").to_string(),
                cancelled: s["cancelled"] == true || s["tripCancelled"] == true,
            })
        })
        .collect();
    out.sort_by_key(|d| d.at);
    out
}

/// Recherche d'arrêts par nom, pour les Settings.
#[tauri::command]
pub async fn transit_search(text: String) -> Result<Vec<TransitStop>, String> {
    let text = text.trim().to_string();
    if text.chars().count() < 2 {
        return Ok(Vec::new());
    }
    tauri::async_runtime::spawn_blocking(move || get("v1/geocode", &[("text", &text), ("type", "STOP")]).map(|v| parse_stops(&v)))
        .await
        .map_err(|e| e.to_string())?
}

/// Les prochains départs d'un arrêt, gardés 45 s.
#[tauri::command]
pub async fn transit_departures(stop_id: String) -> Result<Vec<Departure>, String> {
    if let Some((at, deps)) = DEPARTURES.lock().unwrap().get_or_insert_with(HashMap::new).get(&stop_id) {
        if at.elapsed() < CACHE {
            return Ok(deps.clone());
        }
    }
    let id = stop_id.clone();
    let deps = tauri::async_runtime::spawn_blocking(move || get("v5/stoptimes", &[("stopId", &id), ("n", "20")]).map(|v| parse_departures(&v)))
        .await
        .map_err(|e| e.to_string())??;
    let mut cache = DEPARTURES.lock().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);
    map.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(600));
    map.insert(stop_id, (Instant::now(), deps.clone()));
    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Réseau : `cargo test -- --ignored live_transitous`
    #[test]
    #[ignore]
    fn live_transitous() {
        let stops = parse_stops(&get("v1/geocode", &[("text", "Zürich HB"), ("type", "STOP")]).unwrap());
        assert!(!stops.is_empty());
        let deps = parse_departures(&get("v5/stoptimes", &[("stopId", &stops[0].id), ("n", "5")]).unwrap());
        assert!(!deps.is_empty(), "no departures for {:?}", stops[0]);
        eprintln!("{:?}\n{:?}", stops[0], deps[0]);
    }

    #[test]
    fn user_agent_names_the_app_its_version_and_a_contact() {
        assert!(USER_AGENT.starts_with("Rafiq/"));
        assert!(USER_AGENT.contains(env!("CARGO_PKG_VERSION")) && USER_AGENT.contains("https://"));
    }

    #[test]
    fn stops_keep_their_town_and_country() {
        let v = json!([
            {"type":"STOP","name":"Bern","id":"x:1","country":"CH",
             "areas":[{"name":"Bern","default":false},{"name":"Bern-Mittelland","default":true}]},
            {"type":"ADDRESS","name":"Rue","id":"a"}
        ]);
        let s = parse_stops(&v);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].id.as_str(), s[0].sub.as_str()), ("x:1", "Bern-Mittelland · CH"));
    }

    #[test]
    fn departures_take_the_real_time_and_the_delay() {
        let v = json!({"stopTimes":[
            {"place":{"departure":"2026-10-26T12:48:00Z","scheduledDeparture":"2026-10-26T12:45:00Z","track":"H"},
             "mode":"BUS","headsign":"Jardin Botanique","routeShortName":"22","routeColor":"0078BF"},
            {"place":{"departure":"2026-10-26T12:40:00Z","scheduledDeparture":"2026-10-26T12:40:00Z","scheduledTrack":"2"},
             "mode":"REGIONAL_RAIL","headsign":"","tripTo":{"name":"Annemasse"},"routeShortName":"","displayName":"L1"}
        ]});
        let d = parse_departures(&v);
        assert_eq!(d[0].line, "L1");
        assert_eq!((d[0].to.as_str(), d[0].track.as_str()), ("Annemasse", "2"));
        assert_eq!((d[1].line.as_str(), d[1].delay, d[1].color.as_str()), ("22", 3, "#0078BF"));
    }
}
