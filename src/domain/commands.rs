use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatCommand {
    CreateRoom {
        room_id: Uuid,
        name: String,
        created_by: String,
    },
    JoinRoom {
        user_id: String,
        username: String,
    },
    LeaveRoom {
        user_id: String,
    },
    SendMessage {
        message_id: Uuid,
        user_id: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}
