//////////////////////////////////////////
// Dreambot
//////////////////////////////////////////
use chrono::{Datelike, NaiveDate};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use teloxide::{
    dispatching::dialogue::{serializer::Json, ErasedStorage, SqliteStorage, Storage},
    prelude::*,
};

mod db;
mod tables;
mod tzolkin;

type DreamDialogue = Dialogue<State, ErasedStorage<State>>;
type DreamStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const DATE_FORMAT: &str = "%d.%m.%Y";
const SEALS: &str = "resources/seals.json";
const DB_LOCATION: &str = "/srv/dreambot/db/dreambase.sqlite";
const MAX_DB_CONNECTIONS: u32 = 42;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    Calc,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    pretty_env_logger::init();
    log::info!("Starting Dreambot...");

    let storage: DreamStorage = SqliteStorage::open(DB_LOCATION, Json)
        .await
        .unwrap()
        .erase();

    let db_pool = SqlitePoolOptions::new()
        .max_connections(MAX_DB_CONNECTIONS)
        .connect(DB_LOCATION)
        .await?;

    // let seals = {
    //     let seals = std::fs::read_to_string(SEALS).expect("Can't find seals file");
    //     serde_json::from_str::<tzolkin::Seals>(&seals).expect("Can't parse seals file")
    // };

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, ErasedStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::Calc].endpoint(calc)),
    )
    .dependencies(dptree::deps![storage, db_pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}

async fn start(bot: Bot, dialogue: DreamDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Привет! Когда твой день рождения (дд.мм.гггг)?",
    )
    .await?;
    dialogue.update(State::Calc).await?;
    Ok(())
}

async fn calc(
    bot: Bot,
    dialogue: DreamDialogue,
    msg: Message,
    db_pool: SqlitePool,
) -> HandlerResult {
    match msg
        .text()
        .map(|text| NaiveDate::parse_from_str(text, DATE_FORMAT))
    {
        Some(Ok(date)) => {
            let kin = tzolkin::kin(date.day(), date.month(), date.year());
            let archetype = tzolkin::archetype(kin);

            /////////////////////////////////////////////////////////////////
            let seals = sqlx::query_as::<_, db::Seal>("SELECT * FROM seals WHERE id = ? OR id = ?")
                .bind(archetype.0 - 1)
                .bind(archetype.1 - 1)
                .fetch_all(&db_pool)
                .await?;
            // .unwrap();
            // let main_seal = &seals.0.get((archetype.0 - 1) as usize);
            // let type_seal = &seals.0.get((archetype.1 - 1) as usize);

            let main_seal = &seals[0];
            let type_seal = &seals[1];

            let archetype_image = main_seal.image.to_owned();
            let archetype_description = main_seal.archetype_description.to_owned();
            let portrait_name = main_seal.archetype.to_owned();
            let portrait_image = main_seal.image.to_owned();
            let portrait_description = main_seal.portrait_description.to_owned();
            let type_name = type_seal.archetype.to_owned();
            let type_image = type_seal.image.to_owned();
            let type_description = type_seal.type_description.to_owned();

            let result = kin;
            bot.send_message(msg.chat.id, format!("{result}\n")).await?;

            db::save(msg.chat.id.0, kin);
            /////////////////////////////////////////////////////////////////

            dialogue.update(State::Start).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Извини, но нужно дату в формате дд.мм.гггг")
                .await?;
        }
    }

    Ok(())
}
