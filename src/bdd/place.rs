use std::collections::HashMap;
use mongodb::bson::{doc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use crate::bdd::global::get_collection;
use crate::constants::{PLACE_ID, PLACES_COLLECTION};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Place{
    #[serde(rename = "_id")]
    file_id : ObjectId,
    pub place_id : u64,
    pub server_id : u64,
    pub universe_id : ObjectId,
    pub collectables : HashMap<String, Vec<String>>,
    pub collectables_delay : u64,
    pub delay_before_recollect : u64,
    pub timestamp_before_recollect : u64,
    pub default_channel_id : u64,
    pub role_id : u64,
}

pub async fn get_place(place_id : u64) -> Option<Place> {
    let collection = get_collection(PLACES_COLLECTION).clone_with_type();
    collection.find_one(doc!{PLACE_ID : place_id as i64}, None).await.unwrap()
}
///Returns place doc if given channel id is a place. None else.
pub async fn channel_is_place(ctx : &Context, channel_id : u64) -> Option<Place> {
    let channel = match ctx.http.get_channel(channel_id).await {
        Ok(channel) => {channel}
        Err(_) => {return None}
    };
    let parent_id = match channel.guild().unwrap().parent_id {
        None => {return None}
        Some(parent_id) => {parent_id}
    };

    get_place(parent_id.0).await
}

pub async fn get_server_id_from_place(place_id : u64) -> u64 {
    let place = get_place(place_id).await.unwrap();
    place.server_id
}

