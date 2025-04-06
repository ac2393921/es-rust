use anyhow::Result;
use async_trait::async_trait;
use cqrs_es::{AggregateContext, AggregateError, EventEnvelope, EventStore, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::aggregate::ChatRoom;
use crate::domain::events::ChatEvent;

pub struct ChatServices;

impl ChatServices {
    pub async fn notify_user_joined(&self, room_id: &Uuid, user_id: &str, username: &str) -> Result<()> {
        log::info!("User {} ({}) joined room {}", username, user_id, room_id);
        Ok(())
    }

    pub async fn notify_user_left(&self, room_id: &Uuid, user_id: &str) -> Result<()> {
        log::info!("User {} left room {}", user_id, room_id);
        Ok(())
    }

    pub async fn notify_message_sent(&self, room_id: &Uuid, user_id: &str, content: &str) -> Result<()> {
        log::info!("User {} sent message in room {}: {}", user_id, room_id, content);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoomView {
    pub room_id: Uuid,
    pub name: String,
    pub participants: Vec<UserInfo>,
    pub messages: Vec<MessageView>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageView {
    pub id: Uuid,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct ChatRoomViewRepository {
    views: Arc<RwLock<Vec<ChatRoomView>>>,
}

impl ChatRoomViewRepository {
    pub fn new() -> Self {
        Self {
            views: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_room(&self, room_id: &Uuid) -> Option<ChatRoomView> {
        let views = self.views.read().await;
        views.iter().find(|view| &view.room_id == room_id).cloned()
    }

    pub async fn get_all_rooms(&self) -> Vec<ChatRoomView> {
        let views = self.views.read().await;
        views.clone()
    }
}

#[async_trait]
impl Query<ChatRoom> for ChatRoomViewRepository {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<ChatRoom>]) {
        let result = self.update_view(aggregate_id, events).await;
        if let Err(e) = result {
            log::error!("Error updating view: {}", e);
        }
    }
}

impl ChatRoomViewRepository {
    async fn update_view(&self, aggregate_id: &str, events: &[EventEnvelope<ChatRoom>]) -> Result<(), anyhow::Error> {
        let mut views = self.views.write().await;
        
        for event_envelope in events {
            let event = &event_envelope.payload;
            
            match event {
                ChatEvent::RoomCreated { room_id, name, created_by, timestamp } => {
                    let view = ChatRoomView {
                        room_id: *room_id,
                        name: name.clone(),
                        participants: vec![UserInfo {
                            user_id: created_by.clone(),
                            username: created_by.clone(), // Initially use user_id as username
                        }],
                        messages: Vec::new(),
                        created_at: *timestamp,
                    };
                    views.push(view);
                }
                
                ChatEvent::UserJoined { user_id, username, timestamp: _ } => {
                    if let Some(view) = views.iter_mut().find(|v| v.room_id.to_string() == aggregate_id) {
                        view.participants.push(UserInfo {
                            user_id: user_id.clone(),
                            username: username.clone(),
                        });
                    }
                }
                
                ChatEvent::UserLeft { user_id, timestamp: _ } => {
                    if let Some(view) = views.iter_mut().find(|v| v.room_id.to_string() == aggregate_id) {
                        view.participants.retain(|p| p.user_id != *user_id);
                    }
                }
                
                ChatEvent::MessageSent { message_id, user_id, content, timestamp } => {
                    if let Some(view) = views.iter_mut().find(|v| v.room_id.to_string() == aggregate_id) {
                        let username = view.participants
                            .iter()
                            .find(|p| p.user_id == *user_id)
                            .map(|p| p.username.clone())
                            .unwrap_or_else(|| user_id.clone());
                            
                        view.messages.push(MessageView {
                            id: *message_id,
                            user_id: user_id.clone(),
                            username,
                            content: content.clone(),
                            timestamp: *timestamp,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

pub struct PostgresEventStore {
}

impl PostgresEventStore {
    pub fn new() -> Self {
        Self {}
    }
}

use cqrs_es::{AggregateContext, AggregateError};
use std::collections::HashMap;

#[async_trait]
impl EventStore<ChatRoom> for PostgresEventStore {
    type AC = AggregateContext<ChatRoom>;

    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope<ChatRoom>>, AggregateError<crate::domain::events::ChatError>> {
        Ok(Vec::new())
    }

    async fn load_aggregate(&self, aggregate_id: &str) -> Result<Self::AC, AggregateError<crate::domain::events::ChatError>> {
        let events = self.load_events(aggregate_id).await?;
        let aggregate_context = AggregateContext::new(aggregate_id, events);
        Ok(aggregate_context)
    }

    async fn append_events(&self, aggregate_id: &str, events: &[EventEnvelope<ChatRoom>]) -> Result<(), AggregateError<crate::domain::events::ChatError>> {
        for event in events {
            log::info!("Appending event: {:?} for aggregate: {}", event.payload, aggregate_id);
        }
        Ok(())
    }

    async fn commit(
        &self,
        events: Vec<crate::domain::events::ChatEvent>,
        aggregate_context: Self::AC,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<ChatRoom>>, AggregateError<crate::domain::events::ChatError>> {
        let aggregate_id = aggregate_context.aggregate_id().to_string();
        let committed_events = EventEnvelope::from_events(
            &aggregate_id,
            aggregate_context.current_sequence() + 1,
            events,
            metadata,
        );
        
        self.append_events(&aggregate_id, &committed_events).await?;
        
        Ok(committed_events)
    }
}
