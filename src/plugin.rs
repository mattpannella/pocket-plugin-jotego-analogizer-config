// The wizard itself: two Choice prompts, then writes pocket/Assets/jtpatreon/common/crtcfg.bin.
// wasm32-only because the extism host imports can't link natively; `cargo test` covers config.rs.

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::config::{self, Config};

const BIN_DIR: &str = "pocket/Assets/jtpatreon/common";
const BIN_FILE: &str = "crtcfg.bin";
const STATE_KEY: &str = "state";

#[derive(Clone, Serialize, ToBytes)]
#[encoding(Json)]
enum PluginMessage {
    Choice { name: String, query: String, choices: Vec<String> },
    #[allow(dead_code)]
    Text { name: String, query: String },
    Exit,
}

#[derive(Clone, Deserialize, FromBytes)]
#[encoding(Json)]
enum HostMessage {
    Answer { name: String, value: String },
    Kill,
}

#[host_fn]
extern "ExtismHost" {
    fn open_url(url: &str) -> ();
    fn print_msg(message: &str) -> ();
}

fn println(msg: &str) {
    unsafe {
        let _ = print_msg(&format!("{msg}\n"));
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct State {
    current: Option<Config>,
    crt: Option<char>,
}

fn load_state() -> State {
    var::get::<Json<State>>(STATE_KEY).ok().flatten().map(|j| j.0).unwrap_or_default()
}

fn save_state(state: &State) {
    let _ = var::set(STATE_KEY, Json(state.clone()));
}

fn label(letter: char, text: &str, current: bool) -> String {
    let letter = letter.to_ascii_uppercase();
    if current {
        format!("{letter}: {text}  (current)")
    } else {
        format!("{letter}: {text}")
    }
}

fn ask(name: &str, query: &str, table: &[(char, &str, u32)], current: Option<char>) -> PluginMessage {
    let choices = table.iter().map(|(letter, text, _)| label(*letter, text, current == Some(*letter))).collect();
    PluginMessage::Choice { name: name.into(), query: query.into(), choices }
}

fn ask_crt(state: &State) -> PluginMessage {
    ask("crt", "Select your analog video output:", config::CRT, state.current.map(|c| c.crt))
}

fn ask_snac(state: &State) -> PluginMessage {
    ask("snac", "Select your SNAC controller:", config::SNAC, state.current.map(|c| c.snac))
}

fn parse_letter(table: &[(char, &str, u32)], value: &str) -> Option<char> {
    let letter = value.trim().chars().next()?.to_ascii_lowercase();
    table.iter().any(|(l, _, _)| *l == letter).then_some(letter)
}

fn read_current() -> Option<Config> {
    fs::read(Path::new(BIN_DIR).join(BIN_FILE)).ok().and_then(|bytes| config::from_bytes(&bytes))
}

fn write_config(c: &Config) -> Result<String, String> {
    let bytes = config::to_bytes(c).ok_or("invalid selection")?;
    let dir = Path::new(BIN_DIR);
    fs::create_dir_all(dir).map_err(|e| format!("could not create {BIN_DIR}: {e}"))?;
    let part = dir.join(format!("{BIN_FILE}.part"));
    let dest = dir.join(BIN_FILE);
    fs::write(&part, bytes).map_err(|e| format!("could not write {}: {e}", part.display()))?;
    fs::rename(&part, &dest).map_err(|e| format!("could not replace {}: {e}", dest.display()))?;
    Ok(format!("Assets/jtpatreon/common/{BIN_FILE}"))
}

#[plugin_fn]
pub fn start() -> FnResult<PluginMessage> {
    println("Jotego Analogizer configurator");
    let state = State { current: read_current(), crt: None };
    match state.current {
        Some(c) => println(&format!("Current crtcfg.bin: video {}, controller {}", c.crt.to_ascii_uppercase(), c.snac.to_ascii_uppercase())),
        None => println("No crtcfg.bin on the card yet (or not one this wizard wrote)."),
    }
    save_state(&state);
    Ok(ask_crt(&state))
}

#[plugin_fn]
pub fn handle_response(input: HostMessage) -> FnResult<PluginMessage> {
    let mut state = load_state();
    Ok(match input {
        HostMessage::Kill => {
            println("Cancelled; nothing written.");
            PluginMessage::Exit
        }
        HostMessage::Answer { name, value } if name == "crt" => match parse_letter(config::CRT, &value) {
            Some(letter) => {
                state.crt = Some(letter);
                save_state(&state);
                ask_snac(&state)
            }
            None => {
                println(&format!("'{value}' is not a valid option; asking again."));
                ask_crt(&state)
            }
        },
        HostMessage::Answer { name, value } if name == "snac" => {
            let Some(crt) = state.crt else {
                println("Missing video selection; starting over.");
                return Ok(ask_crt(&state));
            };
            let Some(snac) = parse_letter(config::SNAC, &value) else {
                println(&format!("'{value}' is not a valid option; asking again."));
                return Ok(ask_snac(&state));
            };
            let c = Config { crt, snac };
            match write_config(&c) {
                Ok(path) => println(&format!(
                    "Jotego Analogizer configuration saved to {path} ({}).",
                    config::to_bytes(&c).map(|b| b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")).unwrap_or_default()
                )),
                Err(e) => println(&format!("Failed: {e}")),
            }
            PluginMessage::Exit
        }
        HostMessage::Answer { name, .. } => {
            println(&format!("Unexpected answer for '{name}'; nothing written."));
            PluginMessage::Exit
        }
    })
}
