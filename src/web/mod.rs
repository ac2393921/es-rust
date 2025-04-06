use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::commands::ChatCommand;
use crate::services::ChatRoomView;
use crate::ChatRoomFramework;

pub struct WebApi {
    framework: ChatRoomFramework,
    view_repository: Arc<crate::services::ChatRoomViewRepository>,
}

impl WebApi {
    pub fn new(
        framework: ChatRoomFramework,
        view_repository: Arc<crate::services::ChatRoomViewRepository>,
    ) -> Self {
        Self {
            framework,
            view_repository,
        }
    }

    pub async fn run(self, host: &str, port: u16) -> std::io::Result<()> {
        let framework = self.framework;
        let view_repository = self.view_repository.clone();

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(framework.clone()))
                .app_data(web::Data::new(view_repository.clone()))
                .service(
                    web::scope("/api")
                        .route("/rooms", web::get().to(get_rooms))
                        .route("/rooms", web::post().to(create_room))
                        .route("/rooms/{room_id}", web::get().to(get_room))
                        .route("/rooms/{room_id}/join", web::post().to(join_room))
                        .route("/rooms/{room_id}/leave", web::post().to(leave_room))
                        .route("/rooms/{room_id}/messages", web::post().to(send_message))
                )
        })
        .bind((host, port))?
        .run()
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRoomRequest {
    name: String,
    created_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinRoomRequest {
    user_id: String,
    username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LeaveRoomRequest {
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendMessageRequest {
    user_id: String,
    content: String,
}

async fn get_rooms(
    view_repository: web::Data<Arc<crate::services::ChatRoomViewRepository>>,
) -> impl Responder {
    let rooms = view_repository.get_all_rooms().await;
    HttpResponse::Ok().json(rooms)
}

async fn get_room(
    room_id: web::Path<Uuid>,
    view_repository: web::Data<Arc<crate::services::ChatRoomViewRepository>>,
) -> impl Responder {
    let room_id = room_id.into_inner();
    
    match view_repository.get_room(&room_id).await {
        Some(room) => HttpResponse::Ok().json(room),
        None => HttpResponse::NotFound().body(format!("Room with ID {} not found", room_id)),
    }
}

async fn create_room(
    req: web::Json<CreateRoomRequest>,
    framework: web::Data<ChatRoomFramework>,
) -> impl Responder {
    let room_id = Uuid::new_v4();
    
    let command = ChatCommand::CreateRoom {
        room_id,
        name: req.name.clone(),
        created_by: req.created_by.clone(),
    };
    
    match framework.execute(&room_id.to_string(), command).await {
        Ok(_) => HttpResponse::Created().json(room_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to create room: {}", e)),
    }
}

async fn join_room(
    room_id: web::Path<Uuid>,
    req: web::Json<JoinRoomRequest>,
    framework: web::Data<ChatRoomFramework>,
) -> impl Responder {
    let room_id = room_id.into_inner();
    
    let command = ChatCommand::JoinRoom {
        user_id: req.user_id.clone(),
        username: req.username.clone(),
    };
    
    match framework.execute(&room_id.to_string(), command).await {
        Ok(_) => HttpResponse::Ok().body("Joined room successfully"),
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to join room: {}", e)),
    }
}

async fn leave_room(
    room_id: web::Path<Uuid>,
    req: web::Json<LeaveRoomRequest>,
    framework: web::Data<ChatRoomFramework>,
) -> impl Responder {
    let room_id = room_id.into_inner();
    
    let command = ChatCommand::LeaveRoom {
        user_id: req.user_id.clone(),
    };
    
    match framework.execute(&room_id.to_string(), command).await {
        Ok(_) => HttpResponse::Ok().body("Left room successfully"),
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to leave room: {}", e)),
    }
}

async fn send_message(
    room_id: web::Path<Uuid>,
    req: web::Json<SendMessageRequest>,
    framework: web::Data<ChatRoomFramework>,
) -> impl Responder {
    let room_id = room_id.into_inner();
    let message_id = Uuid::new_v4();
    
    let command = ChatCommand::SendMessage {
        message_id,
        user_id: req.user_id.clone(),
        content: req.content.clone(),
        timestamp: chrono::Utc::now(),
    };
    
    match framework.execute(&room_id.to_string(), command).await {
        Ok(_) => HttpResponse::Created().json(message_id),
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to send message: {}", e)),
    }
}
