mod universe;
mod lang;
mod constants;
mod common_functions;
mod players;
mod items;
mod bdd;

extern crate serenity;
extern crate core;
use serenity::{
    async_trait,
    model::{gateway::Ready},
    prelude::*,
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::{env};
use json::JsonValue;
use serenity::framework::StandardFramework;
use mongodb::{Client as MongoClient};
use once_cell::sync::OnceCell;
use serenity::model::application::interaction::Interaction;
use serenity::model::guild::Member;
use crate::universe::create_universe::{create_universe, create_universe_reactor, get_universe_id_resolved};
use crate::lang::lang_loader::lang_loader;
use crate::constants::{MOVE_MODAL_CUSTOM_ID, CREATE_UNIVERSE_COMMAND_NAME, CREATE_NEW_PLAYER_COMMAND_NAME, CREATE_PLAYER_VALIDATE_BUTTON_CUSTOM_ID, CREATE_PLAYER_CANCEL_BUTTON_CUSTOM_ID, CREATE_PLAYER_MODIFY_BUTTON_CUSTOM_ID, CREATE_PLAYER_REJECT_BUTTON_CUSTOM_ID, CREATE_PLAYER_ACCEPT_BUTTON_CUSTOM_ID, CREATE_PLAYER_FROM_PLAYER_CUSTOM_ID, REJECT_REASON_INTERACTION_ID, FINAL_SETUP_PLAYER_INTERACTION_CUSTOM_ID, CREATE_PLACE_COMMAND_NAME, CREATE_ROAD_COMMAND_NAME, MOVE_COMMAND_LOCALE_NAME, MOVE_VALUE_CUSTOM_ID, PLAYER_COLLECTION, PLAYER_DESTINATION_ROLE_ID, PLAYER_CURRENT_POSITION_ROLE_ID, STOP_MOVE};
use crate::players::new_player::{create_new_player, create_new_player_reactor, create_player_accept_button_trigger, create_player_cancel_button_trigger, create_player_modal_from_player, create_player_modify_button_trigger, create_player_reject_button_trigger, create_player_validate_button_trigger, finalise_player_creation, modify_after_reject};
use crate::players::player_moove::{init_move_list, set_move, setup_player_on_join, start_verification_task, stop_player};
use crate::players::move_function::{move_command_reactor, setup_move_command};
use crate::universe::create_place::{create_place, create_place_reactor};
use crate::universe::create_road::{create_road, create_road_reactor};
static MONGOCLIENT: OnceCell<MongoClient> = OnceCell::new();
static LANGS: OnceCell<HashMap<&str, JsonValue>> = OnceCell::new();


//handler discord, bot core
struct HandlerDiscord;

#[async_trait]
impl EventHandler for HandlerDiscord {
    async fn guild_member_addition(&self, _ctx: Context, mut _new_member: Member) {
        setup_player_on_join(&_ctx, &mut _new_member).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        //initialisation of move list for players moves
        init_move_list().await;
        start_verification_task(&ctx.clone());

        //setup langs
        let mut lang: HashMap<&str, JsonValue> = HashMap::new();
        lang_loader(&mut lang);
        LANGS.set(lang).unwrap();

        let _commands = ctx.http.get_global_application_commands().await.unwrap();
        // for command in &_commands{
        //     ctx.http.delete_global_application_command(command.id.0).await.unwrap();
        // }
        let _create_universe_command = create_universe(ctx.borrow());
        let _create_new_player = create_new_player(ctx.borrow());
        let _create_place = create_place(ctx.borrow());
        let _create_road = create_road(ctx.borrow());
        let _setup_move = setup_move_command(ctx.borrow());
        //join!(_create_universe_command, _create_new_player, _create_place, _create_road, _setup_move);
        let commands = ctx.http.get_global_application_commands().await.unwrap();
        for command in &commands{
            println!("{:?}", command.name);
        }
        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {
        match _interaction {
            Interaction::ApplicationCommand(_aci) => {
                let id = _aci.data.name.as_str();
                match id {
                    CREATE_UNIVERSE_COMMAND_NAME => create_universe_reactor(&_ctx, &_aci).await,
                    CREATE_NEW_PLAYER_COMMAND_NAME => create_new_player_reactor(&_ctx, &_aci).await,
                    CREATE_PLACE_COMMAND_NAME => create_place_reactor(&_ctx, &_aci).await,
                    CREATE_ROAD_COMMAND_NAME => create_road_reactor(&_ctx, &_aci).await,
                    MOVE_COMMAND_LOCALE_NAME => move_command_reactor(&_ctx, &_aci).await,
                    _ => {println!("{}", id)}
                }
            }
            Interaction::MessageComponent(_mci) => {
                let id = _mci.data.custom_id.as_str();
                match id {
                    CREATE_PLAYER_VALIDATE_BUTTON_CUSTOM_ID => {create_player_validate_button_trigger(&_ctx, &_mci).await},
                    CREATE_PLAYER_CANCEL_BUTTON_CUSTOM_ID => {create_player_cancel_button_trigger(&_ctx, &_mci).await},
                    CREATE_PLAYER_MODIFY_BUTTON_CUSTOM_ID => {create_player_modify_button_trigger(&_ctx, &_mci).await},
                    CREATE_PLAYER_REJECT_BUTTON_CUSTOM_ID => {create_player_reject_button_trigger(&_ctx, &_mci).await},
                    CREATE_PLAYER_ACCEPT_BUTTON_CUSTOM_ID => {create_player_accept_button_trigger(&_ctx, &_mci).await},
                    MOVE_MODAL_CUSTOM_ID => {
                        if _mci.data.values.len() == 1{
                            if _mci.data.values.get(0).unwrap().as_str() == STOP_MOVE{
                                let universe_id = get_universe_id_resolved(&_ctx, _mci.clone(), _mci.guild_id.unwrap().0).await;
                                stop_player(_mci.user.id.0, universe_id).await;
                            }
                            else{
                                set_move(&_ctx, _mci.guild_id.unwrap().0, _mci.user.id.0, _mci.channel_id.0, _mci.data.values.get(0).unwrap().as_str().parse().unwrap()).await;
                            }
                        }
                    },
                    _ => {println!("{}", id)}
                }
            }
            Interaction::ModalSubmit(_msi) => {
                let id = _msi.data.custom_id.as_str();
                match id {
                    CREATE_PLAYER_FROM_PLAYER_CUSTOM_ID => {create_player_modal_from_player(&_ctx, &_msi).await},
                    REJECT_REASON_INTERACTION_ID => {modify_after_reject(&_ctx, &_msi).await},
                    FINAL_SETUP_PLAYER_INTERACTION_CUSTOM_ID => {finalise_player_creation(&_ctx, &_msi).await},
                    _ => {println!("{}", id)}
                }
            }
            _ => {}
        }
    }
}

//main function, setup for clients
#[tokio::main]
async fn main() {
    //setup mongo client
    let uri = env::var("MONGODB_LOGIN").unwrap();
    let client = MongoClient::with_uri_str(&uri).await.unwrap();
    MONGOCLIENT.set(client).unwrap();

    //setup discord client
    let token = env::var("DISCORD_TOKEN").expect("token");

    let mut client = Client::builder(&token, Default::default())
        .event_handler(HandlerDiscord)
        .intents(GatewayIntents::all())
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}