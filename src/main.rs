// ---------------------------------------
// Dreambot
// ---------------------------------------
use chrono::{Datelike, NaiveDate};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

mod db;
mod tables;
mod tzolkin;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const DATE_FORMAT: &str = "%d.%m.%Y";

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Calc,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Dreambot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::Calc].endpoint(calc)),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Привет! Когда твой день рождения (дд.мм.гггг)?",
    )
    .await?;
    dialogue.update(State::Calc).await?;
    Ok(())
}

async fn calc(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
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
