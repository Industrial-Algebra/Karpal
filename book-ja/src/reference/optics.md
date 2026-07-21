# オプティクス

プロ関手オプティクス: 第一級のフィールドアクセサとパターンマッチャ。

オプティクスにより、データ構造の一部 — 入れ子になったフィールドや列挙型バリアントの読み取り・書き込み・変換 — に焦点を当てられ、カプセル化を壊しません。Karpal は、それぞれ異なるプロ関手クラスで制約された完全なオプティクス型階層を提供します:

| オプティクス                   | 焦点                   | プロ関手制約 | 読み取り | 書き込み |
|-------------------------|-------------------------|-----------------------|------|-------|
| [Iso](#iso)             | ちょうど 1 (同型) | Profunctor            | はい  | はい   |
| [Lens](#lens)           | ちょうど 1 (フィールド)       | Strong                | はい  | はい   |
| [Prism](#prism)         | 0 または 1 (バリアント)        | Choice                | はい  | はい   |
| [Traversal](#traversal) | 0 から多数               | Traversing            | はい  | はい   |
| [Getter](#getter)       | ちょうど 1 (読み取り専用)   | --                    | はい  | いいえ    |
| [Review](#review)       | 構築のみ       | --                    | いいえ    | はい   |
| [Setter](#setter)       | 変更のみ             | --                    | いいえ    | はい   |
| [Fold](#fold)           | 0 から多数 (読み取り専用)   | --                    | はい  | いいえ    |

オプティクスはサブタイピング階層を形成します — すべての Iso は Lens や Prism として使え、すべての Lens は Getter、Setter、Traversal、Fold として使えます。Karpal はこれらの関係のために明示的な `to_*` 変換メソッドを提供します。

すべてのオプティクス型は `karpal-optics` クレートにあり、[`Optic`](#optic-トレイト) マーカートレイトを実装します。


### Optic

オプティクスファミリーのマーカートレイト。


#### シグネチャ

``` rust
/// すべてのオプティクスのマーカートレイト。
///
/// このトレイトはオプティクスファミリーを単一の分類法の下に統一するために存在する。
/// 具象オプティクス型 (Lens、Prism など) がこのトレイトを実装する。
pub trait Optic {}
```

`Optic` はメソッドを持ちません。型をオプティクスとして分類するためだけに存在し、トレイト境界やドキュメントに有用です。すべての具象オプティクス型が `Optic` を実装します: `Iso`、`Lens`、`ComposedLens`、`Prism`、`Getter`、`ComposedGetter`、`Review`、`Setter`、`Traversal`、`ComposedTraversal`、`Fold`、`ComposedFold`。


### Iso

同型: 二つの表現間の損失のない可逆変換。


#### 構造体定義

``` rust
pub struct Iso<S, T, A, B> {
    forward: fn(&S) -> A,
    backward: fn(B) -> T,
}

pub type SimpleIso<S, A> = Iso<S, S, A, A>;
```

`Iso` は `S` と `A` が同じ情報を運ぶことを証明します。最も強いオプティクスで — `Profunctor` だけを必要とし (`Strong` や `Choice` なし)、任意の他のオプティクス型に変換できます。

#### メソッド

``` rust
impl<S, T, A, B> Iso<S, T, A, B> {
    pub fn new(forward: fn(&S) -> A, backward: fn(B) -> T) -> Self;
    pub fn get(&self, s: &S) -> A;
    pub fn review(&self, b: B) -> T;
    pub fn set(&self, _s: S, b: B) -> T;

    /// プロ関手エンコーディング -- Profunctor だけを必要とする (最弱の制約)。
    pub fn transform<P: Profunctor>(&self, pab: P::P<A, B>) -> P::P<S, T>;

    // 変換
    pub fn to_getter(&self) -> Getter<S, A>;
    pub fn to_review(&self) -> Review<T, B>;
    pub fn to_fold(&self) -> Fold<S, A>;
}

impl<S: Clone, T, A, B> Iso<S, T, A, B> {
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T;
    pub fn to_lens(&self) -> ComposedLens<S, T, A, B>;   // boxed (backward を捕獲)
    pub fn to_setter(&self) -> Setter<S, T, A, B>;
    pub fn to_traversal(&self) -> Traversal<S, T, A, B>;
}
```

#### 法則


ラウンドトリップ (前進-後退)

``` rust
iso.review(iso.get(&s)) == s
```


ラウンドトリップ (後退-前進)

``` rust
iso.get(&iso.review(b)) == b
```


#### 例

``` rust
use karpal_optics::{Iso, SimpleIso};

// 摂氏 <-> 華氏
let temp: SimpleIso<f64, f64> = Iso::new(
    |c: &f64| c * 9.0 / 5.0 + 32.0,  // 前進: C -> F
    |f: f64| (f - 32.0) * 5.0 / 9.0,   // 後退: F -> C
);

assert!((temp.get(&100.0) - 212.0).abs() < 1e-10);
assert!((temp.review(32.0) - 0.0).abs() < 1e-10);

// 「もう一方の」表現で変更
let result = temp.over(0.0, |f| f + 18.0); // 0C に 18F を加算
assert!((result - 10.0).abs() < 1e-10);    // = 10C
```


### Lens

直積型内のフィールドに焦点を当てるための第一級の getter/setter ペア。


#### 構造体定義

``` rust
/// getter/setter 関数ポインタでエンコードされた van Laarhoven スタイルのレンズ。
///
/// `S` -- ソース型、`T` -- 変更されたソース型、
/// `A` -- 焦点型、`B` -- 置換型。
pub struct Lens<S, T, A, B> {
    getter: fn(&S) -> A,
    setter: fn(S, B) -> T,
}

/// `S == T` かつ `A == B` の単純な (単相) レンズ。
pub type SimpleLens<S, A> = Lens<S, S, A, A>;
```

四つの型パラメータは **多相更新** をサポートします: 型 `A` のフィールドを型 `B` の値で置き換え、ソースを `S` から `T` に変更できます。実際にはほとんどのレンズは `S == T` かつ `A == B` の *単純* (単相) です。`SimpleLens` 型エイリアスがこの一般的なケースをカバーします。

#### メソッド

``` rust
impl<S, T, A, B> Lens<S, T, A, B> {
    /// getter と setter から新しいレンズを作成。
    pub fn new(getter: fn(&S) -> A, setter: fn(S, B) -> T) -> Self;

    /// ソースから焦点を抽出。
    pub fn get(&self, s: &S) -> A;

    /// 焦点を置き換え、新しいソースを生成。
    pub fn set(&self, s: S, b: B) -> T;

    /// 別のレンズを連鎖して深く焦点を当て、ComposedLens を生成。
    /// すべての型パラメータが `'static` である必要がある。
    pub fn then<X, Y>(self, inner: Lens<A, B, X, Y>) -> ComposedLens<S, T, X, Y>
    where ...;
}
```

レンズは `.then()` で合成し、深く入れ子になったフィールドに到達できます。`over` メソッドは焦点の値に 関数を適用します。`transform` メソッドは [プロ関手](profunctor-family.md) 抽象化 (`FnP`) を使い、再利用可能な `S -> S` 更新関数を生成します。


### Prism

直和型の単一バリアントに焦点を当てます。`preview` でバリアントを抽出 (マッチしなければ `None`)、`review` でバリアントを構築、`over` でバリアントを変更します (マッチしなければ通過)。これが [プロ関手](profunctor-family.md) の `Choice` クラスによって駆動されます。

法則: `preview(review(b)) == Some(b)` と `review(preview(s))` が元のバリアントを復元 (マッチすれば)。


### Traversal

0 個以上の焦点に焦点を当てます。`over` は各焦点に 関数を適用します。リストの全要素や入れ子構造の全フィールドにアクセスできます。[プロ関手](profunctor-family.md) の `Traversing` クラスによって駆動されます。


### Getter

読み取り専用の焦点 (ちょうど 1 つ)。`get` はありますが `set` はありません。Lens から `to_getter()` で変換できます。


### Review

構築専用の焦点。`review` はありますが `get` はありません。Iso や Prism から変換できます。


### Setter

変更専用の焦点。`over`/`set` はありますが `get` はありません。Lens や Traversal から変換できます。


### Fold

読み取り専用で 0 個以上の焦点。`fold_map` で [Monoid](semigroup-monoid.md) に結果を集約します。Getter と Traversal の読み取り専用の一般化です。


## オプティクスの合成

オプティクスは合成可能です: `Lens . Lens = Lens`、`Lens . Prism = Traversal` など。合成結果は両方の制約の「最大」になります。Karpal は `ComposedLens`、`ComposedGetter`、`ComposedTraversal`、`ComposedFold` 型でこれを表現します。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
