use std::sync::Arc;
use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, ListView, Panel, ScrollView, TextView};
use cursive::Cursive;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::domain::commands::ChatCommand;
use crate::services::ChatRoomView;
use crate::ChatRoomFramework;

pub struct TuiApp {
    framework: Arc<ChatRoomFramework>,
    view_repository: Arc<crate::services::ChatRoomViewRepository>,
    runtime: Runtime,
    current_room: Option<Uuid>,
    user_id: String,
    username: String,
}

impl TuiApp {
    pub fn new(
        framework: Arc<ChatRoomFramework>,
        view_repository: Arc<crate::services::ChatRoomViewRepository>,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        Self {
            framework,
            view_repository,
            runtime,
            current_room: None,
            user_id: String::new(),
            username: String::new(),
        }
    }

    pub fn run(&mut self) {
        let mut siv = cursive::default();
        
        siv.set_theme(cursive::theme::Theme::default());
        
        self.show_login_screen(&mut siv);
        
        siv.run();
    }

    fn show_login_screen(&self, siv: &mut Cursive) {
        let framework = self.framework.clone();
        let view_repository = self.view_repository.clone();
        let runtime = self.runtime.handle().clone();
        
        siv.add_layer(
            Dialog::new()
                .title("Chat App Login")
                .content(
                    LinearLayout::vertical()
                        .child(TextView::new("Enter your username:"))
                        .child(EditView::new().with_name("username"))
                        .child(TextView::new(""))
                )
                .button("Login", move |s| {
                    let username = s.call_on_name("username", |view: &mut EditView| {
                        view.get_content().to_string()
                    }).unwrap();
                    
                    if username.is_empty() {
                        s.add_layer(Dialog::info("Username cannot be empty"));
                        return;
                    }
                    
                    let user_id = Uuid::new_v4().to_string();
                    
                    let app = TuiApp {
                        framework: framework.clone(),
                        view_repository: view_repository.clone(),
                        runtime: runtime.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                        current_room: None,
                        user_id: user_id.clone(),
                        username: username.clone(),
                    };
                    
                    s.pop_layer();
                    app.show_room_list(s);
                })
        );
    }

    fn show_room_list(&self, siv: &mut Cursive) {
        let framework = self.framework.clone();
        let view_repository = self.view_repository.clone();
        let runtime = self.runtime.handle().clone();
        let user_id = self.user_id.clone();
        let username = self.username.clone();
        
        let rooms = runtime.block_on(async {
            view_repository.get_all_rooms().await
        });
        
        let mut room_list = ListView::new();
        
        for room in &rooms {
            let room_id = room.room_id;
            let room_name = room.name.clone();
            let participants_count = room.participants.len();
            
            let framework_for_room = framework.clone();
            let view_repository_for_room = view_repository.clone();
            let runtime_for_room = runtime.clone();
            let user_id_for_room = user_id.clone();
            let username_for_room = username.clone();
            
            let room_id_clone = room_id;
            let room_display = format!("{} ({} participants)", room_name, participants_count);
            
            let room_id_inner = room_id;
            let framework_inner = framework_for_room.clone();
            let view_repository_inner = view_repository_for_room.clone();
            let runtime_inner = runtime_for_room.clone();
            let user_id_inner = user_id_for_room.clone();
            let username_inner = username_for_room.clone();
            
            let button = cursive::views::Button::new("Join", {
                let runtime_inner = runtime_inner.clone();
                let framework_inner = framework_inner.clone();
                let view_repository_inner = view_repository_inner.clone();
                let user_id_inner = user_id_inner.clone();
                let username_inner = username_inner.clone();
                let room_id_inner = room_id_inner;
                
                move |s: &mut Cursive| {
                    runtime_inner.block_on(async {
                        let command = ChatCommand::JoinRoom {
                            user_id: user_id_inner.clone(),
                            username: username_inner.clone(),
                        };
                        
                        let _ = framework_inner.execute(&room_id_inner.to_string(), command).await;
                    });
                    
                    let app = TuiApp {
                        framework: framework_inner.clone(),
                        view_repository: view_repository_inner.clone(),
                        runtime: runtime_inner.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                        current_room: Some(room_id_inner),
                        user_id: user_id_inner.clone(),
                        username: username_inner.clone(),
                    };
                    
                    s.pop_layer();
                    app.show_chat_room(s);
                }
            });
            
            room_list.add_child(&room_display, button);
        }
        
        let framework_for_create = framework.clone();
        let view_repository_for_create = view_repository.clone();
        let user_id_for_create = user_id.clone();
        let username_for_create = username.clone();
        let runtime_for_create = runtime.clone();
        
        siv.add_layer(
            Dialog::new()
                .title("Chat Rooms")
                .content(
                    LinearLayout::vertical()
                        .child(Panel::new(room_list).title("Available Rooms"))
                )
                .button("Create Room", move |s| {
                    let framework_inner = framework_for_create.clone();
                    let view_repository_inner = view_repository_for_create.clone();
                    let user_id_inner = user_id_for_create.clone();
                    let username_inner = username_for_create.clone();
                    let runtime_inner = runtime_for_create.clone();
                    
                    s.add_layer(
                        Dialog::new()
                            .title("Create New Room")
                            .content(
                                LinearLayout::vertical()
                                    .child(TextView::new("Room Name:"))
                                    .child(EditView::new().with_name("room_name"))
                            )
                            .button("Create", move |s2| {
                                let room_name = s2.call_on_name("room_name", |view: &mut EditView| {
                                    view.get_content().to_string()
                                }).unwrap();
                                
                                if room_name.is_empty() {
                                    s2.add_layer(Dialog::info("Room name cannot be empty"));
                                    return;
                                }
                                
                                let room_id = Uuid::new_v4();
                                
                                runtime_inner.block_on(async {
                                    let command = ChatCommand::CreateRoom {
                                        room_id,
                                        name: room_name.clone(),
                                        created_by: user_id_inner.clone(),
                                    };
                                    
                                    let _ = framework_inner.execute(&room_id.to_string(), command).await;
                                });
                                
                                let app = TuiApp {
                                    framework: framework_inner.clone(),
                                    view_repository: view_repository_inner.clone(),
                                    runtime: runtime_inner.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                    current_room: Some(room_id),
                                    user_id: user_id_inner.clone(),
                                    username: username_inner.clone(),
                                };
                                
                                s2.pop_layer();
                                s2.pop_layer();
                                app.show_chat_room(s2);
                            })
                            .button("Cancel", |s2| {
                                s2.pop_layer();
                            })
                    );
                })
                .button("Logout", |s| {
                    s.pop_layer();
                })
        );
    }

    fn show_chat_room(&self, siv: &mut Cursive) {
        if let Some(room_id) = self.current_room {
            let framework = self.framework.clone();
            let view_repository = self.view_repository.clone();
            let runtime = self.runtime.handle().clone();
            let user_id = self.user_id.clone();
            let username = self.username.clone();
            
            let room = runtime.block_on(async {
                view_repository.get_room(&room_id).await
            });
            
            if let Some(room) = room {
                let mut messages = LinearLayout::vertical();
                
                for message in &room.messages {
                    let sender = if message.user_id == user_id {
                        "You".to_string()
                    } else {
                        message.username.clone()
                    };
                    
                    let timestamp = message.timestamp.format("%H:%M:%S").to_string();
                    let content = message.content.clone();
                    
                    messages.add_child(TextView::new(format!("[{}] {}: {}", timestamp, sender, content)));
                }
                
                let mut participants = LinearLayout::vertical();
                
                for participant in &room.participants {
                    participants.add_child(TextView::new(format!("• {}", participant.username)));
                }
                
                let framework_for_input = framework.clone();
                let view_repository_for_input = view_repository.clone();
                let runtime_for_input = runtime.clone();
                let user_id_for_input = user_id.clone();
                let username_for_input = username.clone();
                let room_id_for_input = room_id;
                
                let input = EditView::new()
                    .on_submit(move |s, content| {
                        if !content.is_empty() {
                            let message_id = Uuid::new_v4();
                            let timestamp = chrono::Utc::now();
                            
                            runtime_for_input.block_on(async {
                                let command = ChatCommand::SendMessage {
                                    message_id,
                                    user_id: user_id_for_input.clone(),
                                    content: content.to_string(),
                                    timestamp,
                                };
                                
                                let _ = framework_for_input.execute(&room_id_for_input.to_string(), command).await;
                            });
                            
                            s.call_on_name("message_input", |view: &mut EditView| {
                                view.set_content("");
                            });
                            
                            let app = TuiApp {
                                framework: framework_for_input.clone(),
                                view_repository: view_repository_for_input.clone(),
                                runtime: runtime_for_input.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                current_room: Some(room_id_for_input),
                                user_id: user_id_for_input.clone(),
                                username: username_for_input.clone(),
                            };
                            
                            s.pop_layer();
                            app.show_chat_room(s);
                        }
                    })
                    .with_name("message_input");
                
                let framework_for_refresh = framework.clone();
                let view_repository_for_refresh = view_repository.clone();
                let runtime_for_refresh = runtime.clone();
                let user_id_for_refresh = user_id.clone();
                let username_for_refresh = username.clone();
                let room_id_for_refresh = room_id;
                
                let framework_for_leave = framework.clone();
                let view_repository_for_leave = view_repository.clone();
                let runtime_for_leave = runtime.clone();
                let user_id_for_leave = user_id.clone();
                let username_for_leave = username.clone();
                let room_id_for_leave = room_id;
                
                siv.add_layer(
                    Dialog::new()
                        .title(format!("Chat Room: {}", room.name))
                        .content(
                            LinearLayout::horizontal()
                                .child(
                                    LinearLayout::vertical()
                                        .child(
                                            Panel::new(
                                                ScrollView::new(messages)
                                                    .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom)
                                            )
                                            .title("Messages")
                                            .full_height()
                                        )
                                        .child(Panel::new(input).title("Type your message"))
                                )
                                .child(
                                    Panel::new(participants)
                                        .title("Participants")
                                        .fixed_width(30)
                                )
                        )
                        .button("Refresh", move |s| {
                            let app = TuiApp {
                                framework: framework_for_refresh.clone(),
                                view_repository: view_repository_for_refresh.clone(),
                                runtime: runtime_for_refresh.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                current_room: Some(room_id_for_refresh),
                                user_id: user_id_for_refresh.clone(),
                                username: username_for_refresh.clone(),
                            };
                            
                            s.pop_layer();
                            app.show_chat_room(s);
                        })
                        .button("Leave Room", move |s| {
                            runtime_for_leave.block_on(async {
                                let command = ChatCommand::LeaveRoom {
                                    user_id: user_id_for_leave.clone(),
                                };
                                
                                let _ = framework_for_leave.execute(&room_id_for_leave.to_string(), command).await;
                            });
                            
                            let app = TuiApp {
                                framework: framework_for_leave.clone(),
                                view_repository: view_repository_for_leave.clone(),
                                runtime: runtime_for_leave.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                current_room: None,
                                user_id: user_id_for_leave.clone(),
                                username: username_for_leave.clone(),
                            };
                            
                            s.pop_layer();
                            app.show_room_list(s);
                        })
                );
            }
        }
    }
}
