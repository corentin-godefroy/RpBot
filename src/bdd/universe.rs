use mongodb::bson::{doc, Document, to_document};
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use crate::bdd::global::get_collection;
use crate::constants::UNIVERSE_COLLECTION;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Universe{
    #[serde(rename = "_id")]
    pub universe_id : ObjectId,
    pub name : String,
    pub time_modifier : u64,
    pub creator : u64,
    pub default_locale : String
}

impl Universe{
    pub fn to_doc(&self) -> Document {
        to_document(self).unwrap()
    }
}

pub async fn get_universe_doc(universe_id : &ObjectId) -> Option<Universe> {
    let collection : Collection<Universe> = get_collection(UNIVERSE_COLLECTION).clone_with_type();
    collection.find_one(doc!{"_id" : universe_id}, None).await.unwrap()
}