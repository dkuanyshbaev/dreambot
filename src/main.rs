use chrono::{Datelike, NaiveDate};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const DATE_FORMAT: &str = "%d.%m.%Y";

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Tzolkin,
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
            .branch(dptree::case![State::Tzolkin].endpoint(tzolkin)),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Hi! What's your birth date?")
        .await?;
    dialogue.update(State::Tzolkin).await?;
    Ok(())
}

async fn tzolkin(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg
        .text()
        .map(|text| NaiveDate::parse_from_str(text, DATE_FORMAT))
    {
        Some(Ok(date)) => {
            let result = fn_tzolkin(msg.chat.id, date.day(), date.month(), date.year());
            bot.send_message(msg.chat.id, format!("{result}\n")).await?;
            dialogue.update(State::Start).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Date format is dd.mm.yyyy")
                .await?;
        }
    }

    Ok(())
}

fn fn_tzolkin(_user_id: ChatId, _day: u32, _month: u32, _year: i32) -> String {
    "???".to_string()
}
