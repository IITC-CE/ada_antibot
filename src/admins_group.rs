use crate::moderator;
use teloxide::prelude::*;

static ERROR: &str = "An error occurred while performing the operation";

pub(crate) async fn handle_callback(cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>) {
    if let Some(callback_text) = cx.update.data {
        if let Some(message) = cx.update.message {
            let data: Vec<&str> = callback_text.split("_").collect();
            if data.len() == 2 {
                if let Some(answer) = moder_actions(&cx.requester, data[0], data[1]).await {
                    log::error!("Callback answer: {}", answer);
                    match cx
                        .requester
                        .answer_callback_query(cx.update.id)
                        .text(answer)
                        .send()
                        .await
                    {
                        Ok(..) => {
                            hide_inline_keyboard(&cx.requester, message.id).await;
                        }
                        Err(..) => {
                            log::error!("Error while calling answer_callback_query() function");
                        }
                    }
                }
            }
        }
    }
}

async fn moder_actions<'a>(
    req: &'a AutoSend<Bot>,
    action: &'a str,
    data: &'a str,
) -> Option<&'a str> {
    return match action {
        "unblock" => match data.parse::<i64>() {
            Ok(user_id) => match moderator::unban_member(req, user_id).await {
                Ok(()) => Some("Unblocked"),
                Err(..) => {
                    log::error!("Error while calling unban_member() function");
                    Some(ERROR)
                }
            },
            Err(..) => {
                log::debug!("callback_queries_handler, user_id parsing error: {}", data);
                None
            }
        },
        _ => {
            log::debug!("callback_queries_handler, unexpected action: {}", action);
            None
        }
    };
}

async fn hide_inline_keyboard(req: &AutoSend<Bot>, message_id: i32) {
    match moderator::hide_inline_keyboard(req, message_id).await {
        Ok(()) => {}
        Err(..) => {
            log::error!("Error while calling hide_inline_keyboard() function");
        }
    }
}
