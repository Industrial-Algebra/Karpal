[ホーム](index.md) \> アーキテクチャ

# アーキテクチャ

このページは Karpal の背後にあるコア設計決定を説明します: Rust で高階型をエンコードする方法、完全なトレイト階層、そしてそれらを Rust の型システム内で機能させる Static Land パターン。

## HKT エンコーディング

### 問題

Rust にはネイティブの高階型がありません。`Option` や `Vec` のような *型コンストラクタ* について汎用的なトレイトは書けません — `Option<i32>` のような具体的な型についてのみ書けます。つまり「任意のコンテナ `F` について、`F<A>` で動作する `fmap` をください」と表現する組み込みの方法がありません。

### GAT 解決策

Karpal は型コンストラクタを Generic Associated Type (GAT) を持つトレイトを実装する **マーカー型** としてエンコードします。`HKT` トレイトは型レベル関数として機能します: 型 `T` を与えられると `Self::Of<T>` を生成します。

``` rust
/// GAT による高階型エンコーディング。
///
/// `HKT` を実装する型は型レベル関数として機能する:
/// 型 `T` を与えられると `Self::Of<T>` を生成する。
pub trait HKT {
    type Of<T>;
}
```

各標準コンテナは `Of<T>` を実際の型にマップするゼロサイズのマーカー型を持ちます:

``` rust
/// `Option<T>` の型コンストラクタ。
pub struct OptionF;

impl HKT for OptionF {
    type Of<T> = Option<T>;
}

/// `Result<T, E>` の型コンストラクタ (固定エラー型 `E`)。
pub struct ResultF<E> {
    _marker: PhantomData<E>,
}

impl<E> HKT for ResultF<E> {
    type Of<T> = Result<T, E>;
}

/// `Vec<T>` の型コンストラクタ (alloc ゲート)。
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct VecF;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKT for VecF {
    type Of<T> = Vec<T>;
}
```

### 二パラメータ HKT

二つの型パラメータを持つ型 — バイファンクタとプロ関手 — のため、Karpal は `HKT2` を提供します:

``` rust
/// 二パラメータ型コンストラクタ (バイファンクタ / プロ関手用 HKT)。
pub trait HKT2 {
    type P<A, B>;
}

/// バイファンクタとしての Result (両パラメータが変化)。
pub struct ResultBF;

impl HKT2 for ResultBF {
    type P<A, B> = Result<B, A>;
}

/// バイファンクタとしてのタプル。
pub struct TupleF;

impl HKT2 for TupleF {
    type P<A, B> = (A, B);
}
```

### トレードオフ

| 性質     | 詳細                                                                                                                 |
|--------------|------------------------------------------------------------------------------------------------------------------------|
| 実行時コスト | ゼロ。マーカー型は ZST; すべてのディスパッチはコンパイル時に単相化される。                                            |
| 依存関係 | なし。エンコーディング自体に外部クレートを使わないピュア Rust。                                                       |
| ツールチェーン    | nightly Rust が必要 (edition 2024)。GAT は 1.65 から安定だが、Karpal は `use<>` precise-capture 構文も使う。 |
| エルゴノミクス   | 呼び出しは `value.fmap(...)` ではなく `OptionF::fmap(...)` と書く。これが Static Land スタイル (下記参照)。            |

## トレイト階層

下の図は `karpal-core`、`karpal-profunctor`、`karpal-arrow` に実装された完全なトレイト階層を示します。矢印はスーパートレイトからサブトレイトへ向きます。破線の枠はブランケット実装 (手動 impl が不要) を示します。

![](data:image/svg+xml;base64,PHN2ZyB2aWV3Ym94PSIwIDAgODAwIDc4MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiBzdHlsZT0ibWF4LXdpZHRoOiA4MDBweDsgd2lkdGg6IDEwMCU7IGhlaWdodDogYXV0bzsgZm9udC1mYW1pbHk6ICYjMzk7SW50ZXImIzM5Oywgc2Fucy1zZXJpZjsiPgogICAgICA8ZGVmcz4KICAgICAgICA8bWFya2VyIGlkPSJhcnJvdyIgdmlld2JveD0iMCAwIDEwIDEwIiByZWZ4PSIxMCIgcmVmeT0iNSIgbWFya2Vyd2lkdGg9IjgiIG1hcmtlcmhlaWdodD0iOCIgb3JpZW50PSJhdXRvLXN0YXJ0LXJldmVyc2UiPgogICAgICAgICAgPHBhdGggZD0iTSAwIDAgTCAxMCA1IEwgMCAxMCB6IiBmaWxsPSJ2YXIoLS10ZXh0LW11dGVkLCAjOGI5NDllKSIgLz4KICAgICAgICA8L21hcmtlcj4KICAgICAgICA8c3R5bGU+CiAgICAgICAgICAubm9kZSB7IGZpbGw6IHZhcigtLWJnLWNhcmQsICMxNjFiMjIpOyBzdHJva2U6IHZhcigtLWJvcmRlciwgIzMwMzYzZCk7IHN0cm9rZS13aWR0aDogMS41OyByeDogNjsgcnk6IDY7IH0KICAgICAgICAgIC5ub2RlLWJsYW5rZXQgeyBmaWxsOiB2YXIoLS1iZy1jYXJkLCAjMTYxYjIyKTsgc3Ryb2tlOiB2YXIoLS1ib3JkZXIsICMzMDM2M2QpOyBzdHJva2Utd2lkdGg6IDEuNTsgc3Ryb2tlLWRhc2hhcnJheTogNSAzOyByeDogNjsgcnk6IDY7IH0KICAgICAgICAgIC5ub2RlLWxhYmVsIHsgZmlsbDogdmFyKC0tdGV4dC1wcmltYXJ5LCAjZTZlZGYzKTsgZm9udC1zaXplOiAxMXB4OyBmb250LXdlaWdodDogNjAwOyB0ZXh0LWFuY2hvcjogbWlkZGxlOyBkb21pbmFudC1iYXNlbGluZTogY2VudHJhbDsgfQogICAgICAgICAgLmVkZ2UgeyBzdHJva2U6IHZhcigtLXRleHQtbXV0ZWQsICM4Yjk0OWUpOyBzdHJva2Utd2lkdGg6IDEuMjsgZmlsbDogbm9uZTsgbWFya2VyLWVuZDogdXJsKCNhcnJvdyk7IH0KICAgICAgICAgIC5zZWN0aW9uLWxhYmVsIHsgZmlsbDogdmFyKC0tdGV4dC1tdXRlZCwgIzhiOTQ5ZSk7IGZvbnQtc2l6ZTogOXB4OyBmb250LXdlaWdodDogNTAwOyB0ZXh0LXRyYW5zZm9ybTogdXBwZXJjYXNlOyBsZXR0ZXItc3BhY2luZzogMC4wOGVtOyB9CiAgICAgICAgPC9zdHlsZT4KICAgICAgPC9kZWZzPgoKICAgICAgPCEtLSBTZWN0aW9uIGxhYmVscyAtLT4KICAgICAgPHRleHQgeD0iMTYiIHk9IjE4IiBjbGFzcz0ic2VjdGlvbi1sYWJlbCI+Q292YXJpYW50IChGdW5jdG9yIGZhbWlseSk8L3RleHQ+CiAgICAgIDx0ZXh0IHg9IjE2IiB5PSIyOTgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Db21vbmFkIGZhbWlseTwvdGV4dD4KICAgICAgPHRleHQgeD0iNDMwIiB5PSIyOTgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Db250cmF2YXJpYW50IGZhbWlseTwvdGV4dD4KICAgICAgPHRleHQgeD0iMTYiIHk9IjQ0OCIgY2xhc3M9InNlY3Rpb24tbGFiZWwiPkFsZ2VicmFpYzwvdGV4dD4KICAgICAgPHRleHQgeD0iMjUwIiB5PSI0NDgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Gb2xkYWJsZSAvIFRyYXZlcnNhYmxlPC90ZXh0PgogICAgICA8dGV4dCB4PSIxNiIgeT0iNTQ4IiBjbGFzcz0ic2VjdGlvbi1sYWJlbCI+UHJvZnVuY3RvciBmYW1pbHkgKEhLVDIpPC90ZXh0PgoKICAgICAgPCEtLSA9PT0gQ292YXJpYW50IHJvdyAxOiBGdW5jdG9yIC0+IEFwcGx5IC0+IEFwcGxpY2F0aXZlIC0+IEFsdGVybmF0aXZlIChibGFua2V0KSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iMzAiIHdpZHRoPSI3MiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjUyIiB5PSI0NSI+RnVuY3RvcjwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxMjgiIHk9IjMwIiB3aWR0aD0iNjAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIxNTgiIHk9IjQ1Ij5BcHBseTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyMjgiIHk9IjMwIiB3aWR0aD0iOTAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIyNzMiIHk9IjQ1Ij5BcHBsaWNhdGl2ZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlLWJsYW5rZXQiIHg9IjM1OCIgeT0iMzAiIHdpZHRoPSI5MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjQwMyIgeT0iNDUiPkFsdGVybmF0aXZlPC90ZXh0D4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iODgiIHkxPSI0NSIgeDI9IjEyNiI5Mj0iNDUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxODgiIHkxPSI0NSIgeDI9IjIyNiI5Mj0iNDUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIzMTgiIHkxPSI0NSIgeDI9IjM1NiI5Mj0iNDUiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IENvdmFyaWFudCByb3cgMjogQ2hhaW4sIFNlbGVjdGl2ZSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxMjgiIHk9IjgwIiB3aWR0aD0iNjAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIxNTgiIHk9Ijk1Ij5DaGFpbjwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyMjgiIHk9IjgwIiB3aWR0aD0iOTAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIyNzMiIHk9Ijk1Ij5TZWxlY3RpdmU8L3RleHQ+CiAgICAgIAogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjE1OCIgeTE9IjYwIiB4Mj0iMTU4IiB5Mj0iNzgiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIyNzMiIHkxPSI2MCIgeDI9IjI3MyI5Mj0iNzgiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IENvdmFyaWFudCByb3cgMzogTW9uYWQgKGJsYW5rZXQpID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUtYmxhbmtldCIgeD0iMTY4IiB5PSIxMzAiIHdpZHRoPSI3MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjIwMyIgeT0iMTQ1Ij5Nb25hZDwvdGV4dD4KCiAgICAgIDwhLS0gTGluZXMgZnJvbSBBcHBsaWNhdGl2ZSBhbmQgQ2hhaW4gdG8gTW9uYWQgLS0+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjczIiB5MT0iNjAiIHgyPSIyMzAiIHkyPSIxMjgiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxNTgiIHkxPSIxMTAiIHgyPSIxODUiIHkyPSIxMjgiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEFsdCByb3c6IEZ1bmN0b3IgLT4gQWx0IC0+IFBsdXMgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTI4IiB5PSIxODUiIHdpZHRoPSI0OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE1MiIgeT0iMjAwIj5BbHQ8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMjE2IiB5PSIxODUiIHdpZHRoPSI1MiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI0MiIgeT0iMjAwIj5QbHVzPC90ZXh0D4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjIiIHkxPSI2MCIgeDI9IjE0MCI9Mj0iMTgzIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMTc2IiB5MT0iMjAwIiB4Mj0iMjE0IiB5Mj0iMjAwIj48L2xpbmU+CgogICAgICA8IS0tIFBsdXMgLT4gQWx0ZXJuYXRpdmUgLS0+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjY4IiB5MT0iMjAwIiB4Mj0iNDAzIiB5Mj0iNjIiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEZ1bmN0b3JGaWx0ZXIgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNTAwIiB5PSIzMCIgd2lkdGg9IjEwNSIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjU1MiIgeT0iNDUiPkZ1bmN0b3JGaWx0ZXI8L3RleHQ+CgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9Ijg4IiB5MT0iMzgiIHgyPSI0OTgiIHkyPSIzOCI+PC9saW5lPgoKICAgICAgPCEtLSA9PT0gSW52YXJpYW50ID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjY1MCIgeT0iMzAiIHdpZHRoPSI3NiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjY4OCIgeT0iNDUiPkludmFyaWFudDwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iODgiIHkxPSI1MiI9Mj0iNjQ4IiB5Mj0iNTIiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEJpZnVuY3RvciAoSEtUMikgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNTYwIiB5PSI4MCIgd2lkdGg9IjgwIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjAwIiB5PSI5NSI+QmlmdW5jdG9yPC90ZXh0D4KCiAgICAgIDwhLS0gTmF0dXJhbFRyYW5zZm9ybWF0aW9uIHN0YW5kYWxvbmUgLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NjAiIHk9IjEzMCIgd2lkdGg9IjE0NSIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjYzMiIgeT0iMTQ1Ij5OYXR1cmFsVHJhbnNmb3JtYXRpb248L3RleHQ+CgogICAgICA8IS0tID09PSBDb21vbmFkIGZhbWlseSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iMzEwIiB3aWR0aD0iNjgiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI1MCIgeT0iMzI1Ij5FeHRlbmQ8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTI0IiB5PSIzMTAiIHdpZHRoPSI3OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE2MyIgeT0iMzI1Ij5Db21vbmFkPC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyNDIiIHk9IjMxMCIgd2lkdGg9IjkwIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iMjg3IiB5PSIzMjUiPkNvbW9uYWRFbnY8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMjQyIiB5PSIzNTUiIHdpZHRoPSI5OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI5MSIgeT0iMzcwiPkNvbW9uYWRTdG9yZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyNDIiIHk9IjQwMCIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI5MiIgeT0iNDE1Ij5Db21vbmFkVHJhY2VkPC90ZXh0D4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjIiIHkxPSI2MCI9Mj0iNDAiIHkyPSIzMDgiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI4NCI9Mj0iMTIyIiB5Mj0iMzI1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjAyIiB5MT0iMzI1IiB4Mj0iMjQwIiB5Mj0iMzI1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjAyIiB5MT0iMzMwIiB4Mj0iMjQwIiB5Mj0iMzY1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjAyIiB5MT0iMzM1IiB4Mj0iMjQwIiB5Mj0iNDA1Ij48L2xpbmU+CgogICAgICA8IS0tID09PSBDb250cmF2YXJpYW50IGZhbWlseSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI0MzAiIHk9IjMxMCIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjQ4MCIgeT0iMzI1Ij5Db250cmF2YXJpYW50PC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NzAiIHk9IjMxMCIgd2lkdGg9IjY0IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjAyIiB5PSIzMjUiPkRpdmlkZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2NzQiIHk9IjMxMCIgd2lkdGg9IjcyIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzEwIiB5PSIzMjUiPkRpdmlzaWJsZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NzAiIHk9IjM1NSIgd2lkdGg9IjY0IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjAyIiB5PSIzNzAiPkRlY2lkZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2NzQiIHk9IjM1NSIgd2lkdGg9Ijc4IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzEzIiB5PSIzNzAiPkNvbmNsdWRlPC90ZXh0D4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNTMwIiB5MT0iMzI1IiB4Mj0iNTY4IiB5Mj0iMzI1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjM0IiB5MT0iMzI1IiB4Mj0iNjcyIiB5Mj0iMzI1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNTMwIiB5MT0iMzM1IiB4Mj0iNTY4IiB5Mj0iMzY1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjM0IiB5MT0iMzcwIiB4Mj0iNjcyIiB5Mj0iMzcwIj48L2xpbmU+CgogICAgICA8IS0tID09PSBBbGdlYnJhaWMgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTYiIHk9IjQ2MCIgd2lkdGg9IjgyIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNTciIHk9IjQ3NSI+U2VtaWdyb3VwPC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxMzgiIHk9IjQ2MCIgd2lkdGg9IjY4IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iMTcyIiB5PSI0NzUiPk1vbm9pZDwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iOTgiIHkxPSI0NzUiIHgyPSIxMzYiIHkyPSI0NzUiPjwvbGluZT4KICAgICAgPCEtLSA9PT0gRm9sZGFibGUgLyBUcmF2ZXJzYWJsZSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyODAiIHk9IjQ2MCIgd2lkdGg9IjcyIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iMzE2IiB5PSI0NzUiPkZvbGRhYmxlPC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIzOTIiIHk9IjQ2MCIgd2lkdGg9IjkwIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNDM3IiB5PSI0NzUiPlRyYXZlcnNhYmxlPC90ZXh0D4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMzUyIiB5MT0iNDc1IiB4Mj0iMzkwIiB5Mj0iNDc1Ij48L2xpbmU+CiAgICAgIDwhLS0gPT09IFByb2Z1bmN0b3IgZmFtaWx5ID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjE2IiB5PSI1NjAiIHdpZHRoPSI4NiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjU5IiB5PSI1NzUiPlByb2Z1bmN0b3I8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTQyIiB5PSI1NjAiIHdpZHRoPSI2NCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE3NCIgeT0iNTc1Ij5TdHJvbmc8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTQyIiB5PSI2MDAiIHdpZHRoPSI2NCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE3NCIgeT0iNjE1Ij5DaG9pY2U8L3RleHQ+CgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjEwMiIgeTE9IjU3MCI9Mj0iMTQwIiB5Mj0iNTcwIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMTAyIiB5MT0iNTgwIiB4Mj0iMTQwIiB5Mj0iNjEwIj48L2xpbmU+CgogICAgICA8IS0tID09PSBBcnJvdyBmYW1pbHkgKEhLVDIpID09PSAtLT4KICAgICAgPHRleHQgeD0iMzUwIiB5PSI1NDgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5BcnJvdyBmYW1pbHkgKEhLVDIpPC90ZXh0D4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjM1MCIgeT0iNTYwIiB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNDAwIiB5PSI1NzUiPlNlbWlncm91cG9pZDwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI0OTAiIHk9IjU2MCIgd2lkdGg9IjcyIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNTI2IiB5PSI1NzUiPkNhdGVnb3J5PC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2MDIiIHk9IjU2MCIgd2lkdGg9IjU2IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjMwIiB5PSI1NzUiPkFycm93PC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI3MDAiIHk9IjU0MCIgd2lkdGg9IjkyIiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzQ2IiB5PSI1NTMiPkFycm93Q2hvaWNlPC90ZXh0D4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI3MDAiIHk9IjU3MCIgd2lkdGg9Ijg2IiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzQzIiB5PSI1ODMiPkFycm93QXBwbHk8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNzAwIiB5PSI2MDAiIHdpZHRoPSI4NiIgaGVpZ2h0PSIyNiIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9Ijc0MyIgeT0iNjEzIj5BcnJvd0xvb3A8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNTQwIiB5PSI2MTAiIHdpZHRoPSI4MiIgaGVpZ2h0PSIyNiIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjU4MSIgeT0iNjIzIj5BcnJvd1plcm88L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNjYwIiB5PSI2NDAiIHdpZHRoPSI4MiIgaGVpZ2h0PSIyNiIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjcwMSIgeT0iNjUzIj5BcnJvd1BsdXM8L3RleHQ+CgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjQ1MCI9Mj0iNDg4IiB5Mj0iNTc1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNTYyIiB5MT0iNTc1IiB4Mj0iNjAwIiB5Mj0iNTc1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTY4IiB4Mj0iNjk4IiB5Mj0iNTU1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTc1IiB4Mj0iNjk4IiB5Mj0iNTgwIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTgyIiB4Mj0iNjk4IiB5Mj0iNjEwIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjMwIiB5MT0iNTkwIiB4Mj0iNTcwIiB5Mj0iNjA4Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjIyIiB5MT0iNjM2IiB4Mj0iNjU4IiB5Mj0iNjQ4Ij48L2xpbmU+CiAgICA8L3N2Zz4=)

**凡例:** 実線の枠は標準トレイト。**破線の枠** はブランケット実装を示します — `Monad` は `Applicative` と `Chain` の両方を実装する任意の型に自動的に導出され、`Alternative` は `Applicative + Plus` に導出されます。

## Static Land パターン

Karpal は値のメソッドではなく、**マーカー型上の関連関数** を使います。`some_value.fmap(f)` を呼ぶ代わりに:

``` rust
// Karpal の Static Land スタイル
let result = OptionF::fmap(Some(42), |x| x + 1);

// 値のメソッドスタイルではない (Karpal では不可能)
// let result = Some(42).fmap(|x| x + 1);
```

### なぜこのアプローチか?

Rust の **トレイトコヒーレンス規則** (孤立規則) は、外国のトレイトを外国の型に実装することを防ぎます。`Option` は `std` で定義され、`Functor` は Karpal で定義されるため、`impl Functor for Option<T>` を直接書けません。

マーカー型アプローチはこれを完全に回避します。`OptionF` は *Karpal が所有する* ため、Karpal は自由に任意のトレイトを実装できます。`HKT` GAT が `Of<T>` 関連型を介して実際のコンテナ型へ橋渡しします。

### 他のエコシステムとの比較

| エコシステム          | アプローチ                                                        | トレードオフ                                                     |
|--------------------|-----------------------------------------------------------------|--------------------------------------------------------------|
| Haskell            | HKT サポート付きネイティブ型クラス                             | 理想的なエルゴノミクス; Rust では利用不可                      |
| Scala              | `Kind` 射影付きの暗黙的 / given インスタンス             | 強力だが複雑; JVM ランタイムに依存                  |
| fp-ts (TypeScript) | Static Land — モジュール名前空間内の関数 (`O.map`, `A.map`) | Karpal の設計に最も近い類似物; 同じエルゴノミックトレードオフ |
| **Karpal (Rust)**  | Static Land — マーカー型上の関連関数              | ゼロコスト、型安全、しかし冗長な呼び出し構文                |

## 設計決定

### `no_std` ファースト

`karpal-core`、`karpal-profunctor`、`karpal-arrow` は `std` なしでコンパイルされます。ヒープアロケーションを必要とする型 — `VecF`、`NonEmptyVecF`、`PredicateF`、`StoreF`、`TracedF` — は `alloc` または `std` フィーチャーフラグの背後でゲートされます。これにより Karpal は組み込みや `no_std` 環境で利用可能です。

### nightly edition 2024

ツールチェーンは `rust-toolchain.toml` 経由で nightly に固定されます。GAT 自体は Rust 1.65 で安定化しましたが、Karpal は `use<>` precise-capture 構文 (edition 2024) と `no_std` ビルドの `alloc` フィーチャーゲートにも依存します。nightly 固定によりすべての貢献者が同じコンパイラを使います。

### オプティクスの `fn` ポインタ

`Lens` 構造体はクロージャではなくプレーンな関数ポインタを格納します:

``` rust
pub struct Lens<S, T, A, B> {
    getter: fn(&S) -> A,
    setter: fn(S, B) -> T,
}
```

これにより `Lens` は `Copy` 可能になり、ライフタイムの複雑さを回避します。`Lens::then()` で合成されると、結果はクロージャ合成が `fn` ポインタを生成できないため、代わりに `Box<dyn Fn>` クロージャを使う `ComposedLens` になります。

### `Box<dyn Fn>` の `'static` 境界

内側の表現が boxed クロージャである型 — `PredicateF`、`FnP`、`FnA`、`KleisliF`、`CokleisliF`、`StoreF`、`TracedF` — は `'static` 境界を必要とします。これは Rust の `Box<dyn Fn>` の固有の制限です。結果として、`StoreF` と `TracedF` は汎用 `Functor` トレイト (シグネチャが `'static` 境界を持たない) を実装できません; 代わりに `Extend`/`Comonad` 実装を通じて独自の `fmap` を提供します。

### ブランケット実装

理論が許すところでは、Karpal はボイラープレートを排除するためにブランケット impl を使います:

``` rust
/// Monad: Applicative + Chain で追加メソッドなし (ブランケット impl)。
pub trait Monad: Applicative + Chain {}

impl<F: Applicative + Chain> Monad for F {}
```

`Applicative` と `Chain` の両方を実装する任意のマーカー型は自動的に `Monad` になります。同じパターンが `Alternative` (= `Applicative + Plus`) に適用されます。実装者は原始演算を提供するだけで済みます; 合成された抽象化は無料で付いてきます。

### プロパティベースの法則テスト

Karpal のすべての代数的トレイトには、要求される代数的同一性が成り立つことを検証する [proptest](https://crates.io/crates/proptest) ベースの法則テストがあります。例えば Functor の法則:

``` rust
proptest! {
    #[test]
    fn option_identity(x in any::<Option<i32>>()) {
        // 単位律: fmap(id, fa) == fa
        let result = OptionF::fmap(x.clone(), |a| a);
        prop_assert_eq!(result, x);
    }

    #[test]
    fn option_composition(x in any::<Option<i32>>()) {
        // 合成律: fmap(g . f, fa) == fmap(g, fmap(f, fa))
        let f = |a: i32| a.wrapping_add(1);
        let g = |a: i32| a.wrapping_mul(2);
        let left = OptionF::fmap(x.clone(), |a| g(f(a)));
        let right = OptionF::fmap(OptionF::fmap(x, f), g);
        prop_assert_eq!(left, right);
    }
}
```

このアプローチは、ユニットテストが見逃す微妙なバグ — `Semigroup` 実装の結合律違反や `Alternative` の分配律の失敗など — を捕捉します。

## 証明と検証の階層化

`karpal-proof` と `karpal-verify` は、トレイト階層の上に二つの異なる推論層でこのアーキテクチャを拡張します。

| 層                 | クレート           | アーキテクチャ上の役割                                                                                    |
|-----------------------|-----------------|-------------------------------------------------------------------------------------------------------|
| 内部証拠     | `karpal-proof`  | 法則証拠、書き換え、精密化型を Rust API に直接エンコード                           |
| 外部検証 | `karpal-verify` | Karpal のオブリゲーションを SMT や Lean にブリッジし、結果とインポートされた証明書を明示的に報告 |

### なぜ二つの層か?

`karpal-proof` と `karpal-verify` は意図的に異なる問題を解決します。前者はローカルチェック、トレイト由来の証拠、監査された仮定の後に Rust コードが構造化された証拠を運べるようにします。後者は、ソルバーの結果がコンパイラチェックされた証拠と同じものだと見せかけずに、Karpal が外部の証明器と対話できるようにします。

### `karpal-verify` パイプライン

外部検証層はパイプラインとして編成されます:

1.  目標を `Obligation` または `ObligationBundle` としてモデル化する。
2.  バンドルを SMT-LIB2 スクリプトまたは Lean モジュールにエクスポートする。
3.  定理の同一性、宣言スパン、インポート、エイリアス、パッケージ/プロジェクトスキャフォールドデータなどの構造化 Lean メタデータを添付する。
4.  アーティファクトを書き、`InvocationPlan` 値を構築する。
5.  それらの計画を `VerifierRunner` 経由で実行する (望む場合はプロジェクト認識の `lake env lean` や `lake build` フローを含む)。
6.  ソルバーまたは Lean の出力をパースする (Lean 診断と定理認識失敗マッピングを含む)。
7.  バックエンドごとの明示的な `VerificationPolicy` を通じて成功を解釈する。
8.  結果を CI シリアライズに適した `VerificationReport`、Lean マニフェスト、診断サイドカーに集約する。
9.  成功した証拠を `Proven<P, T>` ではなく `Certified<B, P, T>` としてインポートする。

### 信頼境界

この分離は意図的です。外部証拠は `Certified<...>` を通じて可視的な境界を越え、明示的な `unsafe` 変換経由でのみ `Proven<...>` になります。これによりインポートされた信頼をコードレビューで検索可能に保ち、定理証明器の出力と Rust ネイティブの保証を混同しないようにします。

API の詳細は [証明と検証リファレンス](reference/proof-verification.md) を参照してください。CI 固有の実行/報告指針は [検証 CI ワークフロー](reference/verification-ci.md) を、シリアライズされたアーティファクトの互換性は [検証スキーマ](reference/verification-schemas.md) を、チュートリアル例は [検証ワークフロー](examples/verification-workflow.md) を、`karpal-proof` と `karpal-verify` を組み合わせたドメイン境界の例は [検証済みドメイン API](examples/verified-domain-api.md) を、インポートされた信頼に焦点を当てた設計メモは [信頼モデル](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/phase-12-trust-model.md) を参照してください。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
