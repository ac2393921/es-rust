use std::sync::Arc;
use std::time::Duration;

use cursive::align::HAlign;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, ListView, Panel, ScrollView, TextView};
use cursive::Cursive;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::domain::commands::ChatCommand;
use crate::services::ChatRoomView;
use crate::ChatRoomFramework;

pub struct TuiApp {
    framework: ChatRoomFramework,
    view_repository: Arc<crate::services::ChatRoomViewRepository>,
    runtime: Runtime,
    current_room: Option<Uuid>,
    user_id: String,
    username: String,
}

impl TuiApp {
    pub fn new(
        framework: ChatRoomFramework,
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
                    
                    let mut app = TuiApp {
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
            
            room_list.add_child(
                &format!("{} ({} participants)", room_name, participants_count),
                move |s| {
                    let room_id_clone = room_id;
                    let framework_clone = framework.clone();
                    let view_repository_clone = view_repository.clone();
                    let user_id_clone = user_id.clone();
                    let username_clone = username.clone();
                    
                    runtime.block_on(async {
                        let command = ChatCommand::JoinRoom {
                            user_id: user_id_clone.clone(),
                            username: username_clone.clone(),
                        };
                        
                        let _ = framework_clone.execute(&room_id_clone.to_string(), command).await;
                    });
                    
                    let mut app = TuiApp {
                        framework: framework_clone,
                        view_repository: view_repository_clone,
                        runtime: runtime.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                        current_room: Some(room_id_clone),
                        user_id: user_id_clone,
                        username: username_clone,
                    };
                    
                    s.pop_layer();
                    app.show_chat_room(s);
                },
            );
        }
        
        let framework_clone = framework.clone();
        let view_repository_clone = view_repository.clone();
        let user_id_clone = user_id.clone();
        let username_clone = username.clone();
        
        siv.add_layer(
            Dialog::new()
                .title("Chat Rooms")
                .content(
                    LinearLayout::vertical()
                        .child(Panel::new(room_list).title("Available Rooms"))
                )
                .button("Create Room", move |s| {
                    let framework_inner = framework_clone.clone();
                    let view_repository_inner = view_repository_clone.clone();
                    let user_id_inner = user_id_clone.clone();
                    let username_inner = username_clone.clone();
                    
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
                                
                                runtime.block_on(async {
                                    let command = ChatCommand::CreateRoom {
                                        room_id,
                                        name: room_name.clone(),
                                        created_by: user_id_inner.clone(),
                                    };
                                    
                                    let _ = framework_inner.execute(&room_id.to_string(), command).await;
                                });
                                
                                let mut app = TuiApp {
                                    framework: framework_inner.clone(),
                                    view_repository: view_repository_inner.clone(),
                                    runtime: runtime.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
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
                
                let input = EditView::new()
                    .on_submit(move |s, content| {
                        if !content.is_empty() {
                            let message_id = Uuid::new_v4();
                            let timestamp = chrono::Utc::now();
                            
                            runtime.block_on(async {
                                let command = ChatCommand::SendMessage {
                                    message_id,
                                    user_id: user_id.clone(),
                                    content: content.to_string(),
                                    timestamp,
                                };
                                
                                let _ = framework.execute(&room_id.to_string(), command).await;
                            });
                            
                            s.call_on_name("message_input", |view: &mut EditView| {
                                view.set_content("");
                            });
                            
                            let mut app = TuiApp {
                                framework: framework.clone(),
                                view_repository: view_repository.clone(),
                                runtime: runtime.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                current_room: Some(room_id),
                                user_id: user_id.clone(),
                                username: username.clone(),
                            };
                            
                            s.pop_layer();
                            app.show_chat_room(s);
                        }
                    })
                    .with_name("message_input");
                
                let framework_clone = framework.clone();
                let view_repository_clone = view_repository.clone();
                let user_id_clone = user_id.clone();
                let username_clone = username.clone();
                
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
                            let mut app = TuiApp {
                                framework: framework_clone.clone(),
                                view_repository: view_repository_clone.clone(),
                                runtime: runtime.block_on(async { tokio::runtime::Runtime::new().unwrap() }),
                                current_room: Some(room_id),
                                user_id: user_id_clone.clone(),
                                username: username_clone.clone(),
                            };
                            
                            s.pop_layer();
                            app.show_chat_room(s);
                        })
                        .button("Leave Room", move |s| {
                            runtime.block_on(async {
                                let command = ChatCommand::LeaveRoom {
                                    user_id: user_id.clone(),
                                };
                                
                                let _ = framework.execute(&room_id.to_string(), command).await;
                            });
                            
                            let mut app = TuiApp {
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
        }
    }
}
