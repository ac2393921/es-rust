# 在庫管理システム MVP版

Event Sourcing/CQRSアーキテクチャで実装する在庫管理システムのMVP（Minimum Viable Product）版。

## 概要

商品登録、入庫、出庫の基本機能を提供し、すべての在庫変動履歴をイベントとして永続化します。

### 目的

- **在庫変動の完全な追跡**: すべての入出庫をイベントとして記録
- **監査ログの提供**: イベントソーシングによる完全な履歴追跡
- **任意時点の状態再構築**: イベントを再生して過去の状態を復元可能
- **ビジネスルールの検証**: マイナス在庫の防止など

## 技術スタック

- **言語**: Rust 1.70+
- **Webフレームワーク**: actix-web 4.x
- **CQRS/ES**: cqrs-es 0.4.x
- **データベース**: PostgreSQL 16
- **非同期ランタイム**: tokio 1.x

## アーキテクチャ

```
┌─────────────────────────────────────────────────────┐
│                   Web API Layer                     │
│              (actix-web REST API)                   │
│                   [未実装]                           │
└─────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────┐
│                  Services Layer                     │
│  ┌──────────────────────┐  ┌────────────────────┐  │
│  │ ProductViewRepository│  │  EventStore        │  │
│  │   (Read Model)       │  │  [計画中]          │  │
│  │   [実装済み]         │  │                    │  │
│  └──────────────────────┘  └────────────────────┘  │
└─────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────┐
│                   Domain Layer                      │
│  ┌──────────────────────────────────────────────┐  │
│  │ Product Aggregate                            │  │
│  │ - 商品登録 (RegisterProduct)                 │  │
│  │ - 入庫記録 (RecordInbound)                   │  │
│  │ - 出庫記録 (RecordOutbound)                  │  │
│  │ [実装済み - テスト10個成功]                  │  │
│  └──────────────────────────────────────────────┘  │
│                                                     │
│  Events: ProductRegistered, InboundRecorded,       │
│          OutboundRecorded                          │
│  Commands: RegisterProduct, RecordInbound,         │
│            RecordOutbound                          │
│  Errors: ProductAlreadyExists, ProductNotFound,    │
│          InvalidQuantity, InsufficientStock        │
└─────────────────────────────────────────────────────┘
```

## 現在の実装状況

### ✅ 完成
- **ドメイン層** (フェーズ0-3)
  - Product Aggregate (商品登録・入庫・出庫のビジネスロジック)
  - イベント定義 (ProductEvent)
  - コマンド定義 (ProductCommand)
  - エラー定義 (ProductError)
  - 単体テスト (10個のテストケース、全て成功)

- **サービス層** (フェーズ4 - 部分的)
  - ProductView (Read Model)
  - ProductViewRepository (インメモリView管理)
  - Query<Product> trait実装

### 🚧 実装中
- **EventStore**: PostgreSQL永続化は複雑性により保留中
  - 代替案: cqrs-es::MemStore使用を検討中

### ⏳ 未実装
- **Web API層** (フェーズ5): REST APIエンドポイント
- **main.rs** (フェーズ6): アプリケーションエントリポイント

**進捗**: 全体の約40%完了

## プロジェクト構造

```
.
├── Cargo.toml              # プロジェクト設定と依存関係
├── docker-compose.yml      # PostgreSQLコンテナ定義
├── Dockerfile              # アプリケーションコンテナ定義
├── .env.example            # 環境変数テンプレート
├── docs/
│   ├── DESIGN.md           # システム設計書
│   └── TODO.md             # 実装タスクリスト
├── migrations/
│   └── 001_init.sql        # Event Storeスキーマ
├── src/
│   ├── domain/             # ドメイン層
│   │   ├── aggregate.rs    # Product Aggregate
│   │   ├── commands.rs     # コマンド定義
│   │   ├── events.rs       # イベント・エラー定義
│   │   └── mod.rs          # モジュール定義
│   ├── services.rs         # サービス層
│   ├── web/                # Web API層 (未実装)
│   │   └── mod.rs
│   ├── lib.rs              # ライブラリエクスポート
│   └── main.rs             # エントリポイント (未実装)
└── tests/
    └── domain_tests.rs     # ドメイン層統合テスト
```

## セットアップ

### 必須環境

- Rust 1.70以上
- Docker & Docker Compose
- PostgreSQL 16 (Dockerで起動)

### インストール手順

1. **リポジトリのクローン**
```bash
git clone https://github.com/ac2393921/es-rust.git
cd es-rust
```

2. **依存関係のインストール**
```bash
cargo build
```

3. **環境変数の設定**
```bash
cp .env.example .env
```

`.env`の内容:
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/inventory_system
RUST_LOG=info
HOST=127.0.0.1
PORT=8080
```

4. **PostgreSQLコンテナの起動**
```bash
docker-compose up -d
```

スキーマは`migrations/001_init.sql`に定義されています。

## 開発コマンド

### テスト実行
```bash
# 全テスト実行
cargo test

# 特定のテストのみ実行
cargo test test_register_product

# テスト出力を表示
cargo test -- --nocapture
```

### コード品質チェック
```bash
# リント実行
cargo clippy

# フォーマットチェック
cargo fmt --check

# フォーマット適用
cargo fmt
```

### ビルド
```bash
# 開発ビルド
cargo build

# リリースビルド
cargo build --release
```

### アプリケーション実行（実装完了後）
```bash
cargo run
```

## ビジネスルール

### 商品登録
- 商品IDは一意である必要がある
- 既に登録済みの商品IDでは登録できない

### 入庫処理
- 商品が登録されている必要がある
- 数量は正の整数のみ
- 入庫により在庫が増加する

### 出庫処理
- 商品が登録されている必要がある
- 数量は正の整数のみ
- 在庫数以上の出庫はできない（マイナス在庫の防止）
- 出庫により在庫が減少する

## テスト

現在10個のテストケースが実装されており、すべて成功しています:

```bash
running 10 tests
test test_register_product_success ... ok
test test_register_product_already_exists ... ok
test test_record_inbound_success ... ok
test test_record_inbound_product_not_found ... ok
test test_record_inbound_invalid_quantity ... ok
test test_record_outbound_success ... ok
test test_record_outbound_product_not_found ... ok
test test_record_outbound_invalid_quantity ... ok
test test_record_outbound_insufficient_stock ... ok
test test_record_outbound_exact_stock ... ok

test result: ok. 10 passed; 0 failed
```

### テストカバレッジ
- **ドメイン層**: 80%以上 (ビジネスロジックをカバー)
- **正常系・異常系**: 両方テスト済み

## API仕様（計画中）

以下のREST APIエンドポイントを実装予定:

### 商品登録
```http
POST /api/products
Content-Type: application/json

{
  "product_id": "uuid",
  "product_code": "P001",
  "product_name": "商品A",
  "unit": "個"
}
```

### 商品照会
```http
GET /api/products/{id}
```

### 全商品照会
```http
GET /api/products
```

### 入庫記録
```http
POST /api/products/{id}/inbound
Content-Type: application/json

{
  "quantity": 100,
  "supplier": "仕入先A"
}
```

### 出庫記録
```http
POST /api/products/{id}/outbound
Content-Type: application/json

{
  "quantity": 50,
  "destination": "出荷先B"
}
```

## トラブルシューティング

### PostgreSQLコンテナが起動しない
```bash
# コンテナログを確認
docker-compose logs postgres

# コンテナ再起動
docker-compose down
docker-compose up -d
```

### ビルドエラーが発生する
```bash
# 依存関係をクリーン
cargo clean
cargo build
```

## ライセンス

MIT License

## 参考資料

- [Event Sourcing - Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [CQRS Pattern - Microsoft](https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs)
- [cqrs-es Documentation](https://docs.rs/cqrs-es/latest/cqrs_es/)
- [actix-web Documentation](https://actix.rs/)

## 開発ロードマップ

- [x] フェーズ0: プロジェクト基盤構築
- [x] フェーズ1-3: ドメイン層実装
- [x] フェーズ4 (部分): サービス層 - ViewRepository
- [ ] フェーズ4 (残): サービス層 - EventStore
- [ ] フェーズ5: Web API層
- [ ] フェーズ6: アプリケーションエントリポイント
- [ ] フェーズ7: 最終品質チェックとドキュメント整備

---

**開発状況**: 🚧 開発中（ドメイン層完成、Web API層実装中）
