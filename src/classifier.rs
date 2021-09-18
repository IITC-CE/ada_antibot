use teloxide::types::MessageEntityKind::TextLink;
use teloxide::types::{MessageEntity, User};

static BAD_WORDS: [&str; 3] = [" invest", "btc", "iit "];

fn check_is_spam(raw_text: String) -> bool {
    let text = format!(" {} ", raw_text.to_lowercase());

    let mut is_spam = false;
    for &word in BAD_WORDS.iter() {
        if text.contains(word) {
            is_spam = true;
            break;
        }
    }
    return is_spam;
}

pub(crate) fn is_spam_check_user(user: User) -> Option<&'static str> {
    let mut is_spam = false;

    if check_is_spam(user.first_name) {
        is_spam = true;
    }

    if let Some(text) = user.last_name {
        if is_spam == false && check_is_spam(text) {
            is_spam = true;
        }
    }

    if let Some(text) = user.username {
        if is_spam == false && check_is_spam(text) {
            is_spam = true;
        }
    }

    if is_spam {
        Some("Username")
    } else {
        None
    }
}

pub(crate) fn is_spam_check_message(
    text: Option<String>,
    entities: Vec<MessageEntity>,
) -> Option<&'static str> {
    let mut is_spam = false;

    if let Some(text_string) = text {
        if check_is_spam(text_string) {
            is_spam = true;
        }
    }

    if is_spam == false && entities.len() > 0 {
        match entities[0].clone().kind {
            TextLink { url } => {
                if check_is_spam(url) {
                    is_spam = true;
                }
            }
            _ => {}
        }
    }

    if is_spam {
        Some("Message")
    } else {
        None
    }
}
