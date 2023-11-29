use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::OnceCell;
use std::time::{Duration};
use mongodb::bson::{doc};
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use mongodb::results::UpdateResult;
use serenity::client::Context;
use serenity::futures::{AsyncReadExt, TryStreamExt};
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, RoleId};
use tokio::{join, spawn, time};
use tokio::io::AsyncWriteExt;
use crate::bdd::global::get_collection;
use crate::bdd::place::{get_place, get_server_id_from_place};
use crate::bdd::player::{get_player_doc, get_stat_hashmap, Player};
use crate::bdd::road::{channel_is_road, get_road_by_id, get_road_from_places, Road};
use crate::bdd::stats::update_modifiers;
use crate::bdd::universe::get_universe_doc;
use crate::common_functions::{get_parent_category, get_timestamp, get_universe_id, log, ReportType, send_report};
use crate::constants::{PLAYER_COLLECTION, PLAYER_DESTINATION_SERVER_ID, PLAYER_DESTINATION_ID, PLAYER_END_TIMESTAMP, PLAYER_ID, PLAYER_IS_IN_MOVE, PLAYER_START_TIMESTAMP, RPBOT_BDD, UNIVERSE_ID, HANDLER_DELAY, PLACES_COLLECTION, PLACE_ID, ROLE_ID, DEFAULT_CHANNEL_PLACE, PLAYER_POSITION_TIMESTAMP, SERVER_ID, DISTANCE, PLAYER_STATS, SPEED_STAT, MOUNT_SPEED_STAT, TIME_MODIFIER, PLAYER_CURRENT_POSITION_ID, PLAYER_CURRENT_POSITION_SERVER_ID, PLAYER_DESTINATION_ROLE_ID, PLAYER_CURRENT_POSITION_ROLE_ID, ROAD_ID};
pub static MOVE_LIST: OnceCell<Arc<RwLock<BinaryHeap<PlayerMove>>>> = OnceCell::new();

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PlayerMove {
    pub(crate) player_id: u64,
    pub(crate) universe_id: ObjectId,
    pub(crate) is_end: bool,
    pub(crate) step_end_timestamp: u64,
    pub(crate) player : Player
}

impl PlayerMove {
    pub(crate) fn new_from_player_doc(player: Player, step_end_timestamp : u64, is_end : bool) -> Self {
        PlayerMove {
            player_id: player.id,
            universe_id: player.universe_id,
            is_end : is_end,
            step_end_timestamp : step_end_timestamp,
            player : player
        }
    }
}

/// ATTENTION résultat inversé !
impl PartialOrd<Self> for PlayerMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlayerMove {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.step_end_timestamp > other.step_end_timestamp {
            return Less
        }
        else if self.step_end_timestamp < other.step_end_timestamp {
            return Greater
        }
        return Equal
    }
}

pub async fn init_move_list() {
    MOVE_LIST.get_or_init(|| Arc::new(RwLock::new(BinaryHeap::new())));
    let collection: Collection<Player> = get_collection(PLAYER_COLLECTION).clone_with_type();;

    let filter = doc! { PLAYER_IS_IN_MOVE: true };
    let cursor = collection.find(filter, None).await.unwrap();
    let docs: Result<Vec<Player>, _> = cursor.try_collect().await;
    let docs = docs.unwrap();

    for doc in docs {
        //TODO ajout des moves dans la liste
    }
}

pub async fn set_move(ctx : &Context, guild_id : u64, user_id : u64, channel_id : u64, destination_id : u64){
    //Refaire la pipeline :
    //3 cas possible :
    //Joueur dans une ville
    //joueur sur une route
    //  A l'arret
    //  en déplacement

    //Les déplacements sont décomposés en plusieurs sous déplacements à vitesses variables
    let universe_id = get_universe_id(guild_id).await;
    let updated_player = update_modifiers(user_id, universe_id).await;

    let road_channel = channel_is_road(updated_player.current_position_id).await;
    

    let mut road;
    let mut start_place;
    let destination_server_id = get_server_id_from_place(destination_id).await;
    let previous_player_destination = updated_player.destination_id;
    match road_channel{
        None => {
            //joueur dans une ville
            start_place = get_parent_category(ctx, guild_id, channel_id).await.unwrap();
            road = get_road_from_places(start_place.0, destination_id).await.unwrap();
        }
        Some(player_road) => {
            start_place = ChannelId::from(if player_road.place_1 == destination_id { player_road.place_2 } else { player_road.place_1 });
            road = player_road
            //joueur sur une route
        }
    }

    let mut player_stats = get_stat_hashmap(updated_player.stats);
    let mut speeds : Vec<f64> = vec!();
    speeds.push(player_stats.get(SPEED_STAT).unwrap().base_value);
    speeds.push(player_stats.get(MOUNT_SPEED_STAT).unwrap().base_value);

    //IDEE à chaque fois il faut chercher la plus grande valeur entre
    //  - la vitesse de base avec modifieurs
    //  - la vitesse de la monture avec modifieurs

    //en fonction de ca, il faut calculer la distance parcourue sur la route.
    //Si la distance parcourue est supérieure à la distance restante sur la route alors c'est ok pour atteindre la destination
    //Sinon on recommence avec la vitesse suivante.
    //Si il n'y a plus de vitesses disponible on continue d'utiliser la dernière vitesse.
    //Et on augmente la fatigue du joueur en fonction de la vitesse utilisée.
    //Plus le joueur est fatigué plus il est lent.
    //Si le joueur est trop fatigué il ne peut plus bouger et doit alors dormir pour faire remonter sa vitesse.

    //pseudo alogo
    //pour chaque modifieur sur la vitesse du personnage
    // calculer la vitesse en appliquant le modifieur et tous les suivants
    // si la vitesse est supérieure à celle de la monture,
    //  - utiliser la vitesse de la monture
    // sinon utiliser la vitesse calculée
    // calculer la distance parcourue avec la vitesse
    // si la distance parcourue est supérieure à la distance restante sur la route
    //  - calculer le temps restant sur la route
    // Sinon calculer la distance restante à parcourir et enregistrer le premier déplacement.
    // recommencer avec la distance restante.


    //serveur de destination
    //Univers

    // //Calcul durée
    //     //Distance
    // let distance = road.distance;
    //
    //     //speed
    // let stats = get_stat_hashmap(updated_player.stats);
    // let mut speed : Vec<f64> = vec!();
    // speed.push(stats.get(SPEED_STAT).unwrap().base_value);
    // speed.push(stats.get(MOUNT_SPEED_STAT).unwrap().base_value);
    //
    // if speed.is_empty(){
    //     //TODO erreur
    //     // ne devrais jamais arriver
    //     println!("erreur, vitesse non trouvée");
    //     return;
    // }
    // let speed = speed.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    //
    //     //multiplicateur
    // let universe = get_universe_doc(&universe_id).await.unwrap();
    // let multiplicateur = universe.time_modifier;
    //
    // //timestamps
    // let mut start_timestamp = get_timestamp();
    //
    // let mut end_timestamp = start_timestamp + ((distance * 3600) as f64 / (speed * multiplicateur as f64)).floor() as u64;
    // if updated_player.position_timestamp != 0{
    //     if destination_id != player_original_destination{
    //         //player à l'arret sur une route
    //         let temps_route_originale = updated_player.end_timestamp - updated_player.start_timestamp;
    //         let temps_passe = updated_player.position_timestamp - updated_player.start_timestamp;
    //         let temps_de_route_calcule = end_timestamp - start_timestamp;
    //         end_timestamp = ((temps_route_originale * temps_de_route_calcule) / temps_passe) + start_timestamp;
    //     }
    //     else {
    //         let temps_route_originale = updated_player.end_timestamp - updated_player.start_timestamp;
    //         let temps_passe = updated_player.position_timestamp - updated_player.start_timestamp;
    //         let temps_de_route_calcule = temps_route_originale - temps_passe;
    //         end_timestamp = ((temps_route_originale * temps_de_route_calcule) / temps_passe) + start_timestamp;
    //
    //         start_timestamp = start_timestamp - temps_passe;
    //     }
    // }
    //
    //
    // let destination_place = get_place(destination_id).await.unwrap();
    // let destination_role = destination_place.role_id;
    // let road_role_id = road.role_id;
    // let road_server_id = road.server_id;
    // let road_id = road.road_id;
    //
    // let player_move = PlayerMove
    // {
    //     player_id: user_id,
    //     universe_id: universe_id,
    //     is_end: false,
    //     step_end_timestamp: 0,
    //     player: Default::default(),
    // };
    //
    // set_player_in_move(player_move).await;
    // let member_road = ctx.http.get_member(road_server_id, user_id);
    // let member_current_place = ctx.http.get_member(guild_id, user_id);
    // let (member_road, member_current_place) = join!(member_road, member_current_place);
    //
    // let mut member_road = match member_road {
    //     Ok(member) => {
    //         //member.add_role(&ctx.http, RoleId(road_role_id)).await;
    //         member
    //     }
    //     Err(_) => {
    //         let invite = ctx
    //             .http
    //             .create_invite(road_id, &Default::default(), None)
    //             .await
    //             .expect("Failed to create invite");
    //
    //         let user = ctx.http.get_user(user_id).await.unwrap();
    //         user.direct_message(&ctx.http, |m| {
    //             m.content(format!(
    //                 "The road you want to take is on another server.\nPlease, join it to continue : {}",
    //                 invite.url()
    //             ))
    //         })
    //             .await
    //             .expect("Failed to send DM");
    //         return;
    //     }
    // };
    //
    // let mut member_place = match member_current_place {
    //     Ok(member) => {
    //         //let role_actual_place = get_place(actual_place.0).await.unwrap().get(ROLE_ID).unwrap().as_i64().unwrap() as u64;
    //         //member.remove_role(&ctx.http, RoleId(role_actual_place)).await;
    //         member
    //     }
    //     Err(_) => {println!("erreur, membre non trouvé sur le server de départ"); return}
    // };
    //
    // let role_actual_place = get_place(source_place.0).await.unwrap().role_id;
    // if member_place.guild_id == member_road.guild_id{
    //     let mut member_roles = member_place.roles.clone();
    //     member_roles.retain(|role|{role.0 != role_actual_place});
    //     member_roles.push(RoleId(road_role_id));
    //     member_place.edit(ctx, |member|{
    //         member.roles(member_roles)
    //     })
    //         .await
    //         .unwrap();
    // }
    // else {
    //     let member_palce_role = member_place.remove_role(&ctx.http, RoleId(role_actual_place));
    //     let member_road_role = member_road.add_role(&ctx.http, RoleId(road_role_id));
    //     join!(member_road_role, member_palce_role);
    // }
    //
    // //TODO message du channel départ/route
}

pub async fn add_player_move(player_move: PlayerMove) {
    let mut moves = MOVE_LIST.get().expect("MOVE_LIST not initialized").write().await;
    moves.push(player_move);
}

pub async fn remove_player_move(player_id: u64, universe_id: ObjectId){
    let mut moves = MOVE_LIST.get().expect("MOVE_LIST not initialized").write().await;
    moves.retain(|move_| !(move_.player_id == player_id && move_.universe_id == universe_id));
}

pub async fn _print_moves_list() {
    let moves = MOVE_LIST.get().expect("MOVE_LIST not initialized").read().await;
    println!("Moves {:?}", moves);
}

pub async fn handle_player_moves(ctx: &Context) {
    loop {
        time::sleep(Duration::from_secs(HANDLER_DELAY)).await;
        let moves_guard = MOVE_LIST.get().expect("MOVE_LIST not initialized").read().await;
        let current_timestamp = get_timestamp();

        println!("current_timestamp {:?}", current_timestamp);
        println!("moves_guard {:?}", moves_guard);

        if moves_guard.peek().is_none() || moves_guard.peek().unwrap().end_timestamp >= current_timestamp {
            drop(moves_guard);
            continue;
        };
        drop(moves_guard);
        let mut moves_guard = MOVE_LIST.get().expect("MOVE_LIST not initialized").write().await;

        loop{
            if moves_guard.peek().is_none() || moves_guard.peek().unwrap().end_timestamp >= current_timestamp {
                drop(moves_guard);
                break;
            }
            let player_move = moves_guard.pop().unwrap().clone();
            let cloned_ctx = Arc::new(ctx.clone());
            spawn(async move {
                let cloned_ctx = cloned_ctx.clone();
                handle_completed_move(player_move, &cloned_ctx).await;
            });
        }
    }
}

async fn handle_completed_move(player_move: PlayerMove, ctx: &Context) {
    let server_id = player_move.destination_server_id;
    let member_on_server = ctx.http.get_guild(server_id).await.unwrap().member(&ctx.http, player_move.player_id).await;

    let end_player_move = ending_player_move(player_move.clone());

    if member_on_server.is_err() {
        let user = ctx.http.get_user(player_move.player_id).await.unwrap();
        let collection = get_collection(PLACES_COLLECTION);
        let destination_place = collection.find_one(doc!{PLACE_ID : player_move.destination_id as i64}, None).await.unwrap().unwrap();
        let channel_destination_place_id = destination_place.get(DEFAULT_CHANNEL_PLACE).unwrap().as_i64().unwrap() as u64;

        let invite = ctx
            .http
            .create_invite(channel_destination_place_id, &Default::default(), None)
            .await
            .expect("Failed to create invite");

        user.direct_message(&ctx.http, |m| {
            m.content(format!(
                "You have reached your destination, but it's on another server !\nJoin it to continue : {}",
                invite.url()
            ))
        })
            .await
            .expect("Failed to send DM");

        match end_player_move.await {
            Ok(_) => {}
            Err(e) => {
                log(ReportType::ERROR, e.as_str());
                return;
            }
        };
    }
    else{
        println!("TODO mettre les bons roles");
        let road_role_id = player_move.road_role_id;
        let road_server_id = player_move.road_server_id;
        let destination_role_id = player_move.destination_role_id;
        let mut member_destination = member_on_server.unwrap();
        let member_road = ctx.http.get_member(road_server_id, player_move.player_id).await;
        match member_road {
            Ok(_) => {}
            Err(_) => {println!("erreur, membre non trouvé sur le server de la route"); return}
        };

        let mut member_road = member_road.unwrap();

        if player_move.road_server_id == player_move.destination_server_id{
            member_road.edit(ctx, |member_edit|{
                let mut roles = member_road.roles.clone();
                roles.retain(|role|{role.0 != road_role_id});
                roles.push(RoleId(destination_role_id));
                member_edit.roles(roles)
            }).await.unwrap();

            match ending_player_move(player_move.clone()).await {
                Ok(_) => {//TODO Log + message
                }
                Err(e) => {
                    //TODO message
                    log(ReportType::ERROR, e.as_str());
                    return;
                }
            };
        }
        else {
            let road_role_result = member_road.remove_role(&ctx.http, RoleId(road_role_id.clone())).await;
            let destination_role_result = member_destination.add_role(&ctx.http, RoleId(destination_role_id.clone())).await;
            if road_role_result.is_ok() && destination_role_result.is_ok() {
                match ending_player_move(player_move.clone()).await {
                    Ok(_) => {
                        //TODO Log + message
                    }
                    Err(e) => {
                        //TODO message
                        log(ReportType::ERROR, e.as_str());
                        return;
                    }
                };
            }
            else if road_role_result.is_err() || destination_role_result.is_err(){
                let road_name = ctx.http.get_channel(player_move.road_id).await.unwrap().guild().unwrap().name;
                let dest_name = ctx.http.get_channel(player_move.destination_id).await.unwrap().category().unwrap().name;
                member_destination.remove_role(&ctx.http, RoleId(destination_role_id)).await;
                member_road.add_role(&ctx.http, RoleId(road_role_id)).await;
                member_destination.user.dm(ctx, |dm|{
                    dm.content(format!("erreur ! Le deplacement de la route __**{}**__ à la ville __**{}**__ ne s'est pas terminé correctement. Le status précédent à été restauré.", road_name , dest_name))
                }).await.unwrap();
            }
        }
    }
        //TODO envoyer un message a l'arrivée et dans la route (avec la default locale de l'univers)
}


async fn get_destination_role_id(destination_id: u64) -> Option<serenity::model::id::RoleId> {
    let collection = get_collection(PLACES_COLLECTION);

    let filter = doc! { PLACE_ID: destination_id as i64 };

    if let Some(result) = collection.find_one(filter, None).await.unwrap() {
        let role_id = result.get_i64(ROLE_ID).unwrap();
        return Some(RoleId(role_id as u64));
    }
    None
}

async fn set_player_in_move(player_move : PlayerMove) -> mongodb::error::Result<UpdateResult> {
    let collection = get_collection(PLAYER_COLLECTION);
    let update_doc = doc! {
        "$set": { PLAYER_IS_IN_MOVE: true,
            PLAYER_START_TIMESTAMP: player_move.start_timestamp as i64,
            PLAYER_END_TIMESTAMP: player_move.end_timestamp as i64,
            PLAYER_DESTINATION_ID: player_move.destination_id as i64,
            PLAYER_DESTINATION_SERVER_ID: player_move.destination_server_id as i64,
            PLAYER_CURRENT_POSITION_ID: player_move.road_id as i64,
            PLAYER_CURRENT_POSITION_SERVER_ID: player_move.road_server_id as i64,
            PLAYER_DESTINATION_ROLE_ID: player_move.destination_role_id as i64,
            PLAYER_CURRENT_POSITION_ROLE_ID: player_move.road_role_id as i64
        }
    };

    add_player_move(player_move.clone()).await;
    collection.update_one(doc! { PLAYER_ID: player_move.player_id as i64 }, update_doc, None).await
}

pub async fn stop_player(player_id: u64, universe_id: ObjectId){
    let collection = get_collection(PLAYER_COLLECTION);
    let filter = doc! { PLAYER_ID: player_id as i64, UNIVERSE_ID: universe_id };
    let update_doc = doc! { "$set": { PLAYER_IS_IN_MOVE: false, PLAYER_POSITION_TIMESTAMP : get_timestamp() as i64} };
    collection.update_one(filter, update_doc, None).await;

    remove_player_move(player_id, universe_id).await;
}

async fn ending_player_move(player_move: PlayerMove) -> Result<String, String>{
    let collection = get_collection(PLAYER_COLLECTION);

    let filter = doc! { PLAYER_ID: player_move.player_id as i64, UNIVERSE_ID: player_move.universe_id };
    let update = doc! { "$set": {
        PLAYER_IS_IN_MOVE: false,
        PLAYER_POSITION_TIMESTAMP : 0,
        PLAYER_CURRENT_POSITION_ID : player_move.destination_id as i64,
        PLAYER_CURRENT_POSITION_ROLE_ID : player_move.destination_role_id as i64,
        PLAYER_CURRENT_POSITION_SERVER_ID : player_move.destination_server_id as i64
    }};

    match collection.update_one(filter.clone(), update, None).await {
        Ok(_) => { /*TODO log*/
            Ok("".to_string())
        }
        Err(_) => {
            Err(format!("Error while updating player document on ending move, player_id: {}", player_move.player_id))}
    }
}

pub fn start_verification_task(ctx: &Context) {
    let cloned_ctx = ctx.clone();
    spawn(async move {
        handle_player_moves(&cloned_ctx).await;
    });
}

pub async fn setup_player_on_join(ctx: &Context, member: &mut Member){
    let player_id = member.user.id.0;
    let universe_id = get_universe_id(member.guild_id.0).await;
    let doc = get_player_doc(player_id, &universe_id).await;
    match doc {
        Ok(player_doc) => {
            member.add_role(&ctx.http, RoleId(player_doc.current_position_role_id)).await;
        }
        Err(_) => {println!("Error while getting player document on join, player_id: {}", player_id)}
    };
}