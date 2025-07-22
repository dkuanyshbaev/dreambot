// ---------------------------------------
// Dreambot
// ---------------------------------------
use chrono::{Datelike, NaiveDate};
use sqlx::sqlite::SqlitePoolOptions;
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
const DB_LOCATION: &str = "db/dreambase.sqlite";
const MAX_DB_CONNECTIONS: u32 = 5;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    Calc,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Dreambot...");

    let storage: DreamStorage = SqliteStorage::open(DB_LOCATION, Json)
        .await
        .unwrap()
        .erase();

    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_DB_CONNECTIONS)
        .connect(DB_LOCATION)
        .await?;

    let seals = {
        let seals = std::fs::read_to_string(SEALS).expect("Can't find seals file");
        serde_json::from_str::<tzolkin::Seals>(&seals).expect("Can't parse seals file")
    };

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, ErasedStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::Calc].endpoint(calc)),
    )
    .dependencies(dptree::deps![storage])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
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

async fn calc(bot: Bot, dialogue: DreamDialogue, msg: Message) -> HandlerResult {
    match msg
        .text()
        .map(|text| NaiveDate::parse_from_str(text, DATE_FORMAT))
    {
        Some(Ok(date)) => {
            let kin = tzolkin::calc(date.day(), date.month(), date.year());
            let result = kin;
            db::save(msg.chat.id.0, kin);
            bot.send_message(msg.chat.id, format!("{result}\n")).await?;
            dialogue.update(State::Start).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Извини, но нужно дату в формате дд.мм.гггг")
                .await?;
        }
    }

    Ok(())
}
