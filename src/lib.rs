#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub mod domain;
pub mod services;
pub mod tui;
pub mod web;

use cqrs_es::{CqrsFramework, Query};
use std::sync::Arc;

use domain::aggregate::ChatRoom;
use services::{ChatRoomViewRepository, ChatServices, PostgresEventStore};

pub type ChatRoomFramework = CqrsFramework<ChatRoom, PostgresEventStore>;

pub fn create_chat_framework() -> (ChatRoomFramework, Arc<ChatRoomViewRepository>) {
    let event_store = PostgresEventStore::new();
    let services = ChatServices;
    let view_repository = Arc::new(ChatRoomViewRepository::new());
    
    let query_repo = ChatRoomViewRepository::new();
    let queries: Vec<Box<dyn Query<ChatRoom>>> = vec![Box::new(query_repo)];
    let framework = CqrsFramework::new(event_store, queries, services);
    
    (framework, view_repository)
}
