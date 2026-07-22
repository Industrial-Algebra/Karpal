# はじめての利用

このガイドは Rust プロジェクトに Karpal を追加し、その HKT エンコーディングを理解し、コア抽象化である Functor、Monad、快適な `do_!` と `ado_!` マクロを使うまでを案内します。

## 1. インストール

Karpal を使う最も簡単な方法は `karpal-std` クレート経由です。これは他のワークスペースクレートのすべてを単一のプレリュードに再エクスポートします。

`Cargo.toml` に追加:

``` rust
[dependencies]
karpal-std = "0.7"
```

次に Karpal の型とトレイトを使うモジュールの先頭でプレリュードをインポート:

``` rust
use karpal_std::prelude::*;
```

この単一のインポートがすべての型コンストラクタ (`OptionF`、`VecF`、`ResultF` など)、すべてのトレイト (`Functor`、`Applicative`、`Monad`、`Foldable` など)、`do_!` と `ado_!` マクロをもたらします。

### ツールチェーン要件

Karpal は edition 2024 の機能を使うため **nightly Rust** が必要です。リポジトリには正確な nightly バージョンを固定する `rust-toolchain.toml` が含まれているため、Karpal ワークスペース内で作業する場合、Cargo と rustup が自動的に正しいツールチェーンを選択します。

自分のプロジェクトの依存関係として Karpal を利用する場合、プロジェクトも nightly ツールチェーンを使うようにしてください。プロジェクトルートに `rust-toolchain.toml` を作成できます:

``` rust
[toolchain]
channel = "nightly"
```

## 2. 最初の HKT

高階型 (HKT) により、*型コンストラクタ* について抽象化できます — `Option<i32>` のような具体的な型だけでなく、`Option` コンストラクタ自体について。Rust はネイティブには HKT をサポートしませんが、Karpal は Rust 1.65 から安定している Generic Associated Types (GAT) を使ってエンコードします。

コアトレイトは:

``` rust
trait HKT {
    type Of<T>;
}
```

`HKT` を実装する型は **型コンストラクタ** です — パラメータ `T` を与えられると具体的な型を生成するマーカー型です。Karpal はいくつかの組み込みコンストラクタを提供します:

| マーカー型    | `Of<T>` の解決結果 |
|--------------|---------------------|
| `OptionF`    | `Option<T>`         |
| `VecF`       | `Vec<T>`            |
| `ResultF<E>` | `Result<T, E>`      |

したがって `<OptionF as HKT>::Of<i32>` は単に `Option<i32>` です。値レベルでは何も新しいことはありません — 魔法は *型* レベルにあります。これでコンテナの中身ではなく *形* について汎用的な関数を書けます:

``` rust
use karpal_std::prelude::*;

/// Applicative::pure をサポートする任意のコンテナに値を包む。
fn wrap<F: Applicative>(value: i32) -> F::Of<i32> {
    F::pure(value)
}

let opt: Option<i32> = wrap::<OptionF>(42);   // Some(42)
let vec: Vec<i32>    = wrap::<VecF>(42);      // vec![42]
```

呼び出しが型コンストラクタをジェネリックパラメータとして指定してコンテナを選びます。関数本体はどのコンテナが選ばれても同じです。

## 3. 最初の Functor

**Functor** とは、内容に関数をマップできる任意の型コンストラクタです。`Option::map` や `Iterator::map` を使ったことがあれば、アイデアは既に知っています — Karpal は単に統一インターフェースを与えるだけです。

``` rust
use karpal_std::prelude::*;

let result = OptionF::fmap(Some(2), |x| x * 3);
assert_eq!(result, Some(6));

let result = VecF::fmap(vec![1, 2, 3], |x| x + 10);
assert_eq!(result, vec![11, 12, 13]);
```

これは直接 `.map()` を呼ぶのと似ており、具体的なレベルでは同一に振る舞います。違いは `Functor::fmap` が *型コンストラクタ* 上のトレイトメソッドであることです。つまり任意の関手で動作する関数を書けます:

``` rust
use karpal_std::prelude::*;

fn double_inner<F: Functor>(fa: F::Of<i32>) -> F::Of<i32> {
    F::fmap(fa, |x| x * 2)
}

// Option で動作
assert_eq!(double_inner::<OptionF>(Some(5)), Some(10));
assert_eq!(double_inner::<OptionF>(None), None);

// Vec で動作
assert_eq!(double_inner::<VecF>(vec![1, 2, 3]), vec![2, 4, 6]);
```

一つの関数、複数のコンテナ型、コード重複ゼロ。

### Functor の法則

すべての `Functor` 実装は二つの法則を満たさなければなりません。Karpal はプロパティベースのテストでこれらを検証しますが、非公式に知っておく価値があります:

- **単位律:** 恒等関数でマップしても何も変わりません。`F::fmap(fa, |x| x) == fa`
- **合成律:** `f` でマップしてから `g` でマップするのは `|x| g(f(x))` でマップするのと同じです。`F::fmap(F::fmap(fa, f), g) == F::fmap(fa, |x| g(f(x)))`

これらの法則は `fmap` が値を変換することだけを保証します — コンテナ内の要素を追加・削除・並べ替えすることはありません。

## 4. `do_!` によるモナド記法

Rust のモナド的計算はすぐに深く入れ子になった `.and_then()` 連鎖になります。前の値に依存する各ステップがインデントのレベルを一つ追加します:

``` rust
// 入れ子の問題: 各ステップがコードをさらに右に押し出す
fn fetch_dashboard(user_id: &str) -> Option<Dashboard> {
    lookup_user(user_id).and_then(|user| {
        load_preferences(&user).and_then(|prefs| {
            fetch_activity(&user).and_then(|activity| {
                build_dashboard(&user, &prefs, &activity)
            })
        })
    })
}
```

三ステップなら管理できますが、六、七になると読むのが苦痛になります。`do_!` マクロはこれを上から下への束縛シーケンスに平坦化します:

``` rust
use karpal_std::prelude::*;

fn fetch_dashboard(user_id: &str) -> Option<Dashboard> {
    do_! { OptionF;
        user     = lookup_user(user_id);
        prefs    = load_preferences(&user);
        activity = fetch_activity(&user);
        build_dashboard(&user, &prefs, &activity)
    }
}
```

各 `name = expr` 行が右辺のモナド式からアンラップされた値を束縛します。いずれかのステップが `None` (`ResultF` の場合は `Err`) を返せば、ブロック全体が直ちに短絡します。最終式 (束縛のないもの) がブロックの戻り値です。

### 構文リファレンス

``` rust
do_! { TypeConstructor;
    binding1 = monadic_expr1;
    binding2 = monadic_expr2;
    // ... さらなる束縛 ...
    final_monadic_expr
}
```

- 最初のトークンは型コンストラクタ (`OptionF`、`VecF`、`ResultF<E>` など) で、セミコロンが続きます。
- 各束縛は `<-` ではなく `=` を使います。Rust edition 2024 は `<-` をトークンとして予約するため、矢印構文は使えません。
- 最終行は `F::Of<T>` 型の式でなければなりません — これが `do_!` ブロック全体の戻り値です。
- 束縛は前の束縛を参照できます — 各ステップはその上で束縛されたすべての名前にアクセスできます。

### 具体的な例

``` rust
use karpal_std::prelude::*;

fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

let result = do_! { OptionF;
    x = safe_divide(100.0, 4.0);   // Some(25.0)
    y = safe_divide(x, 5.0);       // Some(5.0)
    z = safe_divide(y, 2.0);       // Some(2.5)
    Some(z + 1.0)                   // Some(3.5)
};

assert_eq!(result, Some(3.5));
```

## 5. `ado_!` によるアプリカティブ記法

計算が *独立* しているとき — どれも前のステップの結果を必要としないとき — `do_!` の完全な力は必要ありません。`ado_!` マクロはこのパターンを表現し、独立性を明示します:

``` rust
use karpal_std::prelude::*;

fn load_host() -> Option<&'static str> { Some("localhost") }
fn load_port() -> Option<u16>           { Some(8080) }
fn load_workers() -> Option<usize>      { Some(4) }

let config = ado_! { OptionF;
    host    = load_host();
    port    = load_port();
    workers = load_workers();
    yield format!("{}:{} ({} workers)", host, port, workers)
};

assert_eq!(config, Some("localhost:8080 (4 workers)".to_string()));
```

`yield` 行がすべての束縛された値を最終結果に結合します。`do_!` と異なり、`ado_!` の束縛は互いを参照できません — すべて独立に評価され、結果が最後に結合されます。

### 構文リファレンス

``` rust
ado_! { TypeConstructor;
    binding1 = applicative_expr1;
    binding2 = applicative_expr2;
    // ... さらなる束縛 ...
    yield combining_expression
}
```

- `do_!` と同じ最初のトークンの慣習: 型コンストラクタ、続いてセミコロン。
- 各束縛は `=` を使います。束縛は独立しており、互いに参照してはなりません。
- `yield` 行がすべての束縛値を最終結果に結合します。`yield` の後の式は束縛名の *純粋な* 関数です — 自動的にアプリカティブ文脈に持ち上げられます。
- いずれかの束縛が `None` (または `Err`) に評価されれば、ブロック全体が短絡します。

### `ado_!` と `do_!` の使い分け

| これを使う | 場合                                               |
|----------|----------------------------------------------------|
| `do_!`   | 後のステップが前の結果に依存する (逐次) |
| `ado_!`  | すべてのステップが独立 (並列安全)          |

実際には、`ado_!` は意図を文書化します: 計算にデータ依存がないことを読み手に伝えます。順序が問わない型 (`Option` など) では実行時の振る舞いは同じですが、意味的な明確さが価値を持ちます。

## 6. 証明と外部検証

Karpal のコア抽象化に慣れたら、次の層は法則について明示的に推論することです。

`karpal-proof` は `Proven<P, T>` のような Rust ネイティブの証拠型、`NonEmpty<T>` や `Positive<T>` のような精密化ラッパ、代数法則テストを生成する derive ヘルパを与えます。

`karpal-verify` は次のステップを外側に進めます: 証明オブリゲーションをモデル化し、それらを SMT-LIB2 や Lean 4 にエクスポートし、アーティファクトを書き、検証実行し、CI に適した JSON / Markdown レポートを収集できます。インポートされた証明書は明示的なままであり、暗黙に Rust の証拠証拠にはなりません。

``` rust
use karpal_std::prelude::*;

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);
let report = verify_bundle(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");
assert_eq!(report.obligation_count(), 3);
```

完全なワークフローと信頼モデルは [証明と検証リファレンス](reference/proof-verification.md) を、アーティファクト/レポートのオーケストレーションは [検証 CI ワークフロー](reference/verification-ci.md) ガイドを、シリアライズされた互換性の詳細は [検証スキーマ](reference/verification-schemas.md) ページを参照してください。

## 7. 次のステップ

Karpal をインストールし、コンテナを汎用的にマップし、モナド連鎖を平坦化し、証拠がエコシステムのどこに適合するか理解できたら、次はここへ:

- [**アーキテクチャ**](architecture-full.md) — Functor から Monad までの完全な関手階層と Alt/Alternative 枝を理解する。トレイトがどう関連し、どの型コンストラクタがそれぞれを実装するかを見る。
- [**関手ファミリーリファレンス**](reference/functor-family.md) — `Functor`、`Apply`、`Applicative`、`Chain`、`Monad` の詳細ドキュメント (すべてのメソッドシグネチャと実装メモを含む)。
- [**マクロリファレンス**](reference/macros.md) — `do_!` と `ado_!` の完全な構文とエッジケース (`ResultF` や `VecF` との使用を含む)。
- [**オプティクス**](reference/optics.md) — 合成可能で第一級のフィールドアクセスとパターンマッチングのためのプロ関手ベースの Lens と Prism。
- [**証明と検証リファレンス**](reference/proof-verification.md) — 法則証拠、derive ベースのチェック、Lean/SMT オブリゲーションエクスポート、プロジェクト認識 Lean 実行、診断マッピング、信頼境界、CI 指向検証レポート。
- [**検証 CI ワークフロー**](reference/verification-ci.md) — アーティファクトレイアウト、レポート書き出し、バックエンドポリシー、Lean マニフェスト/サイドカー生成、`karpal-verify` の CI 統合指針。
- [**検証スキーマ**](reference/verification-schemas.md) — スキーマバージョン管理されたレポート、マニフェスト、診断形式とコンシューマの互換性指針。
- [**設定パイプラインの例**](examples/config-pipeline.md) — Functor、Applicative、モナド連鎖を組み合わせて設定ローダを構築する現実的なエンドツーエンドの例。
- [**データ変換の例**](examples/data-transformation.md) — Foldable、Traversable、FunctorFilter を使ってコレクションを汎用的に処理する。
- [**検証ワークフローの例**](examples/verification-workflow.md) — オブリゲーションバンドルから CI レポートファイルと明示的な証明書インポートまでの完全な `karpal-verify` チュートリアル。
- [**検証済みドメイン API の例**](examples/verified-domain-api.md) — `karpal-proof` の `Proven<...>` ベース API と `karpal-verify` の `Certified<...>` インポートがドメイン境界でどう組み合わさるか。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
