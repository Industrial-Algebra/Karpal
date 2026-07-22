# 反変関手ファミリー

反変関手とそのコンビネータ: 共変階層の双対。

共変 `Functor` が `F<A>` を `F<B>` に変換するために `A -> B` を消費するのに対し、`Contravariant` 関手は *逆方向* への関数 — `B -> A` — を消費して `F<A>` を `F<B>` に変換します。標準的な例は述語です: 整数の述語と文字列から整数を取り出す関数があれば、文字列の述語を構築できます。

反変ファミリーは共変階層を反映する二つの枝に分かれます:

- **直積側:** `Contravariant` → `Divide` → `Divisible`
- **直和側:** `Contravariant` → `Decide` → `Conclude`

Karpal のすべての反変型は **alloc ゲート** です — 内部で `Box<dyn Fn>` を使うため `std` または `alloc` フィーチャーが必要です。

## 共変階層との双対性

各反変トレイトは対応する共変トレイトの双対です。関係は体系的です: 共変側が値を *生成* するところで、反変側は値を *消費* します。

| 反変            | 共変の双対     | 役割                                              |
|-----------------|----------------|---------------------------------------------------|
| `Contravariant` | `Functor`      | 関数経由で入力型を適応                   |
| `Divide`        | `Apply`        | 入力を部分に分割、各々を独立に扱う |
| `Divisible`     | `Applicative`  | 分割の単位元 (任意のものを受け入れる)         |
| `Decide`        | `Alt`          | 入力を二つのハンドラのいずれかに経路付け              |
| `Conclude`      | `Plus`         | 経路付けの単位元 (非居住入力)          |


### Contravariant

出力ではなく入力でマップする関手。


#### シグネチャ

``` rust
/// 反変関手: 関数 `B -> A` を `F<A> -> F<B>` に持ち上げる。
pub trait Contravariant: HKT {
    fn contramap<A: 'static, B>(
        fa: Self::Of<A>,
        f: impl Fn(B) -> A + 'static,
    ) -> Self::Of<B>;
}
```

型 `F<A>` の値と関数 `B -> A` を与えられると、`contramap` は型 `F<B>` の値を生成します。関数は `Functor::fmap` と比べて *逆* 方向へ進みます。`PredicateF` が関数を `Box<dyn Fn>` の内側に格納するため、`'static` 境界が必要です。

#### 法則


単位律

恒等関数で contramap しても何も変わりません:

``` rust
Contravariant::contramap(fa, |x| x) == fa
```


合成律

合成された関数で contramap するのは、各関数を順に contramap するのと同じです (順序が逆になることに注意):

``` rust
contramap(f . g, fa) == contramap(g, contramap(f, fa))
```


#### 実装

| 型コンストラクタ | `Of<T>`                  | 振る舞い                                                  | フィーチャーゲート     |
|------------------|--------------------------|-----------------------------------------------------------|------------------|
| `PredicateF`     | `Box<dyn Fn(T) -> bool>` | 述語の前に適応関数を事前合成 | `std` または `alloc` |

#### 例

``` rust
use karpal_core::contravariant::{Contravariant, PredicateF};

// 整数の述語
let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);

// 長さを取り出して文字列で動作するように適応
let str_len_positive = PredicateF::contramap(is_positive, |s: &str| s.len() as i32);

assert!(str_len_positive("hello"));  // 長さ 5 > 0
assert!(!str_len_positive(""));      // 長さ 0、> 0 ではない
```


### Divide

Apply の反変類似物 — 入力を部分に分割し各々を独立に扱う。


#### シグネチャ

``` rust
/// Divide: Apply の反変類似物。
///
/// `C` を `(A, B)` に分割する方法と、`A` と `B` 上の反変関手を与えられ、
/// `C` 上の反変関手を生成する。
pub trait Divide: Contravariant {
    fn divide<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> (A, B) + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}
```

`Apply` が *出力* の二つのコンテナを結合するところで、`Divide` は *入力* の二つの消費者を結合します。分割関数 `f` は入力 `C` をペア `(A, B)` に分解し、各部分がそれぞれの消費者で扱われます。

`PredicateF` の場合、`divide` は入力を分割し、**両方の** 副述語がそれぞれの部分を受け入れる場合にのみ `true` を返す述語を生成します。

#### 法則


結合律

分割関数が一貫して入力を分解する限り、左または右に `divide` を入れ子にしても同等の結果になります:

``` rust
divide(f, divide(g, a, b), c) == divide(h, a, divide(i, b, c))
```

ここで `f`、`g`、`h`、`i` は成分を等価に分配する適切な分割関数です。


#### 実装

| 型コンストラクタ | `divide` の振る舞い                            | フィーチャーゲート     |
|------------------|-------------------------------------------------|------------------|
| `PredicateF`     | 入力を分割し、`fa(a) && fb(b)` を返す | `std` または `alloc` |

#### 例

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divide::Divide;

let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
let is_even: Box<dyn Fn(i32) -> bool> = Box::new(|x| x % 2 == 0);

// タプルを成分に分割し、両方の述語をチェック
let both: Box<dyn Fn((i32, i32)) -> bool> =
    PredicateF::divide(|pair: (i32, i32)| pair, is_positive, is_even);

assert!(both((3, 4)));   // 3 > 0 かつ 4 は偶数
assert!(!both((-1, 4))); // -1 は > 0 ではない
assert!(!both((3, 3)));  // 3 は偶数ではない
```


### Divisible

Applicative の反変類似物 — Divide の単位元を追加。


#### シグネチャ

``` rust
/// Divisible: Applicative の反変類似物。
///
/// `divide` の単位元 (pure に類似) となる `conquer` 演算を追加。
pub trait Divisible: Divide {
    fn conquer<A: 'static>() -> Self::Of<A>;
}
```

`conquer` メソッドは任意の入力を受け入れ常に成功する消費者を生成します。これは `divide` の単位元です — `conquer()` 値で divide しても結果に影響しません。

`PredicateF` の場合、`conquer` は常に `true` である述語を返します。

#### 法則


左単位律

左に `conquer()` で divide するのは第二射影の contramap と等価です:

``` rust
divide(f, conquer(), fa) == contramap(snd . f, fa)
```


右単位律

右に `conquer()` で divide するのは第一射影の contramap と等価です:

``` rust
divide(f, fa, conquer()) == contramap(fst . f, fa)
```


#### 実装

| 型コンストラクタ | `conquer` の振る舞い                          | フィーチャーゲート     |
|------------------|------------------------------------------------|------------------|
| `PredicateF`     | `Box::new(|_| true)` を返す — 常に受け入れる | `std` または `alloc` |

#### 例

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divisible::Divisible;

// conquer() はすべてを受け入れる述語を生成
let p: Box<dyn Fn(i32) -> bool> = PredicateF::conquer();
assert!(p(42));
assert!(p(-1));
assert!(p(0));
```

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divide::Divide;
use karpal_core::divisible::Divisible;

// 左単位律: 左に conquer() で divide しても影響しない
let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
let result = PredicateF::divide(
    |a: i32| ((), a),
    PredicateF::conquer::<()>(),
    fa,
);
assert!(result(5));   // 元の述語と等価
assert!(!result(-3));
```


### Decide

Alt の反変類似物 — 入力を二つのハンドラのいずれかに経路付け。


#### シグネチャ

``` rust
/// Decide: Alt の反変類似物。
///
/// `C` を `A` か `B` のいずれかに分割する方法と、`A` と `B` 上の反変
/// 関手を与えられ、`C` 上の反変関手を生成する。
pub trait Decide: Contravariant {
    fn choose<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> Result<A, B> + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}
```

`Divide` が直積ケース (両方の部分に分割) を扱うところで、`Decide` は直和ケース (*一方* のハンドラに経路付け) を扱います。分類関数 `f` は `Result<A, B>` を返し、これは Karpal の `Either` エンコーディングとして機能します: `Ok(a)` は `fa` へ、`Err(b)` は `fb` へ経路付けします。

`PredicateF` の場合、`choose` は入力を分類し、一致する述語に委譲します。

#### 法則


結合律

経路付け関数が一貫して分類する限り、左または右に `choose` を入れ子にしても同等の結果になります:

``` rust
choose(f, choose(g, a, b), c) == choose(h, a, choose(i, b, c))
```

ここで `f`、`g`、`h`、`i` はケースを等価に分配する適切な経路付け関数です。


#### 実装

| 型コンストラクタ | `choose` の振る舞い                                                 | フィーチャーゲート     |
|------------------|----------------------------------------------------------------------|------------------|
| `PredicateF`     | `f` で入力を分類し、`Ok` で `fa`、`Err` で `fb` を適用 | `std` または `alloc` |

#### 例

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::decide::Decide;

let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
let is_short: Box<dyn Fn(String) -> bool> = Box::new(|s| s.len() < 5);

// 入力を分類: 整数は Ok、文字列は Err
let classifier = PredicateF::choose(
    |input: Result<i32, String>| input,
    is_positive,
    is_short,
);

assert!(classifier(Ok(5)));                          // 5 > 0
assert!(!classifier(Ok(-1)));                        // -1 は > 0 ではない
assert!(classifier(Err("hi".to_string())));          // 長さ 2 < 5
assert!(!classifier(Err("hello world".to_string()))); // 長さ 11、< 5 ではない
```


### Conclude

Plus の反変類似物 — Decide の単位元。

`Conclude` は `Plus` の反変双対です。これは `Decide` のための単位元 (`conquer` が `Divide` のための単位元であるのと同様) を提供します。居住されていない入力 (決して生じないケース) を処理します。

`PredicateF` の場合、これは常に `false` を返す述語を生成します — この分岐には決して到達しません。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
