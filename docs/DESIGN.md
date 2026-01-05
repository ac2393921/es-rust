# 在庫管理システムMVP版 設計ドキュメント

生成日: 2026-01-06
ジェネレーター: analyzing-requirements

## システム概要

Event Sourcing/CQRSアーキテクチャを用いた在庫管理システムのMVP版。商品の登録、入庫、出庫の基本機能をREST APIで提供し、すべての在庫変動履歴をイベントとして永続化することで、監査ログの完全性と特定時点の在庫状態再構築を実現する。

### 解決する問題
- 在庫変動の完全な追跡と監査ログの提供
- 入出庫履歴の透明性確保
- 在庫数の正確な管理（マイナス在庫の防止）

### ビジネス価値
- Event Sourcingによる完全な監査証跡
- CQRSによる読み書き分離とパフォーマンス最適化
- 過去の任意時点の在庫状態復元が可能

### 対象ユーザー
- 倉庫管理者（入出庫操作）
- 経営者（在庫状況の確認）
- 監査担当者（履歴の確認）

## 機能要件

### 必須機能（MUST have）

#### 1. 商品管理
- **商品登録**: 商品コード、商品名、単位を登録
- **商品照会**: 登録済み商品情報の取得
  - 単一商品の詳細取得
  - 全商品一覧の取得

#### 2. 入庫処理
- 入庫記録の作成（商品コード、数量、入庫日、仕入先）
- 入庫により在庫数が増加
- 入庫履歴の永続化

#### 3. 出庫処理
- 出庫記録の作成（商品コード、数量、出庫日、出荷先/目的）
- 出庫により在庫数が減少
- 在庫不足時のエラーハンドリング
- 出庫履歴の永続化

### オプション機能（NICE to have）
- 在庫照会API（現在の在庫数表示）※今回は実装しない
- 複数倉庫対応
- ロット管理
- 有効期限管理
- 在庫アラート機能

## 非機能要件

### パフォーマンス要件
- レスポンスタイム: 95パーセンタイルで500ms以下
- スループット: 100リクエスト/秒（MVP版のため小規模想定）
- 同時接続数: 50接続

### セキュリティ要件
- **認証方式**: MVP版では未実装（将来的にJWT認証を追加）
- **入力検証**: すべてのAPI入力に対するバリデーション
- **SQLインジェクション対策**: sqlxのパラメータ化クエリを使用

### 可用性・信頼性
- 稼働率目標: 99.0%（MVP版）
- バックアップ戦略: PostgreSQLの日次バックアップ
- 障害復旧時間: RTO 4時間、RPO 24時間

### 保守性
- コードカバレッジ: 80%以上
- ドキュメント整備: API仕様書、アーキテクチャ図
- ログ記録: すべての重要操作のログ出力

## アーキテクチャ設計

### システム構成

```
┌─────────────────┐
│   REST API      │  actix-web
│   (Web Layer)   │
└────────┬────────┘
         │
┌────────▼────────┐
│  CQRS Framework │  cqrs-es
│   (Application) │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼──┐  ┌──▼────┐
│Write │  │ Read  │
│Side  │  │ Side  │
└───┬──┘  └──┬────┘
    │        │
┌───▼────────▼───┐
│  Domain Layer  │
│  (Aggregate)   │
└────────┬────────┘
         │
┌────────▼────────┐
│  Event Store    │  PostgreSQL
│  (Persistence)  │
└─────────────────┘
```

### レイヤーアーキテクチャ

#### 1. プレゼンテーション層（Web Layer）
- **責務**: HTTPリクエストの受付とレスポンス返却
- **技術**: actix-web 4.10.2
- **コンポーネント**:
  - APIハンドラ（register_product, get_product, record_inbound, record_outbound）
  - リクエスト/レスポンスDTO

#### 2. アプリケーション層（Application Layer）
- **責務**: ビジネスフローの制御とCQRS調整
- **技術**: cqrs-es 0.4.2
- **コンポーネント**:
  - CqrsFramework（Command実行とEvent配信）
  - ProductServices（将来的な外部サービス連携）

#### 3. ドメイン層（Domain Layer）
- **責務**: ビジネスルールの実装
- **コンポーネント**:
  - Product Aggregate（状態管理とビジネスロジック）
  - Command（RegisterProduct, RecordInbound, RecordOutbound）
  - Event（ProductRegistered, InboundRecorded, OutboundRecorded）
  - Error（ProductError）

#### 4. インフラストラクチャ層（Infrastructure Layer）
- **責務**: データ永続化と外部リソース管理
- **技術**: PostgreSQL 16、sqlx
- **コンポーネント**:
  - PostgresEventStore（イベント永続化）
  - ProductViewRepository（Read Model管理）

### 技術スタック

- **言語**: Rust (edition 2021)
- **Webフレームワーク**: actix-web 4.10.2
- **Event Sourcing**: cqrs-es 0.4.2
- **非同期ランタイム**: tokio 1.x (full features)
- **データベース**: PostgreSQL 16
- **ORMクエリビルダー**: sqlx 0.8
- **シリアライゼーション**: serde 1.0
- **ユーティリティ**:
  - uuid 1.7.0 (UUIDv4生成)
  - chrono 0.4.35 (日時処理)
  - thiserror 1.0.57 (エラー型定義)
  - anyhow 1.0.79 (エラー伝播)
  - log 0.4.21 + env_logger 0.11.2 (ロギング)
  - dotenv 0.15.0 (環境変数管理)
- **インフラ**: Docker + Docker Compose

### モジュール構成

```
src/
├── main.rs              # アプリケーションエントリポイント
├── lib.rs               # ライブラリのエクスポート
├── domain/              # ドメイン層
│   ├── mod.rs
│   ├── aggregate.rs     # Product Aggregate
│   ├── commands.rs      # ProductCommand
│   └── events.rs        # ProductEvent + ProductError
├── services.rs          # アプリケーション層
│                        # ProductServices, ProductViewRepository,
│                        # PostgresEventStore
└── web/                 # プレゼンテーション層
    └── mod.rs           # REST APIハンドラ
```

## データ設計

### ドメインモデル

#### Product Aggregate

```rust
pub struct Product {
    pub product_id: Option<Uuid>,      // 商品ID（未登録時はNone）
    pub product_code: String,          // 商品コード
    pub product_name: String,          // 商品名
    pub unit: String,                  // 単位（個、箱、kg等）
    pub current_stock: i32,            // 現在の在庫数
    pub created_at: Option<DateTime<Utc>>,  // 作成日時
}
```

### Command定義

```rust
pub enum ProductCommand {
    RegisterProduct {
        product_id: Uuid,
        product_code: String,
        product_name: String,
        unit: String,
    },
    RecordInbound {
        inbound_id: Uuid,
        quantity: i32,
        inbound_date: DateTime<Utc>,
        supplier: String,
    },
    RecordOutbound {
        outbound_id: Uuid,
        quantity: i32,
        outbound_date: DateTime<Utc>,
        destination: String,
    },
}
```

### Event定義

```rust
pub enum ProductEvent {
    ProductRegistered {
        product_id: Uuid,
        product_code: String,
        product_name: String,
        unit: String,
        timestamp: DateTime<Utc>,
    },
    InboundRecorded {
        inbound_id: Uuid,
        quantity: i32,
        inbound_date: DateTime<Utc>,
        supplier: String,
    },
    OutboundRecorded {
        outbound_id: Uuid,
        quantity: i32,
        outbound_date: DateTime<Utc>,
        destination: String,
    },
}
```

### データベーススキーマ

#### events テーブル（Event Store）

```sql
CREATE TABLE events (
    aggregate_type VARCHAR(255) NOT NULL,  -- "Product"
    aggregate_id VARCHAR(255) NOT NULL,    -- product_id (UUID)
    sequence BIGINT NOT NULL,              -- イベントシーケンス番号
    event_type VARCHAR(255) NOT NULL,      -- イベント型名
    event_version VARCHAR(20) NOT NULL,    -- イベントバージョン
    payload JSONB NOT NULL,                -- イベントデータ（JSON）
    metadata JSONB NOT NULL,               -- メタデータ
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id);
CREATE INDEX idx_events_timestamp ON events(timestamp);
```

#### snapshots テーブル（将来的な最適化用）

```sql
CREATE TABLE snapshots (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    last_sequence BIGINT NOT NULL,
    payload JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id)
);
```

### データフロー

#### Write側（Command処理）

```
1. REST API → Command受信
2. CqrsFramework → Aggregateロード（Event再生）
3. Aggregate.handle() → ビジネスルール検証 → Event生成
4. EventStore → Event永続化
5. Aggregate.apply() → 状態更新
6. REST API → レスポンス返却
```

#### Read側（Query処理）

```
1. REST API → Query受信
2. ProductViewRepository → インメモリViewから取得
3. REST API → レスポンス返却
```

#### Event配信（View更新）

```
1. EventStore → Event永続化後
2. CqrsFramework → Eventを配信
3. ProductViewRepository → Viewを更新
```

## API設計

### エンドポイント一覧

| メソッド | エンドポイント | 説明 | 認証 |
|---------|--------------|------|------|
| POST | `/api/products` | 商品登録 | なし |
| GET | `/api/products/{product_id}` | 商品照会（単一） | なし |
| GET | `/api/products` | 商品照会（全件） | なし |
| POST | `/api/products/{product_id}/inbound` | 入庫記録 | なし |
| POST | `/api/products/{product_id}/outbound` | 出庫記録 | なし |

### リクエスト/レスポンス仕様

#### 1. 商品登録

**エンドポイント**: `POST /api/products`

**リクエストボディ**:
```json
{
  "product_code": "P001",
  "product_name": "商品A",
  "unit": "個"
}
```

**レスポンス** (201 Created):
```json
{
  "product_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**エラーレスポンス**:
- 409 Conflict: 商品が既に存在する場合
```json
{
  "error": "ProductAlreadyExists",
  "message": "Product already exists"
}
```

#### 2. 商品照会（単一）

**エンドポイント**: `GET /api/products/{product_id}`

**レスポンス** (200 OK):
```json
{
  "product_id": "550e8400-e29b-41d4-a716-446655440000",
  "product_code": "P001",
  "product_name": "商品A",
  "unit": "個",
  "current_stock": 50,
  "created_at": "2026-01-06T09:00:00Z"
}
```

**エラーレスポンス**:
- 404 Not Found: 商品が存在しない場合

#### 3. 商品照会（全件）

**エンドポイント**: `GET /api/products`

**レスポンス** (200 OK):
```json
[
  {
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "product_code": "P001",
    "product_name": "商品A",
    "unit": "個",
    "current_stock": 50,
    "created_at": "2026-01-06T09:00:00Z"
  },
  {
    "product_id": "660e8400-e29b-41d4-a716-446655440001",
    "product_code": "P002",
    "product_name": "商品B",
    "unit": "箱",
    "current_stock": 30,
    "created_at": "2026-01-06T10:00:00Z"
  }
]
```

#### 4. 入庫記録

**エンドポイント**: `POST /api/products/{product_id}/inbound`

**リクエストボディ**:
```json
{
  "quantity": 100,
  "inbound_date": "2026-01-06T10:00:00Z",
  "supplier": "仕入先A"
}
```

**レスポンス** (201 Created):
```json
{
  "inbound_id": "770e8400-e29b-41d4-a716-446655440002"
}
```

**エラーレスポンス**:
- 404 Not Found: 商品が存在しない場合
- 400 Bad Request: 数量が不正な場合（0以下）
```json
{
  "error": "InvalidQuantity",
  "message": "Quantity must be positive"
}
```

#### 5. 出庫記録

**エンドポイント**: `POST /api/products/{product_id}/outbound`

**リクエストボディ**:
```json
{
  "quantity": 50,
  "outbound_date": "2026-01-06T15:00:00Z",
  "destination": "出荷先B"
}
```

**レスポンス** (201 Created):
```json
{
  "outbound_id": "880e8400-e29b-41d4-a716-446655440003"
}
```

**エラーレスポンス**:
- 404 Not Found: 商品が存在しない場合
- 400 Bad Request: 数量が不正または在庫不足
```json
{
  "error": "InsufficientStock",
  "message": "Current stock: 30, Requested: 50"
}
```

## ビジネスルール

### 商品登録

1. 同じ`product_id`の商品は登録できない（ProductAlreadyExists）
2. 初期在庫数は必ず0
3. `product_code`、`product_name`、`unit`は必須項目

### 入庫処理

1. 商品が未登録の場合はエラー（ProductNotFound）
2. 数量は正の整数のみ許可（InvalidQuantity）
3. 入庫により`current_stock`が増加
4. 入庫履歴は`InboundRecorded`イベントとして永続化

### 出庫処理

1. 商品が未登録の場合はエラー（ProductNotFound）
2. 数量は正の整数のみ許可（InvalidQuantity）
3. 出庫数量が現在の在庫数を超える場合はエラー（InsufficientStock）
4. 出庫により`current_stock`が減少
5. 出庫履歴は`OutboundRecorded`イベントとして永続化

### 在庫計算ロジック

```rust
// Aggregate.apply()での実装
match event {
    ProductEvent::ProductRegistered { .. } => {
        self.current_stock = 0;  // 初期在庫は0
    }
    ProductEvent::InboundRecorded { quantity, .. } => {
        self.current_stock += quantity;  // 入庫で増加
    }
    ProductEvent::OutboundRecorded { quantity, .. } => {
        self.current_stock -= quantity;  // 出庫で減少
    }
}
```

## セキュリティ設計

### 認証・認可
- **MVP版**: 認証なし（将来的にJWT認証を追加予定）
- **将来対応**: Bearer Token認証、ロールベースアクセス制御（RBAC）

### セキュリティ対策

#### 入力検証
- すべてのAPI入力に対するバリデーション
- 数量の範囲チェック（正の整数のみ）
- 文字列長の制限（商品コード、商品名など）

#### SQLインジェクション対策
- sqlxのパラメータ化クエリを使用
- 動的SQL生成の禁止

#### XSS対策
- JSONレスポンスのエスケープ（serdeで自動処理）

#### CSRF対策
- MVP版では未実装（APIのみのため優先度低）

### ログ記録
- すべてのCommand実行をログ出力
- エラー発生時の詳細ログ
- ログレベル: ERROR、WARN、INFO、DEBUG

## パフォーマンス設計

### 最適化戦略

#### キャッシング
- ProductViewRepositoryはインメモリで管理（高速読み取り）
- 将来的にRedisでViewキャッシュを実装

#### 非同期処理
- tokioランタイムで非同期I/O
- PostgreSQL接続プールの活用（sqlx）

#### データベース最適化
- イベントテーブルのインデックス最適化
  - `(aggregate_type, aggregate_id)` 複合インデックス
  - `timestamp` インデックス
- Snapshotによるイベント再生の高速化（将来実装）

### スケーラビリティ

#### 水平スケール
- Statelessな設計（API層）
- ロードバランサーで複数インスタンス起動可能

#### 垂直スケール
- PostgreSQLのリソース増強
- コネクションプール数の調整

## エラー戦略

### エラー分類

#### ドメインエラー（ProductError）

```rust
pub enum ProductError {
    ProductAlreadyExists(String),  // 回復不可能
    ProductNotFound(String),       // 回復不可能
    InvalidQuantity(String),       // 回復不可能（クライアント起因）
    InsufficientStock(String),     // 回復不可能（ビジネスルール違反）
    InvalidOperation(String),      // 回復不可能
}
```

#### 技術的エラー
- データベース接続エラー → リトライ可能
- タイムアウト → リトライ可能
- その他の予期しないエラー → 回復不可能

### エラーハンドリング方針

#### ドメイン層
- `Result<Vec<Event>, ProductError>`を返す
- ビジネスルール違反は明示的なエラー型

#### アプリケーション層
- `anyhow::Error`で技術的エラーを伝播
- リトライロジックの実装（データベースエラー）

#### プレゼンテーション層
- ProductErrorをHTTPステータスコードにマッピング
  - `ProductNotFound` → 404 Not Found
  - `InsufficientStock` → 400 Bad Request
  - `InvalidQuantity` → 400 Bad Request
  - `ProductAlreadyExists` → 409 Conflict
  - その他 → 500 Internal Server Error

### リトライポリシー

- **対象**: データベース接続エラー、一時的なネットワークエラー
- **回数**: 最大3回
- **間隔**: Exponential Backoff（100ms、200ms、400ms）
- **実装**: cqrs-esフレームワークのリトライ機構を活用

### フォールバック処理

- MVP版では未実装
- 将来的にRead側でキャッシュからのフォールバックを実装

### エラーログ・通知

- **ログレベル**:
  - ERROR: 回復不可能なエラー、予期しないエラー
  - WARN: リトライ可能なエラー、ビジネスルール違反
  - INFO: 正常な操作ログ
  - DEBUG: 詳細なトレース情報

- **アラート条件**:
  - エラー率が5%を超える場合
  - データベース接続エラーが連続3回発生

## テスト戦略

### テストピラミッド

```
        E2E (5%)
      ┌─────────┐
      │  1-2本   │
      └─────────┘
    ┌───────────────┐
    │  統合 (15%)    │
    │  API層テスト   │
    └───────────────┘
  ┌─────────────────────┐
  │  単体 (80%)          │
  │  ドメインロジック     │
  └─────────────────────┘
```

### 単体テスト（Domain Layer）

**対象**: Product Aggregate（ビジネスロジック）

**ツール**: `cqrs-es::test::TestFramework`

**テストケース例**:
- `test_register_product_success` - 正常に商品登録
- `test_register_product_already_exists` - 既存商品でエラー
- `test_record_inbound_success` - 正常に入庫記録
- `test_record_inbound_product_not_found` - 商品未登録でエラー
- `test_record_inbound_invalid_quantity` - 数量0以下でエラー
- `test_record_outbound_success` - 正常に出庫記録
- `test_record_outbound_insufficient_stock` - 在庫不足でエラー

**カバレッジ目標**: 80%以上

### 統合テスト（API Layer）

**対象**: REST APIエンドポイント

**ツール**: `actix-web::test`

**テストケース例**:
- `test_api_register_product` - POST /api/products
- `test_api_get_product` - GET /api/products/{id}
- `test_api_record_inbound` - POST /api/products/{id}/inbound
- `test_api_record_outbound` - POST /api/products/{id}/outbound

### E2Eテスト

**対象**: 一連のビジネスフロー

**シナリオ**:
1. 商品登録 → 入庫 → 出庫 → 商品照会
2. 在庫不足シナリオ（出庫拒否）
3. 複数商品の並列操作

### テストデータ戦略

- **フィクスチャ**: テスト用の固定データ（product_id、商品コード等）
- **ファクトリ**: テストデータ生成ヘルパー
- **シード**: 統合テスト用の初期データ

### モック/スタブ方針

- **ドメイン層**: モック不要（純粋な関数）
- **アプリケーション層**: EventStoreのモック実装
- **プレゼンテーション層**: CqrsFrameworkのモック

### CI統合

- **実行タイミング**: Pull Request作成時、mainブランチへのマージ時
- **失敗時の動作**: マージをブロック
- **カバレッジチェック**: 80%未満でビルド失敗

## 開発・運用

### 開発環境セットアップ

```bash
# 1. リポジトリクローン
git clone <repository-url>
cd es-rust

# 2. 環境変数設定
cp .env.example .env

# 3. Dockerコンテナ起動
docker-compose up -d postgres

# 4. マイグレーション実行
sqlx migrate run

# 5. アプリケーション起動
cargo run

# 6. テスト実行
cargo test
```

### 環境変数

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/inventory_system
RUST_LOG=info
HOST=127.0.0.1
PORT=8080
```

### CI/CDパイプライン

```yaml
# GitHub Actions例
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: |
          docker-compose up -d postgres
          cargo test
          cargo clippy
          cargo fmt --check
```

### モニタリング・ロギング

- **監視項目**:
  - API応答時間
  - エラー率
  - データベース接続数
  - メモリ使用量

- **ログ収集**:
  - env_loggerでJSON形式のログ出力
  - 将来的にFluentd/Elasticsearchへ転送

## 制約と前提

### 技術的制約

- Rust edition 2021を使用
- cqrs-es 0.4.2に依存（最新版でない可能性）
- PostgreSQL 16を使用
- Docker環境での実行を前提

### ビジネス制約

- MVP版のため最小限の機能のみ実装
- 認証機能は後回し
- 複数倉庫対応は将来実装

### 依存関係

- **外部サービス**: なし
- **必須ライブラリ**: cqrs-es、actix-web、sqlx、tokio
- **インフラ**: Docker、PostgreSQL

## 参照

- **タスク分解**: planning-tasksスキルでTODO.mdを生成
- **Event Sourcing**: Martin Fowler - https://martinfowler.com/eaaDev/EventSourcing.html
- **CQRS**: Microsoft - https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs
- **cqrs-esドキュメント**: https://docs.rs/cqrs-es/latest/cqrs_es/
