use crate::{classifier, database, moderator};
use teloxide::prelude::*;
use teloxide::types::{MessageEntity, User};

pub(crate) async fn handle_new_chat_member(req: &AutoSend<Bot>, user: User) {
    let user_id = user.id;
    let user_first_name = user.first_name.clone();

    match database::add_member(user_id) {
        Ok(()) => {
            if let Some(reason) = classifier::is_spam_check_user(user) {
                match moderator::ban_member(req, user_id, user_first_name, reason).await {
                    Ok(()) => {}
                    Err(..) => {
                        log::error!("Error while calling ban_member() function");
                    }
                }
            }
        }
        _ => {
            log::error!("Error creating a user in database");
        }
    }
}

pub(crate) async fn handle_message(
    req: &AutoSend<Bot>,
    user: User,
    mess_id: i32,
    text: Option<String>,
    entities: Vec<MessageEntity>,
) {
    if let Ok(db_response_info) = database::get_member_status_and_messages(user.id) {
        if let Some(info) = db_response_info {
            // Check only if the member has recently joined
            if info.is_not_verified == 1 {
                if info.messages > 2 {
                    match database::set_member_verified(user.id) {
                        Ok(()) => {}
                        _ => {
                            log::error!("Error of assigning verified status to member");
                        }
                    }
                } else {
                    match database::add_message(mess_id, user.id) {
                        Ok(()) => {
                            if let Some(reason) = classifier::is_spam_check_message(text, entities)
                            {
                                match moderator::forward_and_ban(
                                    req,
                                    user.id,
                                    user.first_name,
                                    reason,
                                )
                                .await
                                {
                                    Ok(()) => {}
                                    Err(..) => {
                                        log::error!(
                                            "Error while calling forward_and_ban() function"
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            log::error!("Error when saving the message to database");
                        }
                    }
                }
            }
        }
    }
}
