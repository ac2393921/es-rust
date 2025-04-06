# インストールガイド

## 前提条件

このアプリケーションを実行するには、以下のソフトウェアが必要です：

- Docker と Docker Compose
- または、Rust と PostgreSQL（手動セットアップの場合）

## Dockerを使用したセットアップ（推奨）

Dockerを使用すると、アプリケーションとその依存関係を簡単にセットアップできます。

### 手順

1. リポジトリをクローンします：

```bash
git clone https://github.com/ac2393921/es-rust.git
cd es-rust
```

2. Docker Composeを使用してアプリケーションを起動します：

```bash
docker-compose up
```

これにより、以下のコンポーネントが起動します：
- PostgreSQLデータベース（イベントストレージ用）
- チャットアプリケーション（TUIとWeb API）

アプリケーションは以下のエンドポイントで利用可能になります：
- Web API: http://localhost:8080/api
- TUI: コンテナ内で実行（ログで確認可能）

## 手動セットアップ

Docker以外の方法でアプリケーションを実行する場合は、以下の手順に従ってください。

### 前提条件

- Rust と Cargo（[rustup.rs](https://rustup.rs/)からインストール可能）
- PostgreSQL（[postgresql.org](https://www.postgresql.org/download/)からインストール可能）

### 手順

1. リポジトリをクローンします：

```bash
git clone https://github.com/ac2393921/es-rust.git
cd es-rust
```

2. PostgreSQLデータベースをセットアップします：

```bash
createdb chat_app
```

3. アプリケーションをビルドします：

```bash
cargo build --release
```

4. アプリケーションを実行します：

```bash
RUST_LOG=info DATABASE_URL=postgres://username:password@localhost:5432/chat_app ./target/release/chat-app
```

`username`と`password`は、PostgreSQLのユーザー名とパスワードに置き換えてください。

## 環境変数

アプリケーションは以下の環境変数を使用します：

- `DATABASE_URL`: PostgreSQLデータベースへの接続文字列
- `RUST_LOG`: ログレベル（例：info, debug, error）

## トラブルシューティング

### データベース接続エラー

データベース接続エラーが発生した場合は、以下を確認してください：

1. PostgreSQLサービスが実行中であること
2. `DATABASE_URL`が正しいこと
3. 指定されたデータベースが存在すること
4. 指定されたユーザーがデータベースにアクセスする権限を持っていること

### ポートの競合

ポート8080が既に使用されている場合は、`docker-compose.yml`ファイルまたは実行コマンドで別のポートを指定してください。
