# 高次圏

2-圏、豊饒圏、バイ圏、FFunctor/FMonad — `karpal-higher` (フェーズ 15)。


## TwoCategory

厳密 2-圏は対象、対象間の 1-射、平行な 1-射間の 2-射を持ちます:

``` rust
use karpal_higher::{TwoCategory, Cat};

// Cat: 対象 = 型、1-射 = Box、2-射 = ()
let id = Cat::id1::();
assert_eq!(id(42), 42);

let f: Box i32> = Box::new(|x| x + 1);
let g: Box i32> = Box::new(|x| x * 2);
let gf = Cat::compose1(f, g);
assert_eq!(gf(5), 12);
```


## Bicategory

バイ圏は結合律と単位律を同型にまで弱め、結合子と左右の単位子を持ちます:

``` rust
use karpal_higher::{Bicategory, Cat};

// 結合子: (f ∘ g) ∘ h ≅ f ∘ (g ∘ h)
let _alpha = Cat::associator::();

// 左単位子: id ∘ f ≅ f
let _lambda = Cat::left_unitor::();

// 右単位子: f ∘ id ≅ f
let _rho = Cat::right_unitor::();
```


## EnrichedCategory

ホム対象が代数的構造を持つモノイダル基底 V 上の豊饒圏:

``` rust
use karpal_higher::{EnrichedCategory, SetCategory, SetEnrichment};

// Set 上の豊饒化: 通常の圏
let id = SetCategory::id::();
assert_eq!(id(42), 42);

let f: Box i32> = Box::new(|x| x + 1);
let g: Box i32> = Box::new(|x| x * 2);
let gf = SetCategory::compose(f, g);
assert_eq!(gf(5), 12);
```


## FFunctor / FMonad

2-圏間の関手と、自己関手 2-圏におけるモナド:

``` rust
use karpal_higher::{FFunctor, IdentityFFunctor, TwoCategory};

// 恒等 FFunctor は 1-射と 2-射を保存する
let m = IdentityFFunctor::<Cat>::map_morphism::<i32, i32>(Cat::id1());
```


## コヒーレンス証拠

`karpal-proof::Justifies` によるバイ圏コヒーレンス法則の型レベル証拠:

| 証拠                            | 法則                                           |
|--------------------------------|-----------------------------------------------|
| `InterchangeIdentity`          | `(α ∘ᵥ β) ∘ₕ (γ ∘ᵥ δ) = (α ∘ₕ γ) ∘ᵥ (β ∘ₕ δ)` |
| `BicategoryPentagonIdentity`   | 結合子五角形コヒーレンス                 |
| `BicategoryTriangleIdentity`   | 単位子三角形コヒーレンス                     |

``` rust
use karpal_higher::verify_interchange;
let _proof = verify_interchange();
```


## 検証統合

コヒーレンス証明書は `karpal-verify` に接続します:

``` rust
use karpal_higher::higher_coherence_certificates;

let certs = higher_coherence_certificates();
assert_eq!(certs.len(), 3); // interchange、pentagon、triangle
for cert in &certs {
    assert_eq!(cert.backend, "karpal-higher-coherence");
}
```
