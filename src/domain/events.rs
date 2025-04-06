use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatEvent {
    RoomCreated {
        room_id: Uuid,
        name: String,
        created_by: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    UserJoined {
        user_id: String,
        username: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    UserLeft {
        user_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    MessageSent {
        message_id: Uuid,
        user_id: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl DomainEvent for ChatEvent {
    fn event_type(&self) -> String {
        match self {
            ChatEvent::RoomCreated { .. } => "RoomCreated".to_string(),
            ChatEvent::UserJoined { .. } => "UserJoined".to_string(),
            ChatEvent::UserLeft { .. } => "UserLeft".to_string(),
            ChatEvent::MessageSent { .. } => "MessageSent".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Room already exists: {0}")]
    RoomAlreadyExists(String),
    
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    
    #[error("User already in room: {0}")]
    UserAlreadyInRoom(String),
    
    #[error("User not in room: {0}")]
    UserNotInRoom(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<&str> for ChatError {
    fn from(msg: &str) -> Self {
        ChatError::Unknown(msg.to_string())
    }
}
