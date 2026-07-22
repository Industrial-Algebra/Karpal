# FunctorFilter と Selective

関手の文脈内でのフィルタリングと条件付き実行。

## FunctorFilter


### FunctorFilter

マッピング中に要素をフィルタできる `Functor`。`filter_map` は `None` を返して要素を破棄する可能性のある関数を適用し、マッピングとフィルタリングを一回の走査で組み合わせます。

#### シグネチャ

``` rust
pub trait FunctorFilter: Functor {
    fn filter_map<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Option<B>) -> Self::Of<B>;

    fn filter<A: Clone>(fa: Self::Of<A>, pred: impl Fn(&A) -> bool) -> Self::Of<A> {
        Self::filter_map(fa, |a| if pred(&a) { Some(a) } else { None })
    }
}
```

#### メソッド

| メソッド              | 説明                                                                                                                         |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------|
| `filter_map(fa, f)` | 各要素に `f` を適用; `f` が `Some` を返すものだけ保持。実装が提供すべき必須メソッドです。 |
| `filter(fa, pred)`  | `pred` が `true` を返す要素だけ保持。デフォルト実装は `filter_map` に委譲。`A: Clone` が必要。          |

#### 法則


- **単位律:** `filter_map(fa, Some) == fa` — (破棄しない) `Some` でのマッピングは何もしません。
- **合成律:** `filter_map(filter_map(fa, f), g) == filter_map(fa, |a| f(a).and_then(g))` — 二回連続の filter-map は一回に融合できます。


#### 実装

| 型コンストラクタ | `Of<A>`     | 備考                                                                      |
|------------------|-------------|----------------------------------------------------------------------------|
| `OptionF`        | `Option<A>` | `Option::and_then` に委譲。`no_std` で利用可能。                    |
| `VecF`           | `Vec<A>`    | 内部的に `Iterator::filter_map` を使用。`alloc` または `std` フィーチャーが必要。 |

`ResultF<E>` は `FunctorFilter` を実装しません。`Result` のフィルタリングはデフォルトのエラー値 (`E: Default`) を必要とするため、制約が強すぎます。

#### 例

``` rust
use karpal_std::prelude::*;

// filter_map: 正の値だけ保持し、二倍にする
let nums = vec![1, -2, 3, -4, 5];
let result = VecF::filter_map(nums, |x| {
    if x > 0 { Some(x * 2) } else { None }
});
assert_eq!(result, vec![2, 6, 10]);

// filter: 偶数だけ保持
let nums = vec![1, 2, 3, 4, 5, 6];
let evens = VecF::filter(nums, |x| x % 2 == 0);
assert_eq!(evens, vec![2, 4, 6]);

// OptionF の場合: filter_map は and_then のように振る舞う
let value = OptionF::filter_map(Some(10), |x| {
    if x > 5 { Some(x * 3) } else { None }
});
assert_eq!(value, Some(30));

let rejected = OptionF::filter_map(Some(2), |x| {
    if x > 5 { Some(x * 3) } else { None }
});
assert_eq!(rejected, None);
```


## Selective


### Selective

エフェクトを条件付きで適用できる `Applicative`。`Selective` は表現力において `Applicative` と `Monad` の中間に位置します: 完全なモナド束縛なしに関手内の値で分岐できます。分岐は `Result<A, B>` でエンコードされ、`Ok(a)` は「関数の適用が必要」、`Err(b)` は「既に解決済み」を意味します。

#### シグネチャ

``` rust
pub trait Selective: Applicative {
    fn select<A, B, F>(fab: Self::Of<Result<A, B>>, ff: Self::Of<F>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}
```

#### メソッド

| メソッド            | 説明                                                                                                                               |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| `select(fab, ff)` | `fab` が `Ok(a)` を含む場合、`ff` 内の関数を適用して `B` を生成。`fab` が `Err(b)` を含む場合、`ff` を無視して `b` を直接返す。 |

#### 法則


- **単位律:** `select(fmap(Err, x), _) == x` — すべての値が既に解決済み (`Err` に包まれている) 場合、関数引数は使われず、元の値がそのまま通過します。


#### 実装

| 型コンストラクタ | `Of<A>`     | 備考                                                                                                        |
|------------------|-------------|--------------------------------------------------------------------------------------------------------------|
| `OptionF`        | `Option<A>` | `None` は伝播。`Some(Ok(a))` は存在すれば関数を適用。`Some(Err(b))` は直接 `Some(b)` を返す。 |

#### 分岐の意味論

最初の引数内の `Result` は選択をエンコードします:

| `fab`          | `ff`      | 結果                                         |
|----------------|-----------|------------------------------------------------|
| `Some(Ok(a))`  | `Some(f)` | `Some(f(a))` — 関数が適用される             |
| `Some(Ok(a))`  | `None`    | `None` — 関数が必要だが不在            |
| `Some(Err(b))` | *(任意)*   | `Some(b)` — 既に解決済み、関数は無視 |
| `None`         | *(任意)*   | `None` — 分岐すべき値がない                 |

#### 例

``` rust
use karpal_std::prelude::*;

// Ok 分岐: 関数が適用される
let result = OptionF::select(
    Some(Ok(3i32)),
    Some(|x: i32| x * 2),
);
assert_eq!(result, Some(6));

// Err 分岐: 既に解決済み、関数は無視
let result = OptionF::select(
    Some(Err(42i32)),
    Some(|_x: i32| 0),
);
assert_eq!(result, Some(42));

// None 伝播: 値がないので結果もない
let result = OptionF::select(
    None::<Result<i32, i32>>,
    Some(|x: i32| x * 2),
);
assert_eq!(result, None);

// Ok 分岐だが関数が利用不可
let result = OptionF::select(
    Some(Ok(3i32)),
    None::<fn(i32) -> i32>,
);
assert_eq!(result, None);
```

#### Selective を使う場面

`Selective` は、関手的パイプライン内で条件付きロジックが必要だが、`Monad` の完全な力は必要ない場合に有用です。分岐は (任意のクロージャではなく) 型 (`Result<A, B>`) でエンコードされるため、選択的計算は静的に解析できます — 計算を実行する前にその構造を調べたいビルドシステムやタスクスケジューラなどのシナリオに適しています。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
