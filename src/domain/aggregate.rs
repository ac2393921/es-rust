use async_trait::async_trait;
use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::domain::commands::ChatCommand;
use crate::domain::events::{ChatError, ChatEvent};
use crate::services::ChatServices;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub id: Uuid,
    pub user_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRoom {
    pub room_id: Option<Uuid>,
    pub name: String,
    pub created_by: Option<String>,
    pub participants: HashSet<String>,
    pub usernames: HashMap<String, String>,
    pub messages: Vec<Message>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
impl Aggregate for ChatRoom {
    type Command = ChatCommand;
    type Event = ChatEvent;
    type Error = ChatError;
    type Services = ChatServices;

    fn aggregate_type() -> String {
        "ChatRoom".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            ChatCommand::CreateRoom { room_id, name, created_by } => {
                if self.room_id.is_some() {
                    return Err(ChatError::RoomAlreadyExists(format!("Room with ID {} already exists", room_id)));
                }

                Ok(vec![ChatEvent::RoomCreated {
                    room_id,
                    name,
                    created_by,
                    timestamp: chrono::Utc::now(),
                }])
            }

            ChatCommand::JoinRoom { user_id, username } => {
                if self.room_id.is_none() {
                    return Err(ChatError::RoomNotFound("Room does not exist".to_string()));
                }

                if self.participants.contains(&user_id) {
                    return Err(ChatError::UserAlreadyInRoom(format!("User {} is already in the room", user_id)));
                }

                Ok(vec![ChatEvent::UserJoined {
                    user_id,
                    username,
                    timestamp: chrono::Utc::now(),
                }])
            }

            ChatCommand::LeaveRoom { user_id } => {
                if self.room_id.is_none() {
                    return Err(ChatError::RoomNotFound("Room does not exist".to_string()));
                }

                if !self.participants.contains(&user_id) {
                    return Err(ChatError::UserNotInRoom(format!("User {} is not in the room", user_id)));
                }

                Ok(vec![ChatEvent::UserLeft {
                    user_id,
                    timestamp: chrono::Utc::now(),
                }])
            }

            ChatCommand::SendMessage { message_id, user_id, content, timestamp } => {
                if self.room_id.is_none() {
                    return Err(ChatError::RoomNotFound("Room does not exist".to_string()));
                }

                if !self.participants.contains(&user_id) {
                    return Err(ChatError::UserNotInRoom(format!("User {} is not in the room", user_id)));
                }

                Ok(vec![ChatEvent::MessageSent {
                    message_id,
                    user_id,
                    content,
                    timestamp,
                }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ChatEvent::RoomCreated { room_id, name, created_by, timestamp } => {
                self.room_id = Some(room_id);
                self.name = name;
                self.created_by = Some(created_by.clone());
                self.participants.insert(created_by);
                self.created_at = Some(timestamp);
            }

            ChatEvent::UserJoined { user_id, username, timestamp: _ } => {
                self.participants.insert(user_id.clone());
                self.usernames.insert(user_id, username);
            }

            ChatEvent::UserLeft { user_id, timestamp: _ } => {
                self.participants.remove(&user_id);
                self.usernames.remove(&user_id);
            }

            ChatEvent::MessageSent { message_id, user_id, content, timestamp } => {
                self.messages.push(Message {
                    id: message_id,
                    user_id,
                    content,
                    timestamp,
                });
            }
        }
    }
}

#[cfg(test)]
mod aggregate_tests {
    use std::vec;

    use super::*;
    use cqrs_es::test::TestFramework;

    type ChatRoomTestFramework = TestFramework<ChatRoom>;

    #[test]
    fn test_create_room() {
        let room_id = Uuid::new_v4();
        let command = ChatCommand::CreateRoom {
            room_id,
            name: "Test Room".to_string(),
            created_by: "user1".to_string(),
        };

        let expected = ChatEvent::RoomCreated {
            room_id,
            name: "Test Room".to_string(),
            created_by: "user1".to_string(),
            timestamp: chrono::Utc::now(),
        };

        ChatRoomTestFramework::with(ChatServices)
            .given_no_previous_events()
            .when(command)
            .then_expect_events_matching(|events| {
                assert_eq!(events.len(), 1);
                match &events[0] {
                    ChatEvent::RoomCreated { room_id: r, name, created_by, timestamp: _ } => {
                        assert_eq!(r, &room_id);
                        assert_eq!(name, "Test Room");
                        assert_eq!(created_by, "user1");
                        true
                    }
                    _ => false,
                }
            });
    }

    #[test]
    fn test_join_room() {
        let room_id = Uuid::new_v4();
        let previous = ChatEvent::RoomCreated {
            room_id,
            name: "Test Room".to_string(),
            created_by: "user1".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let command = ChatCommand::JoinRoom {
            user_id: "user2".to_string(),
            username: "User Two".to_string(),
        };

        ChatRoomTestFramework::with(ChatServices)
            .given(vec![previous])
            .when(command)
            .then_expect_events_matching(|events| {
                assert_eq!(events.len(), 1);
                match &events[0] {
                    ChatEvent::UserJoined { user_id, username, timestamp: _ } => {
                        assert_eq!(user_id, "user2");
                        assert_eq!(username, "User Two");
                        true
                    }
                    _ => false,
                }
            });
    }

    #[test]
    fn test_send_message() {
        let room_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();
        
        let previous_events = vec![
            ChatEvent::RoomCreated {
                room_id,
                name: "Test Room".to_string(),
                created_by: "user1".to_string(),
                timestamp: chrono::Utc::now(),
            },
            ChatEvent::UserJoined {
                user_id: "user2".to_string(),
                username: "User Two".to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];

        let command = ChatCommand::SendMessage {
            message_id,
            user_id: "user2".to_string(),
            content: "Hello, world!".to_string(),
            timestamp,
        };

        ChatRoomTestFramework::with(ChatServices)
            .given(previous_events)
            .when(command)
            .then_expect_events_matching(|events| {
                assert_eq!(events.len(), 1);
                match &events[0] {
                    ChatEvent::MessageSent { message_id: m, user_id, content, timestamp: t } => {
                        assert_eq!(m, &message_id);
                        assert_eq!(user_id, "user2");
                        assert_eq!(content, "Hello, world!");
                        assert_eq!(t, &timestamp);
                        true
                    }
                    _ => false,
                }
            });
    }

    #[test]
    fn test_leave_room() {
        let room_id = Uuid::new_v4();
        let previous_events = vec![
            ChatEvent::RoomCreated {
                room_id,
                name: "Test Room".to_string(),
                created_by: "user1".to_string(),
                timestamp: chrono::Utc::now(),
            },
            ChatEvent::UserJoined {
                user_id: "user2".to_string(),
                username: "User Two".to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];

        let command = ChatCommand::LeaveRoom {
            user_id: "user2".to_string(),
        };

        ChatRoomTestFramework::with(ChatServices)
            .given(previous_events)
            .when(command)
            .then_expect_events_matching(|events| {
                assert_eq!(events.len(), 1);
                match &events[0] {
                    ChatEvent::UserLeft { user_id, timestamp: _ } => {
                        assert_eq!(user_id, "user2");
                        true
                    }
                    _ => false,
                }
            });
    }
}
