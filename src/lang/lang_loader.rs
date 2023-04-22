use std::fs::File;
use std::io::Read;
use json::JsonValue;
use std::collections::HashMap;
use crate::common_functions::{log};
use crate::common_functions::LogType::*;
use crate::constants::{DEFAULT_LANG, FR_LANG};
use crate::LANGS;

const FILENAMES: [&str; 2] = [DEFAULT_LANG, FR_LANG];

pub fn lang_loader(langs_map: &mut HashMap<&str, JsonValue>){
    for filename in FILENAMES {
        let mut f = match File::open("src/lang/".to_string() + filename + ".json"){
            Ok(f) => {f}
            Err(_) => {
                println!("Error on oppening file {}", filename);
                continue;
            }
        };

        let mut buf = String::new();
        match f.read_to_string(&mut buf){
            Ok(_) => {}
            Err(_) => {
                println!("Error on reading file {}", filename);
                continue;
            }
        }

        match json::parse(&buf){
            Ok(json_file) => { langs_map.insert(filename, json_file); }
            Err(_) => {println!("Error on parsing file {}", filename)}
        }
    }
}

pub fn get_key(locale: &str, key : &str) -> String {
    match LANGS.get().unwrap().get(locale){
        //TODO changer le retour d'erreur par la langue par défaut et afficher un warning
        None => {
            return match LANGS.get().unwrap().get(DEFAULT_LANG) {
                None => {
                    log(ERROR,
                        format!("Locales \"{locale}\" and \"{DEFAULT_LANG}\" not found.").as_str()
                    );
                    "not_found".to_string()
                }
                Some(default_locale) => {
                    log(WARNING,
                        format!("Locales \"{locale}\" not found. {DEFAULT_LANG} are loaded instead.").as_str()
                    );
                    default_locale[key].to_string()
                }
            }
        }
        Some(locale_found) => {
            let value = locale_found[key].to_string();
            if value == "null" && locale.to_string().eq(DEFAULT_LANG) {
                log(ERROR,
                    format!("Key \"{key}\" for locales \"{locale}\" not found. \"not_found\" is returned").as_str() );
                return "not_found".to_string()
            }
            else if value == "null"{
                log(WARNING,
                    format!("Locale \"{locale}\" not found for \"{key}\". \"{DEFAULT_LANG}\" are loaded instead for this key.").as_str());
                return get_key(DEFAULT_LANG, key);
            }

            return value
        }
    }
}