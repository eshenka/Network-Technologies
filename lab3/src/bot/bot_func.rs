use std::default;

use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateFilterExt, UpdateHandler,
    },
    prelude::*,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
enum Command {
    Help,
    Start,
    Cancel,
}

pub fn start_bot(token: &str) -> teloxide::Bot {
    Bot::new(token)
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
enum State {
    #[default]
    Start,
    ReceiveLocation,
}

pub async fn dispatch(bot: teloxide::Bot) {
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![State::Start].branch(case![Command::Start].endpoint(start)));

    let message_handler = Update::filter_message()
        .branch(case![State::ReceiveLocation].endpoint(receive_location))
        .branch(command_handler);

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn receive_location(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(location) => bot.send_message(msg.chat.id, "You sent me: {text}").await?,
        None => {
            bot.send_message(
                msg.chat.id,
                "Please, tell me what location you are going to look up for",
            )
            .await?
        }
    };

    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start!").await?;
    dialogue.update(State::ReceiveLocation).await?;
    Ok(())
}
