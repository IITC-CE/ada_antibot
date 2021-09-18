use teloxide::prelude::*;
use teloxide::requests::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind};
use teloxide::types::{InlineKeyboardMarkup, ParseMode};
use teloxide::Bot;

use crate::{database, TELEGRAM_ADMINS_GROUP_ID, TELEGRAM_TARGET_GROUP_ID};
use std::error::Error;

pub(crate) fn unblock_markup(user_id: i64) -> InlineKeyboardMarkup {
    let button_text = "Unblock";
    let callback_text = format!("unblock_{}", user_id);

    InlineKeyboardMarkup::default().append_row(vec![InlineKeyboardButton::new(
        button_text,
        InlineKeyboardButtonKind::CallbackData(callback_text),
    )])
}

pub(crate) async fn ban_member(
    req: &AutoSend<Bot>,
    user_id: i64,
    user_first_name: String,
    reason: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mention = format!("[{}](tg://user?id={})", user_first_name, user_id);
    let message = format!("User {} is blocked due to reason: {}", mention, reason);

    // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.BanChatMember.html
    req.ban_chat_member(*TELEGRAM_TARGET_GROUP_ID, user_id)
        .revoke_messages(true)
        .await?;

    // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.SendMessage.html
    req.send_message(*TELEGRAM_ADMINS_GROUP_ID, message)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(unblock_markup(user_id))
        .await?;

    Ok(())
}

pub(crate) async fn unban_member(
    req: &AutoSend<Bot>,
    user_id: i64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.UnbanChatMember.html
    req.unban_chat_member(*TELEGRAM_TARGET_GROUP_ID, user_id)
        .send()
        .await?;

    match database::set_member_verified(user_id) {
        Ok(()) => {}
        _ => {
            log::error!("Error of assigning verified status to member");
        }
    }

    if let Ok(messages) = database::get_member_messages(user_id) {
        for mess in messages {
            // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.ForwardMessage.html
            req.forward_message(
                *TELEGRAM_TARGET_GROUP_ID,
                *TELEGRAM_ADMINS_GROUP_ID,
                mess.forward_mess_id,
            )
            .send()
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn hide_inline_keyboard(
    req: &AutoSend<Bot>,
    message_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.EditMessageReplyMarkupInline.html
    req.edit_message_reply_markup(*TELEGRAM_ADMINS_GROUP_ID, message_id)
        .reply_markup(InlineKeyboardMarkup::default())
        .send()
        .await?;

    Ok(())
}

pub(crate) async fn forward_and_ban(
    req: &AutoSend<Bot>,
    user_id: i64,
    user_first_name: String,
    reason: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Ok(messages) = database::get_member_messages(user_id) {
        for mess in messages {
            // https://docs.rs/teloxide/0.5.2/teloxide/payloads/struct.ForwardMessage.html
            let forward = req
                .forward_message(
                    *TELEGRAM_ADMINS_GROUP_ID,
                    *TELEGRAM_TARGET_GROUP_ID,
                    mess.mess_id,
                )
                .send()
                .await?;

            match database::set_message_forward(mess.mess_id, forward.id) {
                Ok(()) => {}
                _ => {
                    log::error!("Error in assigning ID of forwarded message");
                }
            }

            // For some unknown reason, messages are not automatically deleted via member ban method
            req.delete_message(*TELEGRAM_TARGET_GROUP_ID, mess.mess_id)
                .send()
                .await?;
        }

        match ban_member(req, user_id, user_first_name, reason).await {
            Ok(()) => {}
            Err(..) => {
                log::error!("Error while calling ban_member() function");
            }
        }
    }
    Ok(())
}
