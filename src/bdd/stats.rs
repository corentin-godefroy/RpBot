use std::cmp::Ordering;
use std::collections::HashMap;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document, to_document};
use mongodb::options::{FindOneAndDeleteOptions, FindOneAndUpdateOptions, ReturnDocument};
use mongodb::results::UpdateResult;
use serde::{Deserialize, Serialize};
use serenity::model::application::component::InputText;
use crate::bdd::global::get_collection;
use crate::bdd::player::{get_player_doc, Player};
use crate::common_functions::get_timestamp;
use crate::constants::{END_EFFECT_STAT_TIMESTAMP, PLAYER_COLLECTION, PLAYER_ID, UNIVERSE_ID};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UniverseStats {
    #[serde(rename = "_id")]
    pub id : ObjectId,
    pub universe_id : ObjectId,
    pub universal_stats : Vec<Stats>
}

impl UniverseStats {
    pub fn to_doc(&self) -> Document {
        to_document(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Stats{
    pub name : String,
    pub base_value : f64,
    pub modifiers : Vec<StatModifier>,
    pub hide : bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatModifier{
    pub name : String,
    pub modifier : f64,
    pub end_modifier_timestamp : u64
}

impl Stats {
    pub fn to_doc(&self) -> Document {
        to_document(self).unwrap()
    }
}

impl PartialEq for StatModifier {
    fn eq(&self, other: &Self) -> bool {
        self.end_modifier_timestamp == other.end_modifier_timestamp
    }
}
impl PartialOrd for StatModifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.end_modifier_timestamp.partial_cmp(&other.end_modifier_timestamp)
    }
}

impl Eq for StatModifier {}
impl Ord for StatModifier{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.end_modifier_timestamp.cmp(&other.end_modifier_timestamp)
    }
}

pub async fn update_modifiers(player_id : u64, universe_id : ObjectId) -> Player {
    let timestamp = get_timestamp();
    let collection = get_collection(PLAYER_COLLECTION).clone_with_type();
    collection.find_one_and_update(
        doc! {
                PLAYER_ID : player_id as i64,
                UNIVERSE_ID : universe_id
            },
        doc! {
            "$pull": doc!{
                "stats.$[].modifiers": {
                    END_EFFECT_STAT_TIMESTAMP : doc!{ "$lt": timestamp as i64 }
                }
            }
        },
    FindOneAndUpdateOptions::builder().return_document(ReturnDocument::After).build()
    ).await.unwrap().unwrap()
}

pub fn parse_stats(stats: Vec<Stats>) -> HashMap<String, Stats> {
    let mut parsed_stats: HashMap<String, Stats> = HashMap::new();

    for stat in stats {
        parsed_stats.insert(stat.name.clone(), stat.clone());
    }
    parsed_stats
}

pub fn stat_input_to_hash(stats_values: &InputText) -> HashMap<String, Stats> {
    let mut stat_str = stats_values.value.as_str().clone();
    let mut map = HashMap::new();
    let stats_lines= stat_str.split("\n").collect::<Vec<&str>>();
    for stat_line in stats_lines{
        let mut stat_line = stat_line.clone().replace(" ", "");
        let stat = stat_line.split(":").collect::<Vec<&str>>();
        let stat_name = stat[0].clone();
        let stat_value = stat[1].clone();
        let stat = Stats{
            name: stat_name.to_string(),
            base_value: stat_value.parse().unwrap(),
            modifiers: Vec::new(),
            hide: false,
        };
        map.insert(stat_name.to_string().clone(), stat);
    }
    map
}
