# 使用ガイド

## 概要

このチャットアプリケーションは、ターミナルユーザーインターフェース（TUI）とWeb APIの両方を提供します。このガイドでは、両方のインターフェースの使用方法について説明します。

## ターミナルユーザーインターフェース（TUI）

TUIは、ターミナル内で対話型のチャット体験を提供します。

### ログイン

アプリケーションを起動すると、最初にログイン画面が表示されます：

1. ユーザー名を入力します
2. 「Login」ボタンをクリックするか、Enterキーを押します

### チャットルームの管理

ログイン後、利用可能なチャットルームのリストが表示されます：

#### 既存のルームに参加する

1. 参加したいルームを選択します
2. 「Join」ボタンをクリックします

#### 新しいルームを作成する

1. 「Create Room」ボタンをクリックします
2. ルーム名を入力します
3. 「Create」ボタンをクリックします

### チャットルーム内での操作

チャットルーム内では、以下の操作が可能です：

#### メッセージの送信

1. 画面下部の入力フィールドにメッセージを入力します
2. Enterキーを押して送信します

#### 参加者の確認

画面右側のパネルに、現在のルームの参加者リストが表示されます。

#### その他の操作

- 「Refresh」ボタン：チャットの内容を更新します
- 「Leave Room」ボタン：現在のルームを退出し、ルームリストに戻ります

### ログアウト

ルームリスト画面で「Logout」ボタンをクリックすると、ログアウトしてログイン画面に戻ります。

## Web API

Web APIは、プログラムによるアクセスのためのRESTfulインターフェースを提供します。

### エンドポイント

以下のエンドポイントが利用可能です：

#### チャットルーム

- `GET /api/rooms` - すべてのチャットルームをリスト表示
- `POST /api/rooms` - 新しいチャットルームを作成
- `GET /api/rooms/{room_id}` - 特定のルームの詳細を取得

#### ルーム参加/退出

- `POST /api/rooms/{room_id}/join` - チャットルームに参加
- `POST /api/rooms/{room_id}/leave` - チャットルームを退出

#### メッセージ

- `POST /api/rooms/{room_id}/messages` - チャットルームにメッセージを送信

### APIの使用例

#### ルームの作成

```bash
curl -X POST http://localhost:8080/api/rooms \
  -H "Content-Type: application/json" \
  -d '{"name":"テストルーム","created_by":"user1"}'
```

#### ルームへの参加

```bash
curl -X POST http://localhost:8080/api/rooms/00000000-0000-0000-0000-000000000000/join \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user2","username":"ユーザー2"}'
```

#### メッセージの送信

```bash
curl -X POST http://localhost:8080/api/rooms/00000000-0000-0000-0000-000000000000/messages \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user2","content":"こんにちは、世界！"}'
```

#### ルームの情報取得

```bash
curl -X GET http://localhost:8080/api/rooms/00000000-0000-0000-0000-000000000000
```

#### すべてのルームのリスト取得

```bash
curl -X GET http://localhost:8080/api/rooms
```

#### ルームからの退出

```bash
curl -X POST http://localhost:8080/api/rooms/00000000-0000-0000-0000-000000000000/leave \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user2"}'
```

## レスポンス形式

APIレスポンスはJSON形式で返されます。例えば、ルーム情報のレスポンスは以下のようになります：

```json
{
  "room_id": "00000000-0000-0000-0000-000000000000",
  "name": "テストルーム",
  "participants": [
    {
      "user_id": "user1",
      "username": "ユーザー1"
    },
    {
      "user_id": "user2",
      "username": "ユーザー2"
    }
  ],
  "messages": [
    {
      "id": "00000000-0000-0000-0000-000000000001",
      "user_id": "user2",
      "username": "ユーザー2",
      "content": "こんにちは、世界！",
      "timestamp": "2023-04-01T12:34:56Z"
    }
  ],
  "created_at": "2023-04-01T12:30:00Z"
}
```
