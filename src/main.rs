mod universe;
mod lang;
mod constants;
mod common_functions;
mod moderation;

extern crate core;
use serenity::{
    async_trait,
    model::{gateway::Ready},
    prelude::*,
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use json::JsonValue;
use serenity::framework::StandardFramework;
use mongodb::{Client as MongoClient};
use once_cell::sync::OnceCell;

use serenity::model::application::interaction::Interaction;
use tokio::join;

use crate::universe::create_universe::{create_universe, create_universe_reactor};
use crate::lang::lang_loader::lang_loader;
use crate::constants::CREATE_UNIVERSE_COMMAND_NAME;
static MONGOCLIENT: OnceCell<MongoClient> = OnceCell::new();
static LANGS: OnceCell<HashMap<&str, JsonValue>> = OnceCell::new();

//handler discord, bot core
struct HandlerDiscord;
#[async_trait]
impl EventHandler for HandlerDiscord {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut lang: HashMap<&str, JsonValue> = HashMap::new();
        lang_loader(&mut lang);
        LANGS.set(lang).unwrap();

        let commands = ctx.http.get_global_application_commands().await.unwrap();
        /*for command in &commands{
            ctx.http.delete_global_application_command(command.id.0).await.unwrap();
        }*/
        let _create_universe_command = create_universe(ctx.borrow());
        //join!(_create_universe_command);
        let commands = ctx.http.get_global_application_commands().await.unwrap();
        for command in &commands{
            println!("{:?}", command.name);
        }

        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {
        match _interaction {
            Interaction::ApplicationCommand(_aci) => {
                match _aci.data.name.as_str() {
                    CREATE_UNIVERSE_COMMAND_NAME => create_universe_reactor(&_ctx, &_aci).await,
                    _ => {println!("{}", _aci.data.name.as_str())}
                }
            }
            Interaction::MessageComponent(_) => {}
            Interaction::ModalSubmit(_) => {}
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
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}