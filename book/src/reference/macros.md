# Macros

Monadic and applicative notation macros.

Karpal provides two macros that flatten nested monadic and applicative computations into a readable, top-to-bottom sequence of bindings. Both macros use `=` for binding (not `<-`, which is reserved in Rust edition 2024).


### `do_!`

Monadic do-notation. Desugars sequential bindings into nested `Chain::chain` calls.


#### Syntax

``` rust
do_! { F;
    x = monadic_expr_1;
    y = monadic_expr_2;   // can reference x
    // ... more bindings ...
    final_monadic_expr     // bare expression, no binding
}
```

- The first token `F` is the type constructor (`OptionF`, `VecF`, `ResultF<E>`, etc.), followed by a semicolon.
- Each binding uses `=`. Later bindings can reference names bound earlier -- the steps are sequential.
- The final line is a bare expression of type `F::Of<T>`. It is the value returned by the whole `do_!` block.
- If any step produces a short-circuiting value (`None`, `Err(_)`), the entire block short-circuits immediately.

#### Expansion

Each `x = expr;` binding desugars into a `Chain::chain` call. The macro expands recursively:

``` rust
// This:
do_! { F;
    x = expr_a;
    y = expr_b;
    expr_c
}

// Expands to:
<F as Chain>::chain(expr_a, |x| {
    <F as Chain>::chain(expr_b, |y| {
        expr_c
    })
})
```

A single bare expression (no bindings) is returned as-is:

``` rust
do_! { F; some_expr }
// Expands to:
some_expr
```

#### Requirements

The type constructor `F` must implement `Chain` (and therefore `Apply` and `Functor`). In practice, any type that implements `Monad` satisfies this requirement, since `Monad` is a blanket trait over `Applicative + Chain`.

#### Examples

##### OptionF -- sequential computation with short-circuiting

``` rust
use karpal_std::prelude::*;

let result = do_! { OptionF;
    x = Some(1);
    y = Some(x + 1);       // y depends on x
    OptionF::pure(x + y)   // final expression wraps in Some
};
assert_eq!(result, Some(3));
```

##### OptionF -- short-circuiting on None

``` rust
use karpal_std::prelude::*;

let result: Option<i32> = do_! { OptionF;
    x = Some(1);
    _y = None::<i32>;     // short-circuits here
    OptionF::pure(x)       // never reached
};
assert_eq!(result, None);
```

##### OptionF -- single expression (no bindings)

``` rust
use karpal_std::prelude::*;

let result = do_! { OptionF;
    Some(42)
};
assert_eq!(result, Some(42));
```

##### ResultF -- chaining fallible operations

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

##### VecF -- list comprehension (cartesian product)

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

Applicative do-notation. Collects independent bindings and combines them with `Apply::ap` and `Functor::fmap`.


#### Syntax

``` rust
ado_! { F;
    x = applicative_expr_1;
    y = applicative_expr_2;
    // ... up to 4 bindings ...
    yield combining_expression
}
```

- Same first-token convention as `do_!`: the type constructor, then a semicolon.
- Each binding uses `=`. Bindings are **independent** and must not reference each other.
- The `yield` keyword introduces the combining expression. This expression is a pure function of the bound names -- it is automatically lifted into the applicative context.
- Supports 1 to 4 bindings.
- If any binding evaluates to a short-circuiting value (`None`, `Err(_)`), the whole block short-circuits.

#### Expansion

The expansion depends on the number of bindings. With one binding, the macro uses `Functor::fmap`. With two or more, it builds a curried closure and applies it with `Apply::ap`:

##### 1 binding

``` rust
// This:
ado_! { F; x = expr; yield body }

// Expands to:
<F as Functor>::fmap(expr, |x| body)
```

##### 2 bindings

``` rust
// This:
ado_! { F; x = e1; y = e2; yield body }

// Expands to:
<F as Apply>::ap(
    <F as Functor>::fmap(e1, |x| move |y| body),
    e2,
)
```

##### 3 bindings

``` rust
// This:
ado_! { F; x = e1; y = e2; z = e3; yield body }

// Expands to:
<F as Apply>::ap(
    <F as Apply>::ap(
        <F as Functor>::fmap(e1, |x| move |y| move |z| body),
        e2,
    ),
    e3,
)
```

##### 4 bindings

``` rust
// This:
ado_! { F; a = e1; b = e2; c = e3; d = e4; yield body }

// Expands to:
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

#### Requirements

The type constructor `F` must implement `Applicative` (and therefore `Apply` and `Functor`). Unlike `do_!`, it does **not** require `Chain` -- applicative computations are strictly less powerful than monadic ones, which is the point: they express the absence of sequential dependencies.

#### Examples

##### OptionF -- single binding (fmap)

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(5);
    yield x * 2
};
assert_eq!(result, Some(10));
```

##### OptionF -- combining two independent values

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(1);
    y = Some(2);
    yield x + y
};
assert_eq!(result, Some(3));
```

##### OptionF -- short-circuiting on None

``` rust
use karpal_std::prelude::*;

let result = ado_! { OptionF;
    x = Some(1);
    y = None::<i32>;
    yield x + y
};
assert_eq!(result, None);
```

##### OptionF -- combining three values

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

##### OptionF -- combining four values

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

##### VecF -- cartesian product with applicative

``` rust
use karpal_std::prelude::*;

let result = ado_! { VecF;
    x = vec![1, 2];
    y = vec![10, 20];
    yield x + y
};
assert_eq!(result, vec![11, 21, 12, 22]);
```

##### ResultF -- combining independent fallible lookups

``` rust
use karpal_std::prelude::*;

let result = ado_! { ResultF<String>;
    host = Ok::<&str, String>("localhost");
    port = Ok::<u16, String>(3000);
    yield format!("{}:{}", host, port)
};
assert_eq!(result, Ok("localhost:3000".to_string()));
```


## Choosing Between `do_!` and `ado_!`

| Macro   | Trait required  | Bindings                                                | Use when                                                     |
|---------|-----------------|---------------------------------------------------------|--------------------------------------------------------------|
| `do_!`  | `Chain` (Monad) | Sequential -- later bindings can depend on earlier ones | Steps have data dependencies                                 |
| `ado_!` | `Applicative`   | Independent -- bindings must not reference each other   | Steps are independent; documents the absence of dependencies |

## Why `=` Instead of `<-`?

Languages like Haskell and PureScript use `<-` for monadic bindings. Karpal uses `=` instead because Rust edition 2024 reserves the `<-` token, making it unavailable inside macros. The `=` syntax integrates naturally with Rust's existing patterns and avoids any conflict with reserved tokens.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


