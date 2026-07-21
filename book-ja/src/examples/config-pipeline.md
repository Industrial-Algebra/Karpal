# 設定パイプライン

Alt、Traversable、Foldable、Monoid を使って複数ソースからアプリケーション設定を読み込む。

## 概要

実際のアプリケーションが単一ソースから設定を読み込むことは稀です。環境変数、設定ファイル、ハードコードされたデフォルトがそれぞれ部分的な情報を提供します。この例は以下を行う設定パイプラインを構築します:

- **Alt** を使い、複数の設定ソース (env、ファイル、デフォルト) にまたがるフォールバック連鎖を作る。
- **`do_!`** を使い、依存する参照を接続文字列に逐次化する。
- **Traversable** を使い、ポート番号のオールオアナッシングなバッチ検証を行う。
- **Foldable** と **Monoid** を使い、人間可読な設定サマリーを集約する。

完全なソースは `karpal-std/examples/config_pipeline.rs` にあります。

## 1. ドメイン型

この例はデータベース接続設定を表す単純な `AppConfig` 構造体を定義します:

``` rust
#[derive(Debug, Clone, PartialEq)]
struct AppConfig {
    db_host: String,
    db_port: u16,
    db_name: String,
    max_connections: u16,
    timeout_ms: u64,
}
```

## 2. シミュレートされた設定ソース

三つの関数が異なる設定ソースをシミュレートします。各関数はキーを取り `Option<String>` を返します — ソースがそのキーを知っていれば `Some`、さもなくば `None`。

``` rust
fn from_env(key: &str) -> Option<String> {
    // 環境変数をシミュレート (DB_HOST と DB_PORT のみ設定)
    match key {
        "DB_HOST" => Some("prod-db.example.com".into()),
        "DB_PORT" => Some("5432".into()),
        _ => None,
    }
}

fn from_file(key: &str) -> Option<String> {
    // 設定ファイルをシミュレート (DB_NAME と MAX_CONNECTIONS を持つ)
    match key {
        "DB_NAME" => Some("myapp".into()),
        "MAX_CONNECTIONS" => Some("20".into()),
        _ => None,
    }
}

fn from_default(key: &str) -> Option<String> {
    // すべてのハードコードされたデフォルト
    match key {
        "DB_HOST" => Some("localhost".into()),
        "DB_PORT" => Some("5432".into()),
        "DB_NAME" => Some("app".into()),
        "MAX_CONNECTIONS" => Some("10".into()),
        "TIMEOUT_MS" => Some("5000".into()),
        _ => None,
    }
}
```

単一ソースがすべてのキーを持つわけではありません。環境変数はホストとポートを提供し; 設定ファイルはデータベース名と接続プールサイズを提供し; デフォルトがまだ残っているすべて (タイムアウトを含む) を埋めます。

## 3. Alt フォールバック連鎖

[Alt](../reference/alt-family.md) トレイトは型コンストラクタ上の結合的な「or」演算を提供します。`Option` の場合、`Alt::alt` は最初の `Some` 値を返し、現在のソースが `None` を返せば次のソースにフォールスルーします。

``` rust
/// env を最初に、次にファイル、次にデフォルトを試す。
fn resolve(key: &str) -> Option<String> {
    OptionF::alt(OptionF::alt(from_env(key), from_file(key)), from_default(key))
}
```

これは内側から外側へ読みます: `from_env` を試し、`from_file` にフォールバックし、`from_default` にフォールバックします。`Alt` は結合的であるためグループ化は問いません — 左から右の優先順位だけが重要です。

例えば `resolve("DB_HOST")` は環境から `Some("prod-db.example.com")` を返し、`resolve("TIMEOUT_MS")` は env とファイル (どちらも持たない) をスキップしデフォルトから `Some("5000")` を返します。

## 4. 完全な設定の読み込み

`resolve` があれば、完全な `AppConfig` の読み込みは単純です。文字列フィールドは直接解決され; 数値フィールドは解決されてからパースされます:

``` rust
fn load_config() -> Option<AppConfig> {
    // 各キーを Alt フォールバック連鎖で独立に解決
    let db_host = resolve("DB_HOST")?;
    let db_name = resolve("DB_NAME")?;

    // 数値フィールドは解決してからパース
    let db_port = resolve("DB_PORT").and_then(parse_u16)?;
    let max_connections = resolve("MAX_CONNECTIONS").and_then(parse_u16)?;
    let timeout_ms = resolve("TIMEOUT_MS").and_then(parse_u64)?;

    Some(AppConfig {
        db_host,
        db_port,
        db_name,
        max_connections,
        timeout_ms,
    })
}
```

`?` 演算子はキーが解決できないかパースが失敗すれば関数全体を短絡し、`None` を返します。

## 5. `do_!` による接続文字列

[`do_!` マクロ](../reference/macros.md) はモナド的逐次化を提供します。ここでは三つの解決された値を整形された接続文字列に組み合わせます:

``` rust
fn load_connection_string() -> Option<String> {
    do_! { OptionF;
        host = resolve("DB_HOST");
        port = resolve("DB_PORT");
        name = resolve("DB_NAME");
        Some(format!("postgres://{}:{}/{}", host, port, name))
    }
}
```

各 `name = expr` 行は `Option` をアンラップします。`resolve` のいずれかが `None` を返せばブロック全体が短絡します。最終式が `Some` で包まれた接続文字列を生成します。

## 6. Traversable によるバッチ検証

[Traversable](../reference/foldable-traversable.md) はオールオアナッシング意味論を提供します: 失敗しうる関数をコレクションのすべての要素に適用し、いずれかの要素が失敗すれば結果全体が `None` になります。

``` rust
fn parse_u16(s: String) -> Option<u16> {
    s.parse().ok()
}

fn validate_ports(ports: Vec<&str>) -> Option<Vec<u16>> {
    VecF::traverse::<OptionF, _, _, _>(
        ports.into_iter().map(String::from).collect(),
        parse_u16,
    )
}
```

`VecF::traverse` は各要素に `parse_u16` をマップし結果を集めます。すべての要素がパース成功すれば結果は `Some(vec![...])` です。いずれかの要素が失敗すれば結果は `None` です:

``` rust
let good = validate_ports(vec!["80", "443", "8080"]);
// => Some([80, 443, 8080])

let bad = validate_ports(vec!["80", "not_a_port", "8080"]);
// => None
```

これは失敗をフィルタするより厳密に強力です — すべての値が有効であるか、呼び出しが何か問題を起こしたことを知ることを保証します。

## 7. Foldable と Monoid による設定サマリー

[Foldable](../reference/foldable-traversable.md) は構造的トラバーサルを提供し、[Monoid](../reference/algebraic.md) は単位元と結合的合成を提供します。一緒に、`fold_map` は各要素を変換し結果を結合します:

``` rust
fn summarize_keys(keys: Vec<&str>) -> String {
    VecF::fold_map(
        keys.into_iter().map(String::from).collect::<Vec<_>>(),
        |key| {
            match resolve(&key) {
                Some(val) => format!("  {} = {}\n", key, val),
                None => format!("  {} = <missing>\n", key),
            }
        },
    )
}
```

`String` の場合、Monoid 実装は単位元として空文字列を、結合演算として文字列結合を使います。結果はすべての解決された (または欠落した) 設定キーを要約する単一の文字列です。

## 8. `main` 関数

`main` 関数は各セクションを実行し結果を出力します:

``` rust
fn main() {
    println!("=== Config Pipeline Example ===\n");

    // 1. Alt フォールバック連鎖
    println!("--- Resolving individual keys (Alt fallback) ---");
    println!("DB_HOST:         {:?}", resolve("DB_HOST"));
    println!("DB_PORT:         {:?}", resolve("DB_PORT"));
    println!("DB_NAME:         {:?}", resolve("DB_NAME"));
    println!("MAX_CONNECTIONS: {:?}", resolve("MAX_CONNECTIONS"));
    println!("TIMEOUT_MS:      {:?}", resolve("TIMEOUT_MS"));
    println!("UNKNOWN_KEY:     {:?}", resolve("UNKNOWN_KEY"));

    // 2. 完全な設定の読み込み
    println!("\n--- Loading full config ---");
    match load_config() {
        Some(config) => println!("{:#?}", config),
        None => println!("Failed to load config!"),
    }

    // 3. 独立した参照のための do_!
    println!("\n--- Connection string (do_!) ---");
    println!("{:?}", load_connection_string());

    // 4. Traversable: オールオアナッシング検証
    println!("\n--- Batch port validation (Traversable) ---");
    let good_ports = vec!["80", "443", "8080"];
    let good_result = validate_ports(good_ports.clone());
    println!("Valid ports {:?}: {:?}", good_ports, good_result);

    let bad_ports = vec!["80", "not_a_port", "8080"];
    let bad_result = validate_ports(bad_ports.clone());
    println!("Mixed ports {:?}: {:?}", bad_ports, bad_result);

    // 5. Foldable + Monoid: サマリー
    println!("\n--- Config summary (Foldable + Monoid) ---");
    let summary = summarize_keys(vec![
        "DB_HOST", "DB_PORT", "DB_NAME", "MAX_CONNECTIONS", "TIMEOUT_MS", "MISSING",
    ]);
    print!("{}", summary);
}
```

## 実行

ワークスペースルートから:

``` rust
cargo run -p karpal-std --example config_pipeline
```

## 使用するトレイト

| トレイト                | この例での目的                                        | リファレンス                                                        |
|----------------------|----------------------------------------------------------------|------------------------------------------------------------------|
| `Alt`                | 設定ソースにまたがるフォールバック連鎖                          | [Alt ファミリー](../reference/alt-family.md)                       |
| `Monad` (`do_!` 経由) | 依存する参照の逐次合成                    | [関手ファミリー](../reference/functor-family.md)               |
| `Traversable`        | オールオアナッシングなバッチ検証                                | [Foldable と Traversable](../reference/foldable-traversable.md) |
| `Foldable`           | `fold_map` による構造的トラバーサル                           | [Foldable と Traversable](../reference/foldable-traversable.md) |
| `Monoid`             | `fold_map` の結合演算としての文字列結合 | [半群とモノイド](../reference/semigroup-monoid.md)                |


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
