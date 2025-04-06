# 開発ガイド

## 開発環境のセットアップ

### 前提条件

- Rust と Cargo（[rustup.rs](https://rustup.rs/)からインストール可能）
- PostgreSQL（[postgresql.org](https://www.postgresql.org/download/)からインストール可能）
- お好みのコードエディタ（VS Code, IntelliJ IDEA with Rust plugin, Vim など）

### リポジトリのクローン

```bash
git clone https://github.com/ac2393921/es-rust.git
cd es-rust
```

### 依存関係のインストール

```bash
cargo build
```

### テストの実行

```bash
cargo test
```

### コードフォーマットの適用

```bash
cargo fmt
```

### リンターの実行

```bash
cargo clippy
```

## プロジェクト構造

```
es-rust/
├── src/
│   ├── domain/           # ドメインモデル（集約、コマンド、イベント）
│   │   ├── aggregate.rs  # ChatRoom集約
│   │   ├── commands.rs   # コマンド定義
│   │   ├── events.rs     # イベント定義
│   │   └── mod.rs        # モジュール定義
│   ├── services.rs       # サービス層（イベントストア、ビューリポジトリ）
│   ├── tui/              # ターミナルユーザーインターフェース
│   │   └── mod.rs        # TUI実装
│   ├── web/              # Web API
│   │   └── mod.rs        # API実装
│   ├── lib.rs            # ライブラリエントリポイント
│   └── main.rs           # アプリケーションエントリポイント
├── Cargo.toml            # プロジェクト設定と依存関係
├── Cargo.lock            # 依存関係のバージョンロック
├── Dockerfile            # Dockerイメージ定義
├── docker-compose.yml    # Docker Compose設定
└── README.md             # プロジェクト概要
```

## アーキテクチャの概要

このアプリケーションは、イベントソーシングとCQRSのアーキテクチャパターンに基づいています：

### ドメイン層

- **集約（Aggregate）**: システムの状態を表現し、コマンドを処理します
- **コマンド（Commands）**: システムに対する操作を表現します
- **イベント（Events）**: システムの状態変更を表現します

### サービス層

- **イベントストア**: イベントの永続化を担当します
- **ビューリポジトリ**: 読み取り最適化されたビューを提供します

### UI層

- **TUI**: ターミナルベースのユーザーインターフェース
- **Web API**: RESTful APIインターフェース

## 新機能の追加方法

### 新しいコマンドの追加

1. `src/domain/commands.rs`に新しいコマンドを追加します：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatCommand {
    // 既存のコマンド...
    
    // 新しいコマンド
    PinMessage {
        message_id: Uuid,
        user_id: String,
    },
}
```

2. `src/domain/events.rs`に対応するイベントを追加します：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatEvent {
    // 既存のイベント...
    
    // 新しいイベント
    MessagePinned {
        message_id: Uuid,
        pinned_by: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}
```

3. `src/domain/aggregate.rs`の`ChatRoom`集約にコマンドハンドラーを実装します：

```rust
impl Aggregate for ChatRoom {
    // 既存の実装...
    
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            // 既存のコマンドハンドラー...
            
            // 新しいコマンドハンドラー
            ChatCommand::PinMessage { message_id, user_id } => {
                // コマンドの検証ロジック
                
                // イベントの生成
                Ok(vec![ChatEvent::MessagePinned {
                    message_id,
                    pinned_by: user_id,
                    timestamp: chrono::Utc::now(),
                }])
            }
        }
    }
    
    fn apply(&mut self, event: Self::Event) {
        match event {
            // 既存のイベントハンドラー...
            
            // 新しいイベントハンドラー
            ChatEvent::MessagePinned { message_id, pinned_by: _, timestamp: _ } => {
                // 集約の状態を更新
                if let Some(room) = &mut self.room {
                    room.pinned_messages.push(message_id);
                }
            }
        }
    }
}
```

4. `src/services.rs`のビューリポジトリを更新して、新しいイベントを処理します：

```rust
impl ChatRoomViewRepository {
    async fn update_view(&self, aggregate_id: &str, events: &[EventEnvelope<ChatRoom>]) -> Result<(), anyhow::Error> {
        // 既存の実装...
        
        match event {
            // 既存のイベントハンドラー...
            
            // 新しいイベントハンドラー
            ChatEvent::MessagePinned { message_id, pinned_by, timestamp: _ } => {
                if let Some(view) = views.iter_mut().find(|v| v.room_id.to_string() == aggregate_id) {
                    if let Some(message) = view.messages.iter_mut().find(|m| m.id == *message_id) {
                        message.is_pinned = true;
                        message.pinned_by = Some(pinned_by.clone());
                    }
                }
            }
        }
        
        // 残りの実装...
    }
}
```

5. TUIとWeb APIを更新して、新機能をサポートします。

### 新しいAPIエンドポイントの追加

`src/web/mod.rs`に新しいエンドポイントを追加します：

```rust
// リクエスト構造体の定義
#[derive(Debug, Serialize, Deserialize)]
struct PinMessageRequest {
    user_id: String,
}

// エンドポイントの実装
async fn pin_message(
    room_id: web::Path<Uuid>,
    message_id: web::Path<Uuid>,
    req: web::Json<PinMessageRequest>,
    framework: web::Data<Arc<ChatRoomFramework>>,
) -> impl Responder {
    let room_id = room_id.into_inner();
    let message_id = message_id.into_inner();
    
    let command = ChatCommand::PinMessage {
        message_id,
        user_id: req.user_id.clone(),
    };
    
    match framework.execute(&room_id.to_string(), command).await {
        Ok(_) => HttpResponse::Ok().body("Message pinned successfully"),
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to pin message: {}", e)),
    }
}

// ルーターの更新
App::new()
    .app_data(web::Data::new(framework.clone()))
    .app_data(web::Data::new(view_repository.clone()))
    .service(
        web::scope("/api")
            // 既存のルート...
            .route("/rooms/{room_id}/messages/{message_id}/pin", web::post().to(pin_message))
    )
```

## テスト戦略

### ユニットテスト

集約、コマンド、イベントのユニットテストを作成します：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pin_message_command() {
        // テストの準備
        let mut room = ChatRoom::default();
        let room_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let user_id = "user1".to_string();
        
        // ルーム作成
        let create_command = ChatCommand::CreateRoom {
            room_id,
            name: "Test Room".to_string(),
            created_by: user_id.clone(),
        };
        
        let events = room.handle(create_command).unwrap();
        for event in events {
            room.apply(event);
        }
        
        // メッセージ送信
        let send_command = ChatCommand::SendMessage {
            message_id,
            user_id: user_id.clone(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let events = room.handle(send_command).unwrap();
        for event in events {
            room.apply(event);
        }
        
        // メッセージをピン留め
        let pin_command = ChatCommand::PinMessage {
            message_id,
            user_id: user_id.clone(),
        };
        
        let events = room.handle(pin_command).unwrap();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            ChatEvent::MessagePinned { message_id: pinned_id, pinned_by, timestamp } => {
                assert_eq!(pinned_id, &message_id);
                assert_eq!(pinned_by, &user_id);
                assert!(timestamp <= &chrono::Utc::now());
            }
            _ => panic!("Expected MessagePinned event"),
        }
        
        // イベントを適用
        for event in events {
            room.apply(event);
        }
        
        // 状態の検証
        assert!(room.room.unwrap().pinned_messages.contains(&message_id));
    }
}
```

### 統合テスト

APIエンドポイントの統合テストを作成します：

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, App};
    
    #[actix_web::test]
    async fn test_pin_message_endpoint() {
        // テスト用のアプリケーションを設定
        let (framework, view_repository) = create_test_framework();
        let framework = Arc::new(framework);
        let view_repository = Arc::new(view_repository);
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(framework.clone()))
                .app_data(web::Data::new(view_repository.clone()))
                .service(
                    web::scope("/api")
                        .route("/rooms/{room_id}/messages/{message_id}/pin", web::post().to(pin_message))
                )
        ).await;
        
        // テストデータの準備
        let room_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let user_id = "test_user".to_string();
        
        // リクエストの作成と送信
        let req = test::TestRequest::post()
            .uri(&format!("/api/rooms/{}/messages/{}/pin", room_id, message_id))
            .set_json(&PinMessageRequest { user_id: user_id.clone() })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // レスポンスの検証
        assert_eq!(resp.status(), StatusCode::OK);
        
        // ビューモデルの検証
        let room = view_repository.get_room(&room_id).await.unwrap();
        let message = room.messages.iter().find(|m| m.id == message_id).unwrap();
        assert!(message.is_pinned);
        assert_eq!(message.pinned_by.as_ref().unwrap(), &user_id);
    }
}
```

## パフォーマンスの最適化

### イベントストアの最適化

- イベントのバッチ処理
- イベントのスナップショット作成
- インデックスの適切な設定

### ビューモデルの最適化

- キャッシュの導入
- 読み取り専用レプリカの使用
- 非正規化データの適切な設計

## デプロイメント

### 本番環境へのデプロイ

1. アプリケーションをビルドします：

```bash
cargo build --release
```

2. 設定ファイルを準備します：

```bash
cp .env.example .env
# .envファイルを編集して本番環境の設定を行います
```

3. データベースを準備します：

```bash
psql -U postgres -c "CREATE DATABASE chat_app_prod;"
```

4. アプリケーションを実行します：

```bash
RUST_LOG=info DATABASE_URL=postgres://username:password@localhost:5432/chat_app_prod ./target/release/chat-app
```

### Dockerを使用したデプロイ

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## 貢献ガイドライン

1. フォークしてクローンします
2. 新しいブランチを作成します：`git checkout -b feature/your-feature-name`
3. 変更を加えます
4. テストを実行します：`cargo test`
5. コードをフォーマットします：`cargo fmt`
6. リンターを実行します：`cargo clippy`
7. 変更をコミットします：`git commit -m "Add your feature"`
8. プッシュします：`git push origin feature/your-feature-name`
9. プルリクエストを作成します
