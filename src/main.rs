#[macro_use]
extern crate lazy_static;

use std::{env, process};

use teloxide::prelude::*;
use teloxide::types::MediaKind::{Photo, Text};
use teloxide::types::MessageKind::{Common, NewChatMembers};
use tokio_stream::wrappers::UnboundedReceiverStream;

mod admins_group;
mod classifier;
mod database;
mod moderator;
mod target_group;

lazy_static! {
    pub(crate) static ref TELEGRAM_ADMINS_GROUP_ID: i64 = env::var("TELEGRAM_ADMINS_GROUP_ID")
        .expect("TELEGRAM_ADMINS_GROUP_ID is not defined")
        .parse::<i64>()
        .expect("TELEGRAM_ADMINS_GROUP_ID must be a number");
    pub(crate) static ref TELEGRAM_TARGET_GROUP_ID: i64 = env::var("TELEGRAM_TARGET_GROUP_ID")
        .expect("TELEGRAM_TARGET_GROUP_ID is not defined")
        .parse::<i64>()
        .expect("TELEGRAM_TARGET_GROUP_ID must be a number");
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting ADA Antibot...");

    if *TELEGRAM_ADMINS_GROUP_ID >= 0 {
        log::error!("TELEGRAM_ADMINS_GROUP_ID is incorrect, must be a negative number");
        process::exit(1)
    }
    if *TELEGRAM_TARGET_GROUP_ID >= 0 {
        log::error!("TELEGRAM_TARGET_GROUP_ID is incorrect, must be a negative number");
        process::exit(1)
    }

    database::init().expect("Database initialization error");

    let bot = Bot::from_env().auto_send();
    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                match cx.update.kind.clone() {
                    Common(message) => {
                        if let Some(user) = message.from {
                            match message.media_kind {
                                Text(media_text) => {
                                    if cx.update.chat_id() == *TELEGRAM_TARGET_GROUP_ID {
                                        target_group::handle_message(
                                            &cx.requester,
                                            user,
                                            cx.update.id,
                                            Some(media_text.text),
                                            media_text.entities,
                                        )
                                        .await;
                                    }
                                }
                                Photo(media_text) => {
                                    if cx.update.chat_id() == *TELEGRAM_TARGET_GROUP_ID {
                                        target_group::handle_message(
                                            &cx.requester,
                                            user,
                                            cx.update.id,
                                            media_text.caption,
                                            media_text.caption_entities,
                                        )
                                        .await;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    NewChatMembers(members) => {
                        for user in members.new_chat_members {
                            target_group::handle_new_chat_member(&cx.requester, user).await;
                        }
                    }
                    _ => {}
                };
            })
        })
        .callback_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                admins_group::handle_callback(cx).await;
            })
        })
        .dispatch()
        .await;
}
