extern crate core;
use serenity::{
    async_trait,
    model::{gateway::Ready},
    prelude::*,
};
use std::env;
use serenity::framework::StandardFramework;
use serenity::model::application::interaction::{Interaction};
use mongodb::{Client as MongoClient};
use once_cell::sync::OnceCell;


//global variable for mongodb client
static MONGOCLIENT: OnceCell<MongoClient> = OnceCell::new();

//handler discord, bot core
struct HandlerDiscord;
#[async_trait]
impl EventHandler for HandlerDiscord {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let commands = ctx.http.get_global_application_commands().await.unwrap();
        /*for command in commands{
            ctx.http.delete_global_application_command(command.id.0).await.unwrap();
        }*/
        for command in commands{
            println!("{:?}", command.name);
        }

        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction{
            Interaction::ApplicationCommand(command) => {
                match command.data.name.as_str() {
                    _ => ()
                }},

            Interaction::ModalSubmit(mci) => {
                match mci.data.custom_id.as_str() {
                    _ => ()
                }},

            Interaction::MessageComponent(mci) => {
                match mci.data.custom_id.as_str() {
                    _ => { }
                }},

            _ => (),
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