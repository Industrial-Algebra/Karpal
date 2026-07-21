# マクロ

モナドおよびアプリカティブ記法マクロ。

Karpal は、入れ子になったモナド的・アプリカティブな計算を読みやすい上から下への束縛シーケンスに平坦化する二つのマクロを提供します。どちらのマクロも束縛に `=` を使います (`<-` は Rust edition 2024 で予約されているため使いません)。


### `do_!`

モナド的 do 記法。逐次的な束縛を入れ子の `Chain::chain` 呼び出しに脱糖します。


#### 構文

``` rust
do_! { F;
    x = monadic_expr_1;
    y = monadic_expr_2;   // x を参照できる
    // ... さらなる束縛 ...
    final_monadic_expr     // 束縛のない裸の式
}
```

- 最初のトークン `F` は型コンストラクタ (`OptionF`、`VecF`、`ResultF<E>` など) で、セミコロンが続きます。
- 各束縛は `=` を使います。後の束縛は前に束縛された名前を参照できます — ステップは逐次です。
- 最後の行は `F::Of<T>` 型の裸の式です。これが `do_!` ブロック全体の戻り値です。
- いずれかのステップが短絡する値 (`None`、`Err(_)`) を生成すると、ブロック全体が直ちに短絡します。

#### 展開

各 `x = expr;` 束縛は `Chain::chain` 呼び出しに脱糖されます。マクロは再帰的に展開されます:

``` rust
// これが:
do_! { F;
    x = expr_a;
    y = expr_b;
    expr_c
}

// こう展開される:
<F as Chain>::chain(expr_a, |x| {
    <F as Chain>::chain(expr_b, |y| {
        expr_c
    })
})
```

束縛のない単一の裸の式はそのまま返されます:

``` rust
do_! { F; some_expr }
// 展開:
some_expr
```

#### 要件

型コンストラクタ `F` は `Chain` を実装しなければなりません (したがって `Apply` と `Functor` も)。実際には、`Monad` を実装する任意の型がこの要件を満たします。`Monad` は `Applicative + Chain` 上のブランケットトレイトだからです。

#### 例

##### OptionF — 短絡付きの逐次計算

``` rust
use karpal_std::prelude::*;

let result = do_! { OptionF;
    x = Some(1);
    y = Some(x + 1);       // y は x に依存
    OptionF::pure(x + y)   // 最終式が Some で包む
};
assert_eq!(result, Some(3));
```

##### OptionF — None での短絡

``` rust
use karpal_std::prelude::*;

let result: Option<i32> = do_! { OptionF;
    x = Some(1);
    _y = None::<i32>;     // ここで短絡
    OptionF::pure(x)       // 到達しない
};
assert_eq!(result, None);
```

##### OptionF — 単一式 (束縛なし)

``` rust
use karpal_std::prelude::*;

let result = do_! { OptionF;
    Some(42)
};
assert_eq!(result, Some(42));
```

##### ResultF — 失敗しうる操作の連鎖

``` rust
use karpal_std::prelude::*;

fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>().map_err(|e| e.to_string())
}

let result = do_! { ResultF<String>;
    port = parse_port("8080");
    validated = if port > 0 { Ok(port) } else { Err("invalid".into()) };
    Ok(format!("port={}", validated))
};
assert_eq!(result, Ok("port=8080".to_string()));
```

##### VecF — リスト内包表記 (直積)

``` rust
use karpal_std::prelude::*;

let result = do_! { VecF;
    x = vec![1, 2];
    y = vec![10, 20];
    VecF::pure(x + y)
};
assert_eq!(result, vec![11, 21, 12, 22]);
```


### `ado_!`

アプリカティブ do 記法。独立した束縛を収集し、`Apply::ap` と `Functor::fmap` で組み合わせます。


#### 構文

``` rust
ado_! { F;
    x = applicative_expr_1;
    y = applicative_expr_2;
    // ... 最大 4 つの束縛 ...
    yield combining_expression
}
```

- `do_!` と同じ最初のトークンの慣習: 型コンストラクタ、続いてセミコロン。
- 各束縛は `=` を使います。束縛は **独立しており**、互いに参照してはなりません。
- `yield` キーワードが結合式を導入します。この式は束縛された名前の純粋関数です — 自動的にアプリカティブ文脈に持ち上げられます。
- 1 から 4 つの束縛をサポートします。
- いずれかの束縛が短絡する値 (`None`、`Err(_)`) に評価されると、ブロック全体が短絡します。

#### 展開

展開は束縛の数に依存します。束縛が一つの場合、マクロは `Functor::fmap` を使います。二つ以上の場合、カリー化されたクロージャを構築し `Apply::ap` で適用します:

##### 1 つの束縛

``` rust
// これが:
ado_! { F; x = expr; yield body }

// こう展開される:
<F as Functor>::fmap(expr, |x| body)
```

##### 2 つの束縛

``` rust
// これが:
ado_! { F; x = e1; y = e2; yield body }

// こう展開される:
<F as Apply>::ap(
    <F as Functor>::fmap(e1, |x| move |y| body),
    e2,
)
```

##### 3 つの束縛

``` rust
// これが:
ado_! { F; x = e1; y = e2; z = e3; yield body }

// こう展開される:
<F as Apply>::ap(
    <F as Apply>::ap(
        <F as Functor>::fmap(e1, |x| move |y| move |z| body),
        e2,
    ),
    e3,
)
```

##### 4 つの束縛

``` rust
// これが:
ado_! { F; a = e1; b = e2; c = e3; d = e4; yield body }

// こう展開される:
<F as Apply>::ap(
    <F as Apply>::ap(
        <F as Apply>::ap(
            <F as Functor>::fmap(e1, |a| move |b| move |c| move |d| body),
            e2,
        ),
        e3,
    ),
    e4,
)
```

#### 要件

型コンストラクタ `F` は `Applicative` を実装しなければなりません (したがって `Apply` と `Functor` も)。`do_!` と異なり `Chain` を必要と **しません** — アプリカティブ計算はモナド計算より厳密に弱く、それが要点です: 逐次依存の不在を表現します。

#### 例

##### OptionF — 単一束縛 (fmap)

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(5);
    yield x * 2
};
assert_eq!(result, Some(10));
```

##### OptionF — 二つの独立した値の結合

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(1);
    y = Some(2);
    yield x + y
};
assert_eq!(result, Some(3));
```

##### OptionF — None での短絡

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(1);
    y = None::<i32>;
    yield x + y
};
assert_eq!(result, None);
```

##### OptionF — 三つの値の結合

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(1);
    y = Some(2);
    z = Some(3);
    yield x + y + z
};
assert_eq!(result, Some(6));
```

##### OptionF — 四つの値の結合

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    a = Some(1);
    b = Some(2);
    c = Some(3);
    d = Some(4);
    yield a + b + c + d
};
assert_eq!(result, Some(10));
```

##### VecF — アプリカティブでの直積

``` rust
use karpal_std::prelude::*;

let result = ado_! { VecF;
    x = vec![1, 2];
    y = vec![10, 20];
    yield x + y
};
assert_eq!(result, vec![11, 21, 12, 22]);
```

##### ResultF — 独立した失敗しうる参照の結合

``` rust
use karpal_std::prelude::*;

let result = ado_! { ResultF<String>;
    host = Ok::<&str, String>("localhost");
    port = Ok::<u16, String>(3000);
    yield format!("{}:{}", host, port)
};
assert_eq!(result, Ok("localhost:3000".to_string()));
```


## `do_!` と `ado_!` の使い分け

| マクロ   | 必要なトレイト  | 束縛                                                | 使う場面                                                     |
|---------|-----------------|---------------------------------------------------------|--------------------------------------------------------------|
| `do_!`  | `Chain` (Monad) | 逐次 — 後の束縛が前のものに依存できる | ステップ間にデータ依存がある                                 |
| `ado_!` | `Applicative`   | 独立 — 束縛は互いに参照してはならない   | ステップが独立; 依存の不在を文書化 |

## なぜ `<-` ではなく `=` なのか?

Haskell や PureScript のような言語はモナド束縛に `<-` を使います。Karpal は代わりに `=` を使います。Rust edition 2024 が `<-` トークンを予約しており、マクロ内で使えないからです。`=` 構文は Rust の既存パターンに自然に統合され、予約トークンとの競合を避けます。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
