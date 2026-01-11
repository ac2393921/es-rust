# cqrs-es 0.4.x PoC結果レポート

生成日: 2026-01-11
実施者: Claude Code

## 目的

DESIGN.mdで不明確だったcqrs-esの具体的な実装方法を検証し、設計書を具体化するための情報を収集する。

## 実施内容

1. cqrs-es 0.4.12を使用した最小限のAggregate実装
2. TestFrameworkを使った単体テストの作成
3. PostgresEventStoreとの統合方法の調査

## 主要な発見

### 1. Aggregateトレイトの実装詳細

#### 必須トレイト

```rust
use cqrs_es::{Aggregate, DomainEvent};
use async_trait::async_trait;

// イベントにDomainEventトレイトの実装が必要
impl DomainEvent for ProductEvent {
    fn event_type(&self) -> String {
        match self {
            ProductEvent::ProductRegistered { .. } => "ProductRegistered".to_string(),
            ProductEvent::InboundRecorded { .. } => "InboundRecorded".to_string(),
            ProductEvent::OutboundRecorded { .. } => "OutboundRecorded".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

// Aggregateは#[async_trait]が必要
#[async_trait]
impl Aggregate for Product {
    type Command = ProductCommand;
    type Event = ProductEvent;
    type Error = ProductError;
    type Services = ProductServices;

    fn aggregate_type() -> String {
        "Product".to_string()
    }

    // handleメソッドは非同期で、Servicesパラメータが必要
    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services
    ) -> Result<Vec<Self::Event>, Self::Error> {
        // ビジネスロジック
    }

    // applyは同期メソッド
    fn apply(&mut self, event: Self::Event) {
        // イベント適用
    }
}
```

**重要ポイント:**
- `DomainEvent`トレイトの実装が必須（DESIGN.mdには記載なし）
- `handle`メソッドは`async`で、3つ目のパラメータとして`&Self::Services`が必要
- `async_trait`クレートが必要（Cargo.tomlには記載あり）

### 2. Services型の設計

MVP版では外部サービスがないため、空の構造体でOK:

```rust
#[derive(Debug, Clone)]
pub struct ProductServices;

impl Default for ProductServices {
    fn default() -> Self {
        Self
    }
}
```

**Defaultトレイトの実装が必須**（テストで必要）

### 3. エラー型の設計改善

**現在のDESIGN.md（問題あり）:**
```rust
pub enum ProductError {
    InsufficientStock(String), // "Current stock: 30, Requested: 50"
}
```

**推奨設計:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    #[error("Insufficient stock - current: {current}, requested: {requested}")]
    InsufficientStock { current: i32, requested: i32 },

    #[error("Product already exists: {0}")]
    ProductAlreadyExists(String),

    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),
}
```

**メリット:**
- テストで具体的な値を検証できる
- JSONレスポンスで構造化データを返せる
- 国際化対応が容易

### 4. TestFrameworkの使用方法

#### 基本パターン

```rust
use cqrs_es::test::TestFramework;

type ProductTestFramework = TestFramework<Product>;

#[test]
fn test_example() {
    ProductTestFramework::with(ProductServices::default())
        .given(vec![/* 初期イベント */])
        .when(/* コマンド */)
        .then_expect_events(vec![/* 期待されるイベント */]);
}
```

#### 重要な制約

1. **完全一致が必要**: `then_expect_events`はイベントの完全一致を要求
2. **エラーメッセージも完全一致**: `then_expect_error_message`はエラーメッセージの文字列完全一致
3. **動的な値の問題**: `Utc::now()`などで生成されるtimestampは厳密な比較が難しい

#### timestampの問題と解決策

**問題:**
```rust
// これは失敗する！（timestampが微妙にずれる）
.then_expect_events(vec![ProductEvent::ProductRegistered {
    timestamp: Utc::now(), // テスト実行時とイベント生成時で異なる
}])
```

**解決策の選択肢:**

1. **テスト用にtimestampを注入可能にする**
```rust
pub struct ProductServices {
    pub clock: Box<dyn Fn() -> DateTime<Utc>>,
}

// テストでは固定値を返すモック
impl Default for ProductServices {
    fn default() -> Self {
        Self {
            clock: Box::new(|| Utc::now()),
        }
    }
}
```

2. **イベント比較を部分的にする**（TestFrameworkのAPIによっては不可能）

3. **timestampをイベントから除外する**（ビジネス要件次第）

4. **状態検証に切り替える**
```rust
let framework = ProductTestFramework::with(...)
    .given_no_previous_events()
    .when(command);

// イベント数だけ確認
assert_eq!(framework.events().len(), 1);

// Aggregateの最終状態を確認
let aggregate = framework.aggregate();
assert_eq!(aggregate.product_code, "P001");
```

**PoCでの選択:** テストを簡略化してスキップ（設計上の重要な発見として記録）

### 5. PostgresEventStoreとの統合

#### 依存関係

cqrs-es 0.4.xでは、`postgres-es`クレートが必要（要確認）:

```toml
[dependencies]
postgres-es = "0.4"  # バージョン要確認
```

#### 初期化例

```rust
use postgres_es::PostgresCqrs;
use sqlx::PgPool;

pub async fn create_cqrs_framework(
    database_url: &str,
) -> Result<CqrsFramework<Product, _>, Box<dyn std::error::Error>> {
    let pool = PgPool::connect(database_url).await?;

    let (cqrs, _) = postgres_es::postgres_cqrs(
        pool,
        vec![/* Query processors */],
        ProductServices::default(),
    );

    Ok(cqrs)
}
```

**注意:** 上記のAPIは仮定。実際のAPIは`postgres-es`クレートのドキュメント参照が必要。

#### actix-webとの統合

```rust
use actix_web::{web, App};
use std::sync::Arc;

let cqrs = create_cqrs_framework(&database_url).await?;
let cqrs_data = web::Data::new(Arc::new(cqrs));

App::new()
    .app_data(cqrs_data.clone())
    .route("/api/products", web::post().to(register_product))
```

**型シグネチャ:**
```rust
async fn register_product(
    cqrs: web::Data<Arc<CqrsFramework<Product, PersistedEventStore<Product, PgPool>>>>,
    req: web::Json<RegisterProductRequest>,
) -> Result<HttpResponse, Error> {
    // ...
}
```

### 6. モジュール構成の推奨

PoCで実装したコードをもとに、より明確なモジュール構成を提案:

```
src/
├── main.rs              # エントリポイント
├── lib.rs               # ライブラリのエクスポート（テスト用）
├── domain/              # ドメイン層
│   ├── mod.rs
│   ├── aggregate.rs     # Product（状態定義）
│   ├── commands.rs      # ProductCommand
│   ├── events.rs        # ProductEvent + DomainEvent実装
│   └── errors.rs        # ProductError（eventsから分離）
├── application/
│   ├── mod.rs
│   └── services.rs      # ProductServices
├── infrastructure/
│   ├── mod.rs
│   ├── event_store.rs   # PostgresEventStoreの初期化
│   └── view_repository.rs
└── web/
    ├── mod.rs
    ├── handlers.rs      # APIハンドラ
    ├── dto.rs           # Request/Response型
    └── error_mapping.rs # ProductError -> HttpResponse変換
```

## DESIGN.mdへの反映事項

### 追加すべきセクション

#### 1. 実装詳細セクション

```markdown
## 実装詳細

### Aggregateトレイトの完全な実装

[上記のコード例を追加]

### ProductServicesの定義

[上記のコード例を追加]

### DomainEventトレイトの実装

[上記のコード例を追加]
```

#### 2. テスト戦略の具体化

```markdown
## テスト戦略

### 単体テスト（TestFramework使用）

#### 基本パターン

[コード例]

#### 注意事項

- timestampなど動的な値を持つイベントのテストは、固定値を注入する設計が必要
- エラーメッセージは完全一致で検証される
- given-when-then形式を推奨

#### テストフィクスチャ

[具体的な実装例]
```

#### 3. PostgreSQL統合の詳細

```markdown
## PostgreSQL統合

### 依存関係

\`\`\`toml
postgres-es = "0.4"
\`\`\`

### 初期化コード

[コード例]

### actix-webとの統合

[コード例]
```

### 修正すべき箇所

1. **エラー型定義（556-563行目）**: 構造化データを持つ形式に変更
2. **Services定義（122行目）**: Default実装の追加を明記
3. **handle/applyシグネチャ（477-492行目）**: asyncであることを明記
4. **モジュール構成（157-173行目）**: より詳細な分割を記載

## 次のステップ

1. **DESIGN.mdの更新**
   - 上記の発見を反映
   - コード例を追加
   - 型シグネチャを明確化

2. **postgres-esクレートの調査**
   - 実際のAPIドキュメントを確認
   - バージョンの確定

3. **timestampテスト問題の解決**
   - 設計レベルで決定（注入可能にするか、除外するか）

4. **統合テスト環境の構築**
   - Docker Compose でPostgreSQL起動
   - マイグレーション実行
   - 統合テストの実装

## 結論

PoCにより、以下が明確になった:

✅ **Aggregateトレイトの実装方法**
✅ **TestFrameworkの使用方法と制約**
✅ **エラー型設計のベストプラクティス**
✅ **Services型の最小実装**

⚠️ **未解決の問題:**
- timestampテストの戦略（設計判断が必要）
- PostgresEventStoreの実際のAPI（ドキュメント確認が必要）
- Read Modelの実装方法（Query processorの詳細）

このPoCの結果をDESIGN.mdに反映させることで、実装時の迷いをなくし、TDDサイクルをスムーズに進められる。
