//////////////////////////////////////////
// Dreambot
//////////////////////////////////////////
use chrono::{Datelike, NaiveDate};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use teloxide::{
    dispatching::dialogue::{serializer::Json, ErasedStorage, SqliteStorage, Storage},
    prelude::*,
    types::InputFile,
};

mod db;
mod tables;
mod tzolkin;

type DreamDialogue = Dialogue<State, ErasedStorage<State>>;
type DreamStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const DATE_FORMAT: &str = "%d.%m.%Y";
const DB_LOCATION: &str = "/srv/dreambot/db/dreambase.sqlite";
const SEALS_LOCATION: &str = "/srv/seals";
const MAX_DB_CONNECTIONS: u32 = 10;

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
            let main_seal = db::get_seal(&db_pool, archetype.0).await?;
            let type_seal = db::get_seal(&db_pool, archetype.1).await?;

            let archetype_image = main_seal.image;
            let archetype_description = main_seal.archetype_description;
            let portrait_name = main_seal.archetype;
            let portrait_image = archetype_image.clone();
            let portrait_description = main_seal.portrait_description;
            let type_name = type_seal.archetype;
            let type_image = type_seal.image;
            let type_description = type_seal.type_description;

            bot.send_photo(
                msg.chat.id,
                InputFile::file(format!("{SEALS_LOCATION}/{archetype_image}")),
            )
            .await?;
            bot.send_message(msg.chat.id, format!("{archetype_description}\n"))
                .await?;
            bot.send_message(msg.chat.id, format!("{portrait_name}\n"))
                .await?;
            bot.send_photo(
                msg.chat.id,
                InputFile::file(format!("{SEALS_LOCATION}/{portrait_image}")),
            )
            .await?;
            bot.send_message(msg.chat.id, format!("{portrait_description}\n"))
                .await?;
            bot.send_message(msg.chat.id, format!("{type_name}\n"))
                .await?;
            bot.send_photo(
                msg.chat.id,
                InputFile::file(format!("{SEALS_LOCATION}/{type_image}")),
            )
            .await?;
            bot.send_message(msg.chat.id, format!("{type_description}\n"))
                .await?;

            db::save_birthday(&db_pool, msg.chat.id.0, date.to_string()).await;
            dialogue.update(State::Start).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Извини, но нужно дату в формате дд.мм.гггг")
                .await?;
        }
    }

    Ok(())
}
