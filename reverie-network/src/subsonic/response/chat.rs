//! Chat response DTOs

use reverie_core::ChatMessage;
use serde::Serialize;

/// Chat messages wrapper
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessagesData {
    #[serde(rename = "chatMessages")]
    pub chat_messages: ChatMessagesInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessagesInner {
    #[serde(rename = "chatMessage")]
    pub chat_message: Vec<ChatMessageItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageItem {
    pub username: String,
    pub message: String,
    pub time: i64,
}

impl From<&ChatMessage> for ChatMessageItem {
    fn from(m: &ChatMessage) -> Self {
        Self {
            username: m.username.clone(),
            message: m.message.clone(),
            time: m.time,
        }
    }
}

impl From<ChatMessagesData> for super::ResponseData {
    fn from(v: ChatMessagesData) -> Self {
        super::ResponseData::ChatMessages(v)
    }
}
