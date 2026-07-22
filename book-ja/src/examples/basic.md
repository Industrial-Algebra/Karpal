# 基本的な使い方

## Functor と Monad

```rust
use karpal_core::{Functor, Monad, hkt::OptionF};

// fmap: 関数をコンテナに持ち上げる
let doubled: Option<i32> = OptionF::fmap(Some(21), |x| x * 2);
assert_eq!(doubled, Some(42));

// chain (bind/flatMap): 演算を逐次化
let result: Option<i32> = OptionF::chain(Some(5), |x| Some(x + 1));
assert_eq!(result, Some(6));
```

## Applicative と ado_!

```rust
use karpal_core::{ado_, Applicative, hkt::OptionF};

fn load_config(env: &[(&str, &str)]) -> Option<String> {
    let find = |key: &str| env.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string());
    ado_! { OptionF;
        host = find("DB_HOST");
        port = find("DB_PORT");
        yield format!("postgres://{}:{}", host, port)
    }
}
```

## オプティクス

```rust
use karpal_optics::{Lens, Prism};

// Lens: フィールドに焦点を当てる
struct User { name: String, age: u32 }
let name_lens = Lens::new(|u: &User| u.name.clone(), |u, n| User { name: n, ..u });
let user = User { name: "Alice".into(), age: 30 };
let renamed = name_lens.set(user, "Bob".into());
assert_eq!(renamed.name, "Bob");

// Prism: バリアントに焦点を当てる
let some_prism = Prism::new(
    |x: Option<i32>| x.map(Ok).unwrap_or(Err(())),
    |v: i32| Some(v),
);
```

## 図式 DSL

```rust
use karpal_diagram::Diagram;

let circuit = Diagram::box_("f", 1, 1)
    .parallel(Diagram::box_("g", 1, 1))
    .then(Diagram::swap(1, 1));

println!("{}", circuit.render_text());
```

## 検証

```rust
use karpal_diagram::coherence::coherence_certificates;

let certs = coherence_certificates();
assert_eq!(certs.len(), 3); // pentagon、triangle、hexagon
```
