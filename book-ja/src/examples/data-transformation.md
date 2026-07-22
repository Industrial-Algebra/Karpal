# データ変換

ETL パイプライン: Functor、Chain、Lens、Foldable、Monoid を使ってレコードをパースし、フィールドを変換し、統計を集約する。

## 概要

この例は生の文字列レコードを型付きトランザクションに処理し、レンズ経由で変換を適用し、モノイダルな畳み込みで結果を集約する小さな ETL (Extract, Transform, Load) パイプラインを構築します。複数の Karpal 抽象化が現実世界のデータ処理ワークフローに自然に合成される仕方を実演します:

- **ドメイン型** — 生の入力レコードと型付き出力構造体、集約用の `Semigroup` と `Monoid` を実装する `Summary` 型。
- **`do_!` と Chain** — 最初の無効フィールドで短絡するモナド的パース。
- **Lens** — 個別の構造体フィールドのための合成可能な getter と setter。
- **Functor** — トランザクションのコレクション上で変換をマップする。
- **Traversable** — 単一レコードが無効でも全体が失敗するオールオアナッシングなバッチパース。
- **Foldable + Monoid** — トランザクションをサマリーに集約する (カテゴリ別内訳を含む)。

## 1. ドメイン型

パイプラインは `RawRecord` から始まります。ここですべてのフィールドは `String` です (CSV パーサや HTTP リクエストから受け取るような)。目標はこれらを強く型付けされた `Transaction` 値にパースすることです。

``` rust
#[derive(Debug, Clone)]
struct RawRecord {
    id: String,
    name: String,
    amount: String,
    category: String,
}

#[derive(Debug, Clone)]
struct Transaction {
    id: u32,
    name: String,
    amount_cents: i64,
    category: String,
}
```

集約のため、トランザクション数とセント単位の総額を追跡する `Summary` 型を定義します。`Semigroup` と `Monoid` を実装することで、手動の蓄積ロジックなしに `fold_map` でサマリーを結合できます。

``` rust
#[derive(Debug, Clone)]
struct Summary {
    count: i64,
    total_cents: i64,
}

impl Semigroup for Summary {
    fn combine(self, other: Self) -> Self {
        Summary {
            count: self.count + other.count,
            total_cents: self.total_cents + other.total_cents,
        }
    }
}

impl Monoid for Summary {
    fn empty() -> Self {
        Summary {
            count: 0,
            total_cents: 0,
        }
    }
}
```

`Semigroup::combine` 実装はカウントと総額を足し合わせます。`Monoid::empty` 値は単位元です — ゼロトランザクションとゼロ総額 — これは任意の畳み込みの出発点として機能します。

## 2. `do_!` (Chain) によるパース

各生レコードは二つのフィールドのパースを必要とします: `id` (`u32`) と `amount` (浮動小数点のドル値をセントに変換)。いずれかのパースが失敗すればレコード全体が無効です。`do_!` マクロによりこの逐次検証は上から下へ読め、`None` で自動的に短絡します:

``` rust
fn parse_record(raw: RawRecord) -> Option<Transaction> {
    let name = raw.name.clone();
    let category = raw.category.clone();
    do_! { OptionF;
        id = raw.id.parse::<u32>().ok();
        amount = raw.amount.parse::<f64>().ok();
        Some(Transaction {
            id,
            name: name.clone(),
            amount_cents: (amount * 100.0) as i64,
            category: category.clone(),
        })
    }
}
```

各 `name = expr` 行は右辺が返す `Option` をアンラップします。`raw.id.parse::<u32>().ok()` が `None` を返せば、amount のパースを試みずにブロック全体が直ちに `None` に評価されます。これが `OptionF` が提供する `Chain` (モナド束縛) の振る舞いです。

レコードのバッチ全体をオールオアナッシング意味論でパースするには、`Traversable` を使います:

``` rust
fn parse_all(records: Vec<RawRecord>) -> Option<Vec<Transaction>> {
    VecF::traverse::<OptionF, _, _, _>(records, parse_record)
}
```

`VecF::traverse` はベクタのすべての要素に `parse_record` を適用し結果を集めます。すべてのレコードがパース成功すれば `Some(vec_of_transactions)` が得られます。単一レコードでも失敗すれば結果全体が `None` になります。これが `Traversable` の「オールオアナッシング」保証です。

## 3. Lens フィールドアクセス

構造体を手動でデストラクトせずに `Transaction` の個別フィールドを変更するため、レンズを定義します。`SimpleLens<S, A>` は単一フィールドの getter (`S -> A`) と setter (`(S, A) -> S`) を提供します:

``` rust
fn amount_lens() -> SimpleLens<Transaction, i64> {
    Lens::new(
        |t: &Transaction| t.amount_cents,
        |t, amount_cents| Transaction {
            amount_cents,
            ..t
        },
    )
}

fn name_lens() -> SimpleLens<Transaction, String> {
    Lens::new(
        |t: &Transaction| t.name.clone(),
        |t, name| Transaction { name, ..t },
    )
}
```

getter クロージャはフィールドを読み; setter クロージャはその一つのフィールドを置き換えた新しい `Transaction` を返し、残りのフィールドをコピーするために Rust の構造体更新構文 (`..t`) を使います。Lens は第一級の値です — 保存し、関数に渡し、`.then()` で合成できます。

## 4. Functor 変換

レンズがあれば、コレクション全体にわたって特定のフィールドを変更する変換関数を定義できます。`VecF::fmap` は `Vec` のすべての要素に 関数を適用し、Lens の `.over()` メソッドは焦点のフィールドに関数を適用します:

``` rust
/// 割引を適用: 金額をパーセンテージで減らす。
fn apply_discount(transactions: Vec<Transaction>, pct: f64) -> Vec<Transaction> {
    let lens = amount_lens();
    VecF::fmap(transactions, |t| {
        lens.over(t, |a| (a as f64 * (1.0 - pct / 100.0)) as i64)
    })
}

/// 名前を大文字に正規化。
fn normalize_names(transactions: Vec<Transaction>) -> Vec<Transaction> {
    let lens = name_lens();
    VecF::fmap(transactions, |t| {
        lens.over(t, |n| n.to_uppercase())
    })
}
```

`apply_discount` は `amount_lens` を使い各トランザクションに到達し `amount_cents` フィールドをスケールします。`normalize_names` は `name_lens` を使い `name` フィールドを大文字にします。どちらの関数も `Transaction` の他のフィールドを知る必要がありません — レンズが読み・変更・書き戻しのボイラープレートを処理します。

## 5. Foldable + Monoid 集約

パイプラインの最終段階はトランザクションをサマリーに集約します。`Summary` は `Monoid` を実装するため、`VecF::fold_map` を使い各トランザクションを単一要素サマリーに変換してからすべてを結合できます:

``` rust
fn summarize(transactions: &[Transaction]) -> Summary {
    VecF::fold_map(transactions.to_vec(), |t| Summary {
        count: 1,
        total_cents: t.amount_cents,
    })
}
```

`fold_map` は各要素を `Summary` (カウント 1 とそのトランザクションの金額) にマップし、`Monoid::empty()` から始めて `Semigroup::combine` ですべてのサマリーを結合します。空のコレクションの場合、単位サマリー (0 トランザクション、0 総額) を返します。

グループ化集約のため、カテゴリで分割し各グループを独立にサマリーします:

``` rust
fn summarize_by_category(transactions: &[Transaction]) -> Vec<(String, Summary)> {
    let mut categories: Vec<String> = transactions.iter().map(|t| t.category.clone()).collect();
    categories.sort();
    categories.dedup();

    categories
        .into_iter()
        .map(|cat| {
            let filtered: Vec<Transaction> = transactions
                .iter()
                .filter(|t| t.category == cat)
                .cloned()
                .collect();
            (cat, summarize(&filtered))
        })
        .collect()
}
```

各カテゴリは同じ `summarize` 関数で計算された独自の `Summary` を持ちます。モノイダル構造により、集約ロジックは (`Semigroup` と `Monoid` で) 一度定義され、どこでも再利用されます。

## 6. 完全なパイプライン

`main` 関数がすべてを結び付けます。サンプルデータを作成し、パースし、変換を適用し、集約結果を出力します:

``` rust
fn main() {
    // サンプルデータ
    let records = vec![
        RawRecord { id: "1".into(), name: "Alice".into(),
                     amount: "99.99".into(), category: "electronics".into() },
        RawRecord { id: "2".into(), name: "Bob".into(),
                     amount: "24.50".into(), category: "books".into() },
        RawRecord { id: "3".into(), name: "Carol".into(),
                     amount: "149.00".into(), category: "electronics".into() },
        RawRecord { id: "4".into(), name: "Dave".into(),
                     amount: "12.75".into(), category: "books".into() },
    ];

    // 1. すべてのレコードをパース (Traversable)
    let transactions = parse_all(records).expect("All records should parse");

    // 2. Functor + Lens で変換
    let discounted = apply_discount(transactions.clone(), 10.0);
    let normalized = normalize_names(transactions.clone());

    // 3. Foldable + Monoid で集約
    let summary = summarize(&transactions);
    let by_category = summarize_by_category(&transactions);

    // 4. 失敗したパースを実演
    let bad_records = vec![
        RawRecord { id: "5".into(), name: "Eve".into(),
                     amount: "50.00".into(), category: "food".into() },
        RawRecord { id: "bad".into(), name: "Frank".into(),
                     amount: "30.00".into(), category: "food".into() },
    ];
    let result = parse_all(bad_records); // None -- "bad" は有効な u32 ではない
}
```

パイプラインは明確な順序で流れます: 生の文字列が型付き値にパースされ、レンズで変換され、モノイダルな畳み込みで集約されます。各段階は異なる Karpal 抽象化を使いますが、すべて同じ標準型上で動作するためシームレスに合成されます。

## 実行

ワークスペースルートから:

``` rust
cargo run -p karpal-std --example data_transformation
```

期待される出力:

    === Data Transformation Example ===

    --- Parse records (Traversable) ---
      #1: Alice - $99.99 (electronics)
      #2: Bob - $24.50 (books)
      #3: Carol - $149.00 (electronics)
      #4: Dave - $12.75 (books)

    --- Apply 10% discount (Functor + Lens) ---
      #1: $89.99
      #2: $22.05
      #3: $134.10
      #4: $11.47

    --- Normalize names (Functor + Lens) ---
      #1: ALICE
      #2: BOB
      #3: CAROL
      #4: DAVE

    --- Overall summary (Foldable + Monoid) ---
      4 transactions, total: $286.24

    --- By category ---
      books: 2 transactions, total: $37.25
      electronics: 2 transactions, total: $248.99

    --- Failed parse (bad data) ---
      parse_all result: None

## 使用するトレイト

| トレイト         | この例での役割                                                         | リファレンス                                                        |
|---------------|------------------------------------------------------------------------------|------------------------------------------------------------------|
| `Semigroup`   | カウントと総額を足して二つの `Summary` 値を結合                    | [半群とモノイド](../reference/semigroup-monoid.md)                |
| `Monoid`      | 畳み込みのための単位 `Summary` (ゼロカウント、ゼロ総額) を提供         | [半群とモノイド](../reference/semigroup-monoid.md)                |
| `Functor`     | `VecF::fmap` がトランザクションにわたり割引と名前正規化を適用     | [関手ファミリー](../reference/functor-family.md)               |
| `Chain`       | 失敗時に短絡する逐次パースのための `do_!` マクロを駆動 | [関手ファミリー](../reference/functor-family.md)               |
| `Foldable`    | `VecF::fold_map` がトランザクションをモノイダルサマリーに集約             | [Foldable と Traversable](../reference/foldable-traversable.md) |
| `Traversable` | `VecF::traverse` がオールオアナッシング意味論ですべてのレコードをパース            | [Foldable と Traversable](../reference/foldable-traversable.md) |
| `Lens`        | `amount_cents` と `name` フィールドのための合成可能な getter/setter     | [オプティクス](../reference/optics.md)                               |


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
