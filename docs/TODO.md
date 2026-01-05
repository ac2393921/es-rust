# TODO: 在庫管理システムMVP版

作成日: 2026-01-06
生成元: planning-tasks
設計書: docs/DESIGN.md

## 概要

Event Sourcing/CQRSアーキテクチャで実装する在庫管理システムのMVP版。商品登録、入庫、出庫の基本機能をREST APIで提供し、すべての在庫変動履歴をイベントとして永続化します。

**目的**:
- 在庫変動の完全な追跡と監査ログの提供
- Event Sourcingによる任意時点の状態再構築
- マイナス在庫の防止（ビジネスルール検証）

**技術スタック**: Rust、actix-web、cqrs-es、PostgreSQL

## 実装タスク

### フェーズ0: プロジェクト基盤構築

- [x] Cargo.tomlの作成（依存関係定義）
- [ ] [STRUCTURAL] docker-compose.ymlの作成
- [ ] [STRUCTURAL] PostgreSQLスキーマ（migrations/001_init.sql）の作成
- [ ] [STRUCTURAL] .env.exampleの作成
- [ ] [STRUCTURAL] Dockerfileの作成
- [ ] [STRUCTURAL] 基本ディレクトリ構造の作成（src/domain/, src/web/, tests/）
- [ ] [CHECK] プロジェクト構成の確認（cargo build が通ること）

### フェーズ1: 商品登録機能の実装

#### 1.1 ドメイン層のテスト作成（RED）

- [ ] [RED] tests/domain/aggregate_tests.rs に商品登録テストを作成
  - `test_register_product_success` - 正常に商品登録できる
  - `test_register_product_already_exists` - 既存商品IDでエラー
- [ ] [CHECK] テスト実行で失敗を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "test: add product registration tests"

#### 1.2 ドメイン層の実装（GREEN）

- [ ] [GREEN] src/domain/events.rs の作成
  - ProductEvent::ProductRegistered の定義
  - ProductError enum の定義（ProductAlreadyExists等）
  - serde traits の実装
- [ ] [GREEN] src/domain/commands.rs の作成
  - ProductCommand::RegisterProduct の定義
- [ ] [GREEN] src/domain/aggregate.rs の作成
  - Product struct の定義
  - Aggregate trait の実装
  - handle() メソッド（RegisterProduct処理）
  - apply() メソッド（ProductRegistered適用）
- [ ] [GREEN] src/domain/mod.rs の作成（モジュール公開）
- [ ] [GREEN] src/lib.rs の作成（ライブラリエクスポート）
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "feat: implement product registration"

#### 1.3 リファクタリング（REFACTOR）

- [ ] [REFACTOR] コード品質の改善
  - 不要なコメントの削除
  - 型シグネチャの明確化
  - エラーメッセージの改善
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [CHECK] lint/format の実行（cargo clippy && cargo fmt）
- [ ] [STRUCTURAL] コミット: "refactor: improve product registration code quality"

### フェーズ2: 入庫処理の実装

#### 2.1 ドメイン層のテスト作成（RED）

- [ ] [RED] tests/domain/aggregate_tests.rs に入庫処理テストを追加
  - `test_record_inbound_success` - 正常に入庫記録できる
  - `test_record_inbound_product_not_found` - 商品未登録でエラー
  - `test_record_inbound_invalid_quantity` - 数量0以下でエラー
- [ ] [CHECK] テスト実行で失敗を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "test: add inbound recording tests"

#### 2.2 ドメイン層の実装（GREEN）

- [ ] [GREEN] src/domain/events.rs に InboundRecorded イベントを追加
- [ ] [GREEN] src/domain/commands.rs に RecordInbound コマンドを追加
- [ ] [GREEN] src/domain/aggregate.rs に入庫処理を追加
  - handle() に RecordInbound 処理を追加
  - apply() に InboundRecorded 処理を追加（在庫数増加）
  - バリデーションロジック（商品存在チェック、数量チェック）
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "feat: implement inbound recording"

#### 2.3 リファクタリング（REFACTOR）

- [ ] [REFACTOR] コード品質の改善
  - バリデーションロジックの整理
  - エラーメッセージの統一
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [CHECK] lint/format の実行（cargo clippy && cargo fmt）
- [ ] [STRUCTURAL] コミット: "refactor: improve inbound recording code quality"

### フェーズ3: 出庫処理の実装

#### 3.1 ドメイン層のテスト作成（RED）

- [ ] [RED] tests/domain/aggregate_tests.rs に出庫処理テストを追加
  - `test_record_outbound_success` - 正常に出庫記録できる
  - `test_record_outbound_product_not_found` - 商品未登録でエラー
  - `test_record_outbound_invalid_quantity` - 数量0以下でエラー
  - `test_record_outbound_insufficient_stock` - 在庫不足でエラー
  - `test_record_outbound_exact_stock` - 在庫数ちょうどで成功
- [ ] [CHECK] テスト実行で失敗を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "test: add outbound recording tests"

#### 3.2 ドメイン層の実装（GREEN）

- [ ] [GREEN] src/domain/events.rs に OutboundRecorded イベントを追加
- [ ] [GREEN] src/domain/commands.rs に RecordOutbound コマンドを追加
- [ ] [GREEN] src/domain/aggregate.rs に出庫処理を追加
  - handle() に RecordOutbound 処理を追加（在庫チェック含む）
  - apply() に OutboundRecorded 処理を追加（在庫数減少）
  - InsufficientStock エラーの実装
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "feat: implement outbound recording with stock validation"

#### 3.3 リファクタリング（REFACTOR）

- [ ] [REFACTOR] コード品質の改善
  - 在庫チェックロジックの明確化
  - エラーメッセージの詳細化
- [ ] [CHECK] テスト実行で成功を確認（cargo test）
- [ ] [CHECK] lint/format の実行（cargo clippy && cargo fmt）
- [ ] [STRUCTURAL] コミット: "refactor: improve outbound recording code quality"

### フェーズ4: サービス層の実装

#### 4.1 ViewRepository とサービスの実装

- [ ] [GREEN] src/services.rs の作成
  - ProductView struct（Read Model）
  - ProductViewRepository struct（インメモリView管理）
  - Query<Product> trait 実装
  - ProductServices struct（将来の外部サービス連携用）
- [ ] [CHECK] 既存テストが成功することを確認（cargo test）
- [ ] [BEHAVIORAL] コミット: "feat: implement product services and view repository"

#### 4.2 PostgreSQL EventStore の実装

- [ ] [GREEN] src/services.rs に PostgresEventStore を追加
  - EventStore<Product> trait 実装
  - load_events() メソッド（イベント読み込み）
  - append_events() メソッド（イベント追加）
  - sqlx を使用した PostgreSQL 接続
- [ ] [CHECK] 手動でPostgreSQL接続確認
- [ ] [BEHAVIORAL] コミット: "feat: implement PostgreSQL event store"

#### 4.3 リファクタリング（REFACTOR）

- [ ] [REFACTOR] サービス層のコード整理
  - エラーハンドリングの統一
  - ログ出力の追加
- [ ] [CHECK] lint/format の実行（cargo clippy && cargo fmt）
- [ ] [STRUCTURAL] コミット: "refactor: improve services layer code quality"

### フェーズ5: Web API層の実装

#### 5.1 商品登録APIの実装

- [ ] [GREEN] src/web/mod.rs の作成
  - register_product ハンドラ
  - RegisterProductRequest struct
  - HTTPエラーマッピング（ProductError → HTTPステータス）
- [ ] [CHECK] 手動でAPI動作確認（curl等）
- [ ] [BEHAVIORAL] コミット: "feat: implement product registration API"

#### 5.2 商品照会APIの実装

- [ ] [GREEN] src/web/mod.rs に照会APIを追加
  - get_product ハンドラ（GET /api/products/{id}）
  - get_all_products ハンドラ（GET /api/products）
- [ ] [CHECK] 手動でAPI動作確認
- [ ] [BEHAVIORAL] コミット: "feat: implement product query APIs"

#### 5.3 入庫APIの実装

- [ ] [GREEN] src/web/mod.rs に入庫APIを追加
  - record_inbound ハンドラ
  - RecordInboundRequest struct
- [ ] [CHECK] 手動でAPI動作確認
- [ ] [BEHAVIORAL] コミット: "feat: implement inbound recording API"

#### 5.4 出庫APIの実装

- [ ] [GREEN] src/web/mod.rs に出庫APIを追加
  - record_outbound ハンドラ
  - RecordOutboundRequest struct
- [ ] [CHECK] 手動でAPI動作確認
- [ ] [BEHAVIORAL] コミット: "feat: implement outbound recording API"

#### 5.5 リファクタリング（REFACTOR）

- [ ] [REFACTOR] Web API層のコード整理
  - レスポンス形式の統一
  - エラーハンドリングの共通化
- [ ] [CHECK] lint/format の実行（cargo clippy && cargo fmt）
- [ ] [STRUCTURAL] コミット: "refactor: improve web API code quality"

### フェーズ6: アプリケーションエントリポイント

#### 6.1 main.rsの実装

- [ ] [GREEN] src/main.rs の作成
  - 環境変数読み込み（dotenv）
  - PostgreSQL接続初期化
  - EventStore初期化
  - ViewRepository初期化
  - CqrsFramework初期化
  - actix-web サーバー起動
- [ ] [CHECK] アプリケーション起動確認（cargo run）
- [ ] [CHECK] Docker環境での動作確認（docker-compose up）
- [ ] [BEHAVIORAL] コミット: "feat: implement application entry point"

#### 6.2 統合テストの実装

- [ ] [RED] tests/integration/ ディレクトリ作成
- [ ] [RED] E2Eテストシナリオの実装
  - 商品登録 → 入庫 → 出庫 → 照会の一連フロー
  - 在庫不足シナリオ（出庫拒否）
- [ ] [CHECK] 統合テスト実行（cargo test --test integration）
- [ ] [BEHAVIORAL] コミット: "test: add end-to-end integration tests"

### フェーズ7: ドキュメントと品質保証

#### 7.1 READMEの作成

- [ ] [STRUCTURAL] README.mdの作成
  - プロジェクト概要
  - 技術スタック
  - セットアップ手順
  - API仕様
  - 使用例（curlコマンド）
- [ ] [STRUCTURAL] コミット: "docs: add comprehensive README"

#### 7.2 最終品質チェック

- [ ] [CHECK] 全テスト実行（cargo test）
- [ ] [CHECK] コードカバレッジ確認（80%以上）
- [ ] [CHECK] lint実行（cargo clippy）
- [ ] [CHECK] format実行（cargo fmt --check）
- [ ] [CHECK] build実行（cargo build --release）
- [ ] [CHECK] Docker環境での動作確認
- [ ] [CHECK] 各APIエンドポイントの手動テスト

#### 7.3 最終リファクタリング（必要に応じて）

- [ ] [REFACTOR] 全体的なコード整理（動作変更なし）
  - 不要なコメントの削除
  - モジュール構成の最適化
  - 命名の統一
- [ ] [CHECK] 全テスト実行で成功確認
- [ ] [STRUCTURAL] コミット: "refactor: final code cleanup"

## 実装ノート

### MUSTルール遵守事項

#### TDDサイクルの厳守
- **RED**: テストを先に書き、失敗を確認
- **GREEN**: テストを通過させる最小限の実装
- **REFACTOR**: テストが通った状態でコードを改善
- **CHECK**: 各フェーズ完了時に lint/format/build を実行

#### コミット規律
- **[BEHAVIORAL]**: 動作変更を伴うコミット（機能追加、バグ修正、テスト追加）
- **[STRUCTURAL]**: 動作変更を伴わないコミット（リファクタリング、フォーマット、ドキュメント）
- コミットメッセージは日本語で具体的に記述

#### Tidy First原則
- 構造変更（STRUCTURAL）と動作変更（BEHAVIORAL）は分離
- リファクタリングは別コミットで実施
- 動作変更前に構造を整理

### テスト戦略

#### 単体テスト（80%）
- ドメイン層のビジネスロジック
- すべてのCommand/Event/Errorパターン
- エッジケースとエラー条件

#### 統合テスト（15%）
- API層のエンドポイント
- リクエスト/レスポンス形式
- HTTPステータスコード

#### E2Eテスト（5%）
- 一連のビジネスフロー
- 複数機能の連携動作

### 開発環境

#### 必須ツール
- Rust 1.70+
- Docker & Docker Compose
- PostgreSQL 16
- cargo (Rustツールチェーン)

#### 環境変数（.env）
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/inventory_system
RUST_LOG=info
HOST=127.0.0.1
PORT=8080
```

#### コマンド
```bash
# テスト実行
cargo test

# リント実行
cargo clippy

# フォーマット
cargo fmt

# ビルド
cargo build

# アプリケーション起動
cargo run

# Docker環境起動
docker-compose up -d
```

### 参照ドキュメント

- **設計書**: docs/DESIGN.md
- **Event Sourcing**: https://martinfowler.com/eaaDev/EventSourcing.html
- **CQRS**: https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs
- **cqrs-es**: https://docs.rs/cqrs-es/latest/cqrs_es/

## 進捗トラッキング

- **フェーズ0**: [ ] 完了（0/7タスク）
- **フェーズ1**: [ ] 完了（0/9タスク）
- **フェーズ2**: [ ] 完了（0/9タスク）
- **フェーズ3**: [ ] 完了（0/10タスク）
- **フェーズ4**: [ ] 完了（0/7タスク）
- **フェーズ5**: [ ] 完了（0/13タスク）
- **フェーズ6**: [ ] 完了（0/5タスク）
- **フェーズ7**: [ ] 完了（0/7タスク）

**全体進捗**: 1/67タスク完了（1.5%）
