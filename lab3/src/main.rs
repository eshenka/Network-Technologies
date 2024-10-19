use token::TOKEN;
mod token;

use lab3::*;

#[tokio::main]
async fn main() {
    let bot = bot::start_bot(TOKEN);

    bot::dispatch(bot).await;
}
