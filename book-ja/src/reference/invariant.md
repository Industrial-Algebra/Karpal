# Invariant

Invariant 関手: 両方向を必要とするマッピング。

`Invariant` 関手は、共変 (`Functor`) と反変 (`Contravariant`) の両方の関手を一般化します。`Functor` は内容を変換するために前方関数 `A -> B` だけを必要とし、`Contravariant` は後方関数 `B -> A` だけを必要としますが、`Invariant` 関手は *両方* の方向を必要とします。これにより三つの中で最も一般的なものとなります — 共変または反変のいずれかである型は自動的に invariant でもあります。


### Invariant

共変関数と反変関数の両方でマップする関手。


#### シグネチャ

``` rust
/// Invariant 関手: 共変関数と反変関数の両方でマップする。
///
/// すべての共変 Functor は自明に Invariant です (`g` を無視)。
/// すべての Contravariant も Invariant です (`f` を無視)。
///
/// 法則:
/// - 単位律: `invmap(fa, id, id) == fa`
/// - 合成律: `invmap(fa, g1 . f1, f2 . g2) == invmap(invmap(fa, f1, f2), g1, g2)`
pub trait Invariant: HKT {
    fn invmap<A, B>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B,
        g: impl Fn(B) -> A,
    ) -> Self::Of<B>;
}
```

`invmap` メソッドは関手内の値 (`fa`)、前方関数 `f: A -> B`、後方関数 `g: B -> A` を取り、`Self::Of<B>` 型の新しい値を生成します。前方関数 `f` は出ていく値の変換に使われ、後方関数 `g` は入ってくる値を変換する必要がある型のために用意されます。

#### 法則


単位律

二つの恒等関数でマップしても何も変わりません:

``` rust
F::invmap(fa, |a| a, |a| a) == fa
```

どちらの方向も値を変換しなければ、構造は変化しません。


合成律

二回の `invmap` 呼び出しの合成は、関数を合成して一度 `invmap` を呼ぶのと同じです:

``` rust
F::invmap(fa, |a| g1(f1(a)), |a| f2(g2(a)))
    == F::invmap(F::invmap(fa, f1, f2), g1, g2)
```

前方関数は左から右へ (`g1 . f1`)、後方関数は右から左へ (`f2 . g2`) 合成されます。これは共変と反変のマッピングが逆方向に合成される仕方を反映しています。


#### 実装

| 型コンストラクタ | `invmap` の振る舞い                                                              | フィーチャーゲート |
|------------------|----------------------------------------------------------------------------------|--------------------|
| `OptionF`        | 内側の値を `f` でマップ (`g` は無視); `None` はそのまま                              | なし (`no_std`)    |
| `ResultF<E>`     | `Ok` 値を `f` でマップ (`g` は無視); `Err` は変更なし                                | なし (`no_std`)    |
| `VecF`           | 各要素を `f` でマップ (`g` は無視)                                                      | `std` または `alloc` |
| `IdentityF`      | 値に直接 `f` を適用 (`g` は無視)                                               | なし (`no_std`)    |
| `NonEmptyVecF`   | 先頭と残りの要素を `f` でマップ (`g` は無視)                                        | `std` または `alloc` |
| `EnvF<E>`        | タプルの第二要素を `f` でマップ (`g` は無視); 環境 `E` は変更なし | なし (`no_std`)    |

上記の実装はすべて共変関手であるため、前方関数 `f` だけを使い後方関数 `g` を無視します。共変でも反変でもない真に invariant な型は、両方の関数を必要とします。そのような型は双方向コーデック、シリアライザ/デシリアライザ、同型において実用上現れます。

#### 例

``` rust
use karpal_core::hkt::{OptionF, VecF, IdentityF, EnvF, ResultF};
use karpal_core::invariant::Invariant;

// Option: Some 値をマップ、None はそのまま
let doubled = OptionF::invmap(Some(3), |x| x * 2, |x| x / 2);
assert_eq!(doubled, Some(6));

let nothing = OptionF::invmap(None::<i32>, |x| x * 2, |x| x / 2);
assert_eq!(nothing, None);

// Result: Ok 値をマップ、Err はそのまま
let ok = ResultF::<&str>::invmap(Ok(5), |x| x + 1, |x| x - 1);
assert_eq!(ok, Ok(6));

// Vec: 各要素をマップ
let scaled = VecF::invmap(vec![1, 2, 3], |x| x * 2, |x| x / 2);
assert_eq!(scaled, vec![2, 4, 6]);

// Identity: 関数を直接適用
let result = IdentityF::invmap(42, |x| x + 1, |x| x - 1);
assert_eq!(result, 43);

// Env: 値をマップ、環境は保持
let env = EnvF::<&str>::invmap(("hello", 42), |x| x + 1, |x| x - 1);
assert_eq!(env, ("hello", 43));
```

#### Functor および Contravariant との関係

`Invariant` は変性階層の頂点に位置します。すべての `Functor` (共変関手) は自明に `Invariant` です: 後方関数 `g` を無視して `f` だけを使います。同様に、すべての `Contravariant` 関手も自明に `Invariant` です: 前方関数 `f` を無視して `g` だけを使います。

``` rust
// Functor は g を無視して Invariant を実装できる:
//   fn invmap(fa, f, _g) { F::fmap(fa, f) }
//
// Contravariant は f を無視して Invariant を実装できる:
//   fn invmap(fa, _f, g) { C::contramap(fa, g) }
```

つまり `Invariant` は型コンストラクタの「マップ可能性」の最も一般的な概念を捉えます。共変・反変・そのいずれでもない型について抽象化する必要がある場合 — 例えば値が両方向に流れる汎用コーデックやシリアライズフレームワークを構築する場合 — に有用です。

Karpal では、提供される実装はすべて共変 (すべて `Functor`) であるため `g` を無視します。しかし `Invariant` トレイトは、本当に両方向を必要とするユーザー定義型のために利用可能です。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
