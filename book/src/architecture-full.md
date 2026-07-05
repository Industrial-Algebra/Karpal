[Home](index.md) \> Architecture

# Architecture

This page explains the core design decisions behind Karpal: how it encodes higher-kinded types in Rust, the full trait hierarchy, and the Static Land pattern that makes it all work within Rust's type system.

## HKT Encoding

### The problem

Rust has no native higher-kinded types. You cannot write a trait that is generic over a *type constructor* like `Option` or `Vec` — only over concrete types like `Option<i32>`. This means there is no built-in way to express "for any container `F`, give me an `fmap` that works on `F<A>`."

### The GAT solution

Karpal encodes type constructors as **marker types** that implement a trait with a Generic Associated Type (GAT). The `HKT` trait acts as a type-level function: given a type `T`, it produces `Self::Of<T>`.

``` rust
/// Higher-Kinded Type encoding via GATs.
///
/// A type implementing `HKT` acts as a type-level function:
/// given a type `T`, it produces `Self::Of<T>`.
pub trait HKT {
    type Of<T>;
}
```

Each standard container gets a zero-sized marker type that maps `Of<T>` to the real type:

``` rust
/// Type constructor for `Option<T>`.
pub struct OptionF;

impl HKT for OptionF {
    type Of<T> = Option<T>;
}

/// Type constructor for `Result<T, E>` (fixed error type `E`).
pub struct ResultF<E> {
    _marker: PhantomData<E>,
}

impl<E> HKT for ResultF<E> {
    type Of<T> = Result<T, E>;
}

/// Type constructor for `Vec<T>` (alloc-gated).
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct VecF;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKT for VecF {
    type Of<T> = Vec<T>;
}
```

### Two-parameter HKT

For types with two type parameters — bifunctors and profunctors — Karpal provides `HKT2`:

``` rust
/// Two-parameter type constructor (HKT for bifunctors / profunctors).
pub trait HKT2 {
    type P<A, B>;
}

/// Result as a bifunctor (both parameters vary).
pub struct ResultBF;

impl HKT2 for ResultBF {
    type P<A, B> = Result<B, A>;
}

/// Tuple as a bifunctor.
pub struct TupleF;

impl HKT2 for TupleF {
    type P<A, B> = (A, B);
}
```

### Tradeoffs

| Property     | Detail                                                                                                                 |
|--------------|------------------------------------------------------------------------------------------------------------------------|
| Runtime cost | Zero. Marker types are ZSTs; all dispatch is monomorphized at compile time.                                            |
| Dependencies | None. Pure Rust with no external crates for the encoding itself.                                                       |
| Toolchain    | Requires nightly Rust (edition 2024). GATs are stable since 1.65, but Karpal also uses `use<>` precise-capture syntax. |
| Ergonomics   | Callers write `OptionF::fmap(...)` instead of `value.fmap(...)`. This is the Static Land style (see below).            |

## Trait Hierarchy

The diagram below shows the full trait hierarchy implemented in `karpal-core`, `karpal-profunctor`, and `karpal-arrow`. Arrows point from supertrait to subtrait. Dashed borders indicate blanket implementations (no manual impl needed).

![](data:image/svg+xml;base64,PHN2ZyB2aWV3Ym94PSIwIDAgODAwIDc4MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiBzdHlsZT0ibWF4LXdpZHRoOiA4MDBweDsgd2lkdGg6IDEwMCU7IGhlaWdodDogYXV0bzsgZm9udC1mYW1pbHk6ICYjMzk7SW50ZXImIzM5Oywgc2Fucy1zZXJpZjsiPgogICAgICA8ZGVmcz4KICAgICAgICA8bWFya2VyIGlkPSJhcnJvdyIgdmlld2JveD0iMCAwIDEwIDEwIiByZWZ4PSIxMCIgcmVmeT0iNSIgbWFya2Vyd2lkdGg9IjgiIG1hcmtlcmhlaWdodD0iOCIgb3JpZW50PSJhdXRvLXN0YXJ0LXJldmVyc2UiPgogICAgICAgICAgPHBhdGggZD0iTSAwIDAgTCAxMCA1IEwgMCAxMCB6IiBmaWxsPSJ2YXIoLS10ZXh0LW11dGVkLCAjOGI5NDllKSIgLz4KICAgICAgICA8L21hcmtlcj4KICAgICAgICA8c3R5bGU+CiAgICAgICAgICAubm9kZSB7IGZpbGw6IHZhcigtLWJnLWNhcmQsICMxNjFiMjIpOyBzdHJva2U6IHZhcigtLWJvcmRlciwgIzMwMzYzZCk7IHN0cm9rZS13aWR0aDogMS41OyByeDogNjsgcnk6IDY7IH0KICAgICAgICAgIC5ub2RlLWJsYW5rZXQgeyBmaWxsOiB2YXIoLS1iZy1jYXJkLCAjMTYxYjIyKTsgc3Ryb2tlOiB2YXIoLS1ib3JkZXIsICMzMDM2M2QpOyBzdHJva2Utd2lkdGg6IDEuNTsgc3Ryb2tlLWRhc2hhcnJheTogNSAzOyByeDogNjsgcnk6IDY7IH0KICAgICAgICAgIC5ub2RlLWxhYmVsIHsgZmlsbDogdmFyKC0tdGV4dC1wcmltYXJ5LCAjZTZlZGYzKTsgZm9udC1zaXplOiAxMXB4OyBmb250LXdlaWdodDogNjAwOyB0ZXh0LWFuY2hvcjogbWlkZGxlOyBkb21pbmFudC1iYXNlbGluZTogY2VudHJhbDsgfQogICAgICAgICAgLmVkZ2UgeyBzdHJva2U6IHZhcigtLXRleHQtbXV0ZWQsICM4Yjk0OWUpOyBzdHJva2Utd2lkdGg6IDEuMjsgZmlsbDogbm9uZTsgbWFya2VyLWVuZDogdXJsKCNhcnJvdyk7IH0KICAgICAgICAgIC5zZWN0aW9uLWxhYmVsIHsgZmlsbDogdmFyKC0tdGV4dC1tdXRlZCwgIzhiOTQ5ZSk7IGZvbnQtc2l6ZTogOXB4OyBmb250LXdlaWdodDogNTAwOyB0ZXh0LXRyYW5zZm9ybTogdXBwZXJjYXNlOyBsZXR0ZXItc3BhY2luZzogMC4wOGVtOyB9CiAgICAgICAgPC9zdHlsZT4KICAgICAgPC9kZWZzPgoKICAgICAgPCEtLSBTZWN0aW9uIGxhYmVscyAtLT4KICAgICAgPHRleHQgeD0iMTYiIHk9IjE4IiBjbGFzcz0ic2VjdGlvbi1sYWJlbCI+Q292YXJpYW50IChGdW5jdG9yIGZhbWlseSk8L3RleHQ+CiAgICAgIDx0ZXh0IHg9IjE2IiB5PSIyOTgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Db21vbmFkIGZhbWlseTwvdGV4dD4KICAgICAgPHRleHQgeD0iNDMwIiB5PSIyOTgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Db250cmF2YXJpYW50IGZhbWlseTwvdGV4dD4KICAgICAgPHRleHQgeD0iMTYiIHk9IjQ0OCIgY2xhc3M9InNlY3Rpb24tbGFiZWwiPkFsZ2VicmFpYzwvdGV4dD4KICAgICAgPHRleHQgeD0iMjUwIiB5PSI0NDgiIGNsYXNzPSJzZWN0aW9uLWxhYmVsIj5Gb2xkYWJsZSAvIFRyYXZlcnNhYmxlPC90ZXh0PgogICAgICA8dGV4dCB4PSIxNiIgeT0iNTQ4IiBjbGFzcz0ic2VjdGlvbi1sYWJlbCI+UHJvZnVuY3RvciBmYW1pbHkgKEhLVDIpPC90ZXh0PgoKICAgICAgPCEtLSA9PT0gQ292YXJpYW50IHJvdyAxOiBGdW5jdG9yIC0+IEFwcGx5IC0+IEFwcGxpY2F0aXZlIC0+IEFsdGVybmF0aXZlIChibGFua2V0KSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iMzAiIHdpZHRoPSI3MiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjUyIiB5PSI0NSI+RnVuY3RvcjwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxMjgiIHk9IjMwIiB3aWR0aD0iNjAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIxNTgiIHk9IjQ1Ij5BcHBseTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyMjgiIHk9IjMwIiB3aWR0aD0iOTAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIyNzMiIHk9IjQ1Ij5BcHBsaWNhdGl2ZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlLWJsYW5rZXQiIHg9IjM1OCIgeT0iMzAiIHdpZHRoPSI5MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjQwMyIgeT0iNDUiPkFsdGVybmF0aXZlPC90ZXh0PgoKICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI4OCIgeTE9IjQ1IiB4Mj0iMTI2IiB5Mj0iNDUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxODgiIHkxPSI0NSIgeDI9IjIyNiIgeTI9IjQ1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMzE4IiB5MT0iNDUiIHgyPSIzNTYiIHkyPSI0NSI+PC9saW5lPgoKICAgICAgPCEtLSA9PT0gQ292YXJpYW50IHJvdyAyOiBDaGFpbiwgU2VsZWN0aXZlID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjEyOCIgeT0iODAiIHdpZHRoPSI2MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE1OCIgeT0iOTUiPkNoYWluPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjIyOCIgeT0iODAiIHdpZHRoPSI5MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI3MyIgeT0iOTUiPlNlbGVjdGl2ZTwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMTU4IiB5MT0iNjAiIHgyPSIxNTgiIHkyPSI3OCI+PC9saW5lPgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjI3MyIgeTE9IjYwIiB4Mj0iMjczIiB5Mj0iNzgiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IENvdmFyaWFudCByb3cgMzogTW9uYWQgKGJsYW5rZXQpID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUtYmxhbmtldCIgeD0iMTY4IiB5PSIxMzAiIHdpZHRoPSI3MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjIwMyIgeT0iMTQ1Ij5Nb25hZDwvdGV4dD4KCiAgICAgIDwhLS0gTGluZXMgZnJvbSBBcHBsaWNhdGl2ZSBhbmQgQ2hhaW4gdG8gTW9uYWQgLS0+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjczIiB5MT0iNjAiIHgyPSIyMzAiIHkyPSIxMjgiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxNTgiIHkxPSIxMTAiIHgyPSIxODUiIHkyPSIxMjgiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEFsdCByb3c6IEZ1bmN0b3IgLT4gQWx0IC0+IFBsdXMgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTI4IiB5PSIxODUiIHdpZHRoPSI0OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE1MiIgeT0iMjAwIj5BbHQ8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMjE2IiB5PSIxODUiIHdpZHRoPSI1MiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI0MiIgeT0iMjAwIj5QbHVzPC90ZXh0PgoKICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI2MiIgeTE9IjYwIiB4Mj0iMTQwIiB5Mj0iMTgzIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMTc2IiB5MT0iMjAwIiB4Mj0iMjE0IiB5Mj0iMjAwIj48L2xpbmU+CgogICAgICA8IS0tIFBsdXMgLT4gQWx0ZXJuYXRpdmUgLS0+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMjY4IiB5MT0iMjAwIiB4Mj0iNDAzIiB5Mj0iNjIiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEZ1bmN0b3JGaWx0ZXIgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNTAwIiB5PSIzMCIgd2lkdGg9IjEwNSIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjU1MiIgeT0iNDUiPkZ1bmN0b3JGaWx0ZXI8L3RleHQ+CgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9Ijg4IiB5MT0iMzgiIHgyPSI0OTgiIHkyPSIzOCI+PC9saW5lPgoKICAgICAgPCEtLSA9PT0gSW52YXJpYW50ID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjY1MCIgeT0iMzAiIHdpZHRoPSI3NiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjY4OCIgeT0iNDUiPkludmFyaWFudDwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iODgiIHkxPSI1MiIgeDI9IjY0OCIgeTI9IjUyIj48L2xpbmU+CgogICAgICA8IS0tID09PSBCaWZ1bmN0b3IgKEhLVDIpID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjU2MCIgeT0iODAiIHdpZHRoPSI4MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjYwMCIgeT0iOTUiPkJpZnVuY3RvcjwvdGV4dD4KCiAgICAgIDwhLS0gTmF0dXJhbFRyYW5zZm9ybWF0aW9uIHN0YW5kYWxvbmUgLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NjAiIHk9IjEzMCIgd2lkdGg9IjE0NSIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjYzMiIgeT0iMTQ1Ij5OYXR1cmFsVHJhbnNmb3JtYXRpb248L3RleHQ+CgogICAgICA8IS0tID09PSBDb21vbmFkIGZhbWlseSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iMzEwIiB3aWR0aD0iNjgiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI1MCIgeT0iMzI1Ij5FeHRlbmQ8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTI0IiB5PSIzMTAiIHdpZHRoPSI3OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE2MyIgeT0iMzI1Ij5Db21vbmFkPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjI0MiIgeT0iMzEwIiB3aWR0aD0iOTAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIyODciIHk9IjMyNSI+Q29tb25hZEVudjwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyNDIiIHk9IjM1NSIgd2lkdGg9Ijk4IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iMjkxIiB5PSIzNzAiPkNvbW9uYWRTdG9yZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIyNDIiIHk9IjQwMCIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjI5MiIgeT0iNDE1Ij5Db21vbmFkVHJhY2VkPC90ZXh0PgoKICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI2MiIgeTE9IjYwIiB4Mj0iNDAiIHkyPSIzMDgiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI4NCIgeTE9IjMyNSIgeDI9IjEyMiIgeTI9IjMyNSI+PC9saW5lPgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjIwMiIgeTE9IjMyNSIgeDI9IjI0MCIgeTI9IjMyNSI+PC9saW5lPgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjIwMiIgeTE9IjMzMCIgeDI9IjI0MCIgeTI9IjM2NSI+PC9saW5lPgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9IjIwMiIgeTE9IjMzNSIgeDI9IjI0MCIgeTI9IjQwNSI+PC9saW5lPgoKICAgICAgPCEtLSA9PT0gQ29udHJhdmFyaWFudCBmYW1pbHkgPT09IC0tPgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNDMwIiB5PSIzMTAiIHdpZHRoPSIxMDAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI0ODAiIHk9IjMyNSI+Q29udHJhdmFyaWFudDwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NzAiIHk9IjMxMCIgd2lkdGg9IjY0IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjAyIiB5PSIzMjUiPkRpdmlkZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2NzQiIHk9IjMxMCIgd2lkdGg9IjcyIiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzEwIiB5PSIzMjUiPkRpdmlzaWJsZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NzAiIHk9IjM1NSIgd2lkdGg9IjY0IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNjAyIiB5PSIzNzAiPkRlY2lkZTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2NzQiIHk9IjM1NSIgd2lkdGg9Ijc4IiBoZWlnaHQ9IjMwIiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzEzIiB5PSIzNzAiPkNvbmNsdWRlPC90ZXh0PgoKICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI1MzAiIHkxPSIzMjUiIHgyPSI1NjgiIHkyPSIzMjUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI2MzQiIHkxPSIzMjUiIHgyPSI2NzIiIHkyPSIzMjUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI1MzAiIHkxPSIzMzUiIHgyPSI1NjgiIHkyPSIzNjUiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSI2MzQiIHkxPSIzNzAiIHgyPSI2NzIiIHkyPSIzNzAiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEFsZ2VicmFpYyA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iNDYwIiB3aWR0aD0iODIiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI1NyIgeT0iNDc1Ij5TZW1pZ3JvdXA8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMTM4IiB5PSI0NjAiIHdpZHRoPSI2OCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjE3MiIgeT0iNDc1Ij5Nb25vaWQ8L3RleHQ+CgogICAgICA8bGluZSBjbGFzcz0iZWRnZSIgeDE9Ijk4IiB5MT0iNDc1IiB4Mj0iMTM2IiB5Mj0iNDc1Ij48L2xpbmU+CgogICAgICA8IS0tID09PSBGb2xkYWJsZSAvIFRyYXZlcnNhYmxlID09PSAtLT4KICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjI4MCIgeT0iNDYwIiB3aWR0aD0iNzIiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIzMTYiIHk9IjQ3NSI+Rm9sZGFibGU8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMzkyIiB5PSI0NjAiIHdpZHRoPSI5MCIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjQzNyIgeT0iNDc1Ij5UcmF2ZXJzYWJsZTwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iMzUyIiB5MT0iNDc1IiB4Mj0iMzkwIiB5Mj0iNDc1Ij48L2xpbmU+CgogICAgICA8IS0tID09PSBQcm9mdW5jdG9yIGZhbWlseSA9PT0gLS0+CiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSIxNiIgeT0iNTYwIiB3aWR0aD0iODYiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI1OSIgeT0iNTc1Ij5Qcm9mdW5jdG9yPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjE0MiIgeT0iNTYwIiB3aWR0aD0iNjQiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIxNzQiIHk9IjU3NSI+U3Ryb25nPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjE0MiIgeT0iNjAwIiB3aWR0aD0iNjQiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSIxNzQiIHk9IjYxNSI+Q2hvaWNlPC90ZXh0PgoKICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxMDIiIHkxPSI1NzAiIHgyPSIxNDAiIHkyPSI1NzAiPjwvbGluZT4KICAgICAgPGxpbmUgY2xhc3M9ImVkZ2UiIHgxPSIxMDIiIHkxPSI1ODAiIHgyPSIxNDAiIHkyPSI2MTAiPjwvbGluZT4KCiAgICAgIDwhLS0gPT09IEFycm93IGZhbWlseSAoSEtUMikgPT09IC0tPgogICAgICA8dGV4dCB4PSIzNTAiIHk9IjU0OCIgY2xhc3M9InNlY3Rpb24tbGFiZWwiPkFycm93IGZhbWlseSAoSEtUMik8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iMzUwIiB5PSI1NjAiIHdpZHRoPSIxMDAiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI0MDAiIHk9IjU3NSI+U2VtaWdyb3Vwb2lkPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjQ5MCIgeT0iNTYwIiB3aWR0aD0iNzIiIGhlaWdodD0iMzAiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI1MjYiIHk9IjU3NSI+Q2F0ZWdvcnk8L3RleHQ+CgogICAgICA8cmVjdCBjbGFzcz0ibm9kZSIgeD0iNjAyIiB5PSI1NjAiIHdpZHRoPSI1NiIgaGVpZ2h0PSIzMCIgLz4KICAgICAgPHRleHQgY2xhc3M9Im5vZGUtbGFiZWwiIHg9IjYzMCIgeT0iNTc1Ij5BcnJvdzwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI3MDAiIHk9IjU0MCIgd2lkdGg9IjkyIiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzQ2IiB5PSI1NTMiPkFycm93Q2hvaWNlPC90ZXh0PgoKICAgICAgPHJlY3QgY2xhc3M9Im5vZGUiIHg9IjcwMCIgeT0iNTcwIiB3aWR0aD0iODYiIGhlaWdodD0iMjYiIC8+CiAgICAgIDx0ZXh0IGNsYXNzPSJub2RlLWxhYmVsIiB4PSI3NDMiIHk9IjU4MyI+QXJyb3dBcHBseTwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI3MDAiIHk9IjYwMCIgd2lkdGg9Ijg2IiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzQzIiB5PSI2MTMiPkFycm93TG9vcDwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI1NDAiIHk9IjYxMCIgd2lkdGg9IjgyIiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNTgxIiB5PSI2MjMiPkFycm93WmVybzwvdGV4dD4KCiAgICAgIDxyZWN0IGNsYXNzPSJub2RlIiB4PSI2NjAiIHk9IjY0MCIgd2lkdGg9IjgyIiBoZWlnaHQ9IjI2IiAvPgogICAgICA8dGV4dCBjbGFzcz0ibm9kZS1sYWJlbCIgeD0iNzAxIiB5PSI2NTMiPkFycm93UGx1czwvdGV4dD4KCiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNDUwIiB5MT0iNTc1IiB4Mj0iNDg4IiB5Mj0iNTc1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNTYyIiB5MT0iNTc1IiB4Mj0iNjAwIiB5Mj0iNTc1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTY4IiB4Mj0iNjk4IiB5Mj0iNTU1Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTc1IiB4Mj0iNjk4IiB5Mj0iNTgwIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjU4IiB5MT0iNTgyIiB4Mj0iNjk4IiB5Mj0iNjEwIj48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjMwIiB5MT0iNTkwIiB4Mj0iNTcwIiB5Mj0iNjA4Ij48L2xpbmU+CiAgICAgIDxsaW5lIGNsYXNzPSJlZGdlIiB4MT0iNjIyIiB5MT0iNjM2IiB4Mj0iNjU4IiB5Mj0iNjQ4Ij48L2xpbmU+CiAgICA8L3N2Zz4=)

**Key:** Solid borders are standard traits. **Dashed borders** indicate blanket implementations — `Monad` is automatically derived for any type that implements both `Applicative` and `Chain`, and `Alternative` for `Applicative + Plus`.

## The Static Land Pattern

Karpal uses **associated functions on marker types**, not methods on values. Instead of calling `some_value.fmap(f)`, you write:

``` rust
// Karpal's Static Land style
let result = OptionF::fmap(Some(42), |x| x + 1);

// NOT the method-on-value style (not possible in Karpal)
// let result = Some(42).fmap(|x| x + 1);
```

### Why this approach?

Rust's **trait coherence rules** (the orphan rule) prevent you from implementing a foreign trait on a foreign type. Since `Option` is defined in `std` and `Functor` is defined in Karpal, you cannot write `impl Functor for Option<T>` directly.

The marker-type approach sidesteps this entirely. `OptionF` is *owned by Karpal*, so Karpal can freely implement any trait on it. The `HKT` GAT bridges the gap back to the actual container type via the `Of<T>` associated type.

### Comparison with other ecosystems

| Ecosystem          | Approach                                                        | Tradeoff                                                     |
|--------------------|-----------------------------------------------------------------|--------------------------------------------------------------|
| Haskell            | Native typeclasses with HKT support                             | Ideal ergonomics; not available in Rust                      |
| Scala              | Implicits / given instances with `Kind` projections             | Powerful but complex; relies on JVM runtime                  |
| fp-ts (TypeScript) | Static Land — functions in module namespaces (`O.map`, `A.map`) | Closest analogue to Karpal's design; same ergonomic tradeoff |
| **Karpal (Rust)**  | Static Land — associated functions on marker types              | Zero-cost, type-safe, but verbose call syntax                |

## Design Decisions

### `no_std` first

`karpal-core`, `karpal-profunctor`, and `karpal-arrow` compile without `std`. Types that require heap allocation — `VecF`, `NonEmptyVecF`, `PredicateF`, `StoreF`, `TracedF` — are gated behind the `alloc` or `std` feature flags. This makes Karpal usable in embedded and `no_std` environments.

### Nightly edition 2024

The toolchain is pinned to nightly via `rust-toolchain.toml`. While GATs themselves stabilized in Rust 1.65, Karpal also relies on `use<>` precise-capture syntax (edition 2024) and the `alloc` feature gate for `no_std` builds. The nightly pin ensures all contributors use the same compiler.

### `fn` pointers in optics

The `Lens` struct stores plain function pointers rather than closures:

``` rust
pub struct Lens<S, T, A, B> {
    getter: fn(&S) -> A,
    setter: fn(S, B) -> T,
}
```

This keeps `Lens` `Copy`-able and avoids lifetime complications. When lenses are composed via `Lens::then()`, the result is a `ComposedLens` that uses `Box<dyn Fn>` closures instead, since closure composition cannot produce `fn` pointers.

### `'static` bounds on `Box<dyn Fn>`

Types whose inner representation is a boxed closure — `PredicateF`, `FnP`, `FnA`, `KleisliF`, `CokleisliF`, `StoreF`, `TracedF` — require `'static` bounds. This is an inherent limitation of `Box<dyn Fn>` in Rust. As a consequence, `StoreF` and `TracedF` cannot implement the generic `Functor` trait (whose signature does not carry a `'static` bound); they provide their own `fmap` through the `Extend`/`Comonad` implementation instead.

### Blanket implementations

Where the theory permits, Karpal uses blanket impls to eliminate boilerplate:

``` rust
/// Monad: Applicative + Chain with no extra methods (blanket impl).
pub trait Monad: Applicative + Chain {}

impl<F: Applicative + Chain> Monad for F {}
```

Any marker type that implements both `Applicative` and `Chain` is automatically a `Monad`. The same pattern applies to `Alternative` (= `Applicative + Plus`). Implementors only need to provide the primitive operations; the composed abstractions come for free.

### Property-based law testing

Every algebraic trait in Karpal has [proptest](https://crates.io/crates/proptest)-based law tests that verify the required algebraic identities hold. For example, the Functor laws:

``` rust
proptest! {
    #[test]
    fn option_identity(x in any::<Option<i32>>()) {
        // Identity law: fmap(id, fa) == fa
        let result = OptionF::fmap(x.clone(), |a| a);
        prop_assert_eq!(result, x);
    }

    #[test]
    fn option_composition(x in any::<Option<i32>>()) {
        // Composition law: fmap(g . f, fa) == fmap(g, fmap(f, fa))
        let f = |a: i32| a.wrapping_add(1);
        let g = |a: i32| a.wrapping_mul(2);
        let left = OptionF::fmap(x.clone(), |a| g(f(a)));
        let right = OptionF::fmap(OptionF::fmap(x, f), g);
        prop_assert_eq!(left, right);
    }
}
```

This approach catches subtle bugs that unit tests miss — such as associativity violations in `Semigroup` implementations or distributivity failures in `Alternative`.

## Proof and Verification Layering

`karpal-proof` and `karpal-verify` extend this architecture above the trait hierarchy with two distinct reasoning layers.

| Layer                 | Crate           | Architectural role                                                                                    |
|-----------------------|-----------------|-------------------------------------------------------------------------------------------------------|
| Internal evidence     | `karpal-proof`  | Encodes law witnesses, rewrites, and refinement types directly in Rust APIs                           |
| External verification | `karpal-verify` | Bridges Karpal obligations to SMT and Lean, then reports results and imported certificates explicitly |

### Why two layers?

`karpal-proof` and `karpal-verify` intentionally solve different problems. The first lets Rust code carry structured evidence after local checks, trait-derived witnesses, or audited assumptions. The second lets Karpal talk to external provers without pretending that a solver result is the same thing as compiler-checked evidence.

### `karpal-verify` pipeline

The external verification layer is organized as a pipeline:

1.  Model a goal as an `Obligation` or `ObligationBundle`.
2.  Export the bundle to SMT-LIB2 scripts or a Lean module.
3.  Attach structured Lean metadata such as theorem identities, declaration spans, imports, aliases, and package/project scaffold data.
4.  Write artifacts and build `InvocationPlan` values.
5.  Execute those plans through a `VerifierRunner`, including project-aware `lake env lean` or `lake build` flows when desired.
6.  Parse solver or Lean output, including Lean diagnostics and theorem-aware failure mapping.
7.  Interpret success through an explicit `VerificationPolicy` per backend.
8.  Aggregate results into a `VerificationReport`, Lean manifest, and diagnostics sidecar, all suitable for CI serialization.
9.  Import successful evidence as `Certified<B, P, T>`, not directly as `Proven<P, T>`.

### Trust boundary

This separation is deliberate. External evidence crosses a visible boundary through `Certified<...>` and only becomes `Proven<...>` via an explicit `unsafe` conversion. That keeps imported trust searchable in code review and avoids conflating theorem prover output with Rust-native guarantees.

For API details, see the [Proof & Verification reference](reference/proof-verification.md). For CI-specific execution/reporting guidance, see [Verification CI Workflow](reference/verification-ci.md). For serialized artifact compatibility, see [Verification Schemas](reference/verification-schemas.md). For a walkthrough example, see [Verification Workflow](examples/verification-workflow.md). For a domain-boundary example that combines `karpal-proof` and `karpal-verify`, see [Verified Domain API](examples/verified-domain-api.md). For the design note focused on imported trust, see [Trust Model](dev/phase-12-trust-model.md).


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


