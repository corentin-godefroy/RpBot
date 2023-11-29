use std::collections::HashMap;
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, Cursor};
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::futures::{SinkExt, StreamExt};
use serenity::model::channel::Channel;
use serenity::model::id::ChannelId;
use crate::bdd::global::get_collection;
use crate::bdd::place::Place;
use crate::constants::{ROAD_CATEGORY_ID, PLACES_COLLECTION, ROADS_COLLECTION, ROAD_ID, PLACE_ID, PLACE_1, PLACE_2, ROLE_ID};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Road{
    #[serde(rename = "_id")]
    pub file_id : ObjectId,
    pub road_id : u64,
    pub role_id : u64,
    pub collectables : HashMap<String, Vec<String>>, // item, quantité, rareté
    pub collectables_delay : u64,
    pub delay_before_recollect : u64,
    pub timestamp_before_recollect : u64,
    pub distance : u64,
    pub place_1 : u64,
    pub place_2 : u64,
    pub is_secret: bool,
    pub server_id : u64,
    pub universe_id : ObjectId,
    pub privileges : Vec<u64>
}

///Returns place doc if given channel id is a place. None else.
pub async fn channel_is_road(channel_id : u64) -> Option<Road> {
    let collection = get_collection(ROADS_COLLECTION).clone_with_type();
    match collection.find_one(doc!{ROAD_ID : channel_id as i64}, None).await.unwrap() {
        None => {None}
        Some(doc) => {Some(doc)}
    }
}

pub async fn get_roads_from_place(place_id : u64) -> Vec<Road>{
    let collection = get_collection(ROADS_COLLECTION).clone_with_type();
    let mut cursor = collection.find(doc!{
        "$or" : [
            {PLACE_1 : place_id as i64},
            {PLACE_2 : place_id as i64}
        ]
    }, None).await.unwrap();

    let mut roads = Vec::new();
    while let Some(doc) = cursor.next().await {
        roads.push(doc.unwrap());
    }
    roads
}

pub async fn get_road_from_places(place_id_1 : u64, place_id_2 : u64) -> Option<Road> {
    let collection : Collection<Road> = get_collection(ROADS_COLLECTION).clone_with_type();
    let mut cursor = collection.find(doc!{
        "$or" : [
            {
                PLACE_1 : place_id_1 as i64,
                PLACE_2 : place_id_2 as i64
            },
            {
                PLACE_1 : place_id_2 as i64,
                PLACE_2 : place_id_1 as i64
            }
        ]
    }, None).await.unwrap();

    let mut roads = Vec::new();
    while let Some(doc) = cursor.next().await {
        roads.push(doc.unwrap());
    }
    if roads.len() != 1 {
        return None
    }
    Some(roads[0].clone())
}

pub async fn get_road_by_id(road_id : u64) -> Option<Document> {
    let collection = get_collection(ROADS_COLLECTION);
    let road = collection.find_one(
        doc!{
            ROAD_ID : road_id as i64
        }
    , None).await.unwrap();
    road
}

pub async fn get_road_destinations(road_id: u64) -> Vec<Document> {
    let collection = get_collection(ROADS_COLLECTION);

    let mut cursor = collection
        .aggregate(
            vec![
                doc! {
                    "$match": {
                        ROAD_ID: road_id as i64
                    }
                },
                doc! {
                    "$lookup": {
                        "from": PLACES_COLLECTION,
                        "localField": PLACE_1,
                        "foreignField": PLACE_ID,
                        "as": "place1"
                    }
                },
                doc! {
                    "$lookup": {
                        "from": PLACES_COLLECTION,
                        "localField": PLACE_2,
                        "foreignField": PLACE_ID,
                        "as": "place2"
                    }
                },
                doc! {
                    "$project": {
                        "place1": { "$arrayElemAt": ["$place1", 0] },
                        "place2": { "$arrayElemAt": ["$place2", 0] }
                    }
                },
            ],
            None,
        )
        .await
        .unwrap();

    let mut destinations = Vec::new();
    while let Some(doc) = cursor.next().await {
        let doc = doc.unwrap();
        let place1 = doc.get_document("place1").unwrap().clone();
        let place2 = doc.get_document("place2").unwrap().clone();
        destinations.push(place1);
        destinations.push(place2);
    }
    destinations
}

pub async fn get_road_from_role(role_id : u64) -> Road{
    let collection = get_collection(ROADS_COLLECTION).clone_with_type();
    let mut cursor = collection.find(doc!{
        "$match" : [
            {ROLE_ID : role_id as i64}
        ]
    }, None).await.unwrap();

    let mut roads: Vec<Road> = Vec::new();
    while let Some(road) = cursor.next().await {
        roads.push(road.unwrap());
    }
    if roads.len() != 1{
        //ERREUR
    }
    roads.get(0).unwrap().clone()
}

pub async fn get_road_destination_from_place(place_id : u64) -> Vec<u64> {
    let collection = get_collection(ROADS_COLLECTION);

    let mut cursor : Cursor<Document> = collection
        .aggregate([
        doc! {
            "$match": doc! {
                "$or": [
                    doc! {
                        PLACE_1: place_id as i64
                    },
                    doc! {
                        PLACE_2: place_id as i64
                    }
                ]
            }
        },
        doc! {
            "$lookup": doc! {
                "from": PLACES_COLLECTION,
                "localField": PLACE_1,
                "foreignField": PLACE_ID,
                "as": PLACE_1
            }
        },
        doc! {
            "$lookup": doc! {
                "from": PLACES_COLLECTION,
                "localField": PLACE_2,
                "foreignField": PLACE_ID,
                "as": PLACE_2
            }
        },
        doc! {
            "$addFields": doc! {
                "places": doc! {
                    "$filter": doc! {
                        "input": doc! {
                            "$concatArrays": [
                                doc! {
                                    "$ifNull": [
                                        "$place_1.place_id", [place_id as i64]
                                    ]
                                },
                                doc! {
                                    "$ifNull": [
                                        "$place_2.place_id", [place_id as i64]
                                    ]
                                }
                            ]
                        },
                        "as": "place",
                        "cond": doc! {
                            "$ne": ["$$place", place_id as i64]
                        }
                    }
                }
            }
        },
        doc! {
            "$project": doc! {
                "places": 1
            }
        }], None)
        .await
        .unwrap()
        .with_type();

    let doc = cursor.next().await.unwrap().unwrap();
    let dests = doc.get("places").unwrap().as_array().unwrap().clone();
    let mut destinations = Vec::new();
    for dest in dests {
        let dest = dest.as_i64().unwrap() as u64;
        destinations.push(dest);
    }
    destinations
}
