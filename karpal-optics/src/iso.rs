use crate::fold::Fold;
use crate::getter::Getter;
use crate::optic::Optic;
use crate::review::Review;
use karpal_profunctor::Profunctor;

/// An isomorphism witnesses that `S` and `A` are "the same" structure.
///
/// Only requires `Profunctor` — the weakest constraint of any optic.
///
/// `S` — source, `T` — modified source, `A` — focus, `B` — replacement.
pub struct Iso<S, T, A, B> {
    forward: fn(&S) -> A,
    backward: fn(B) -> T,
}

/// A simple (monomorphic) iso where `S == T` and `A == B`.
pub type SimpleIso<S, A> = Iso<S, S, A, A>;

impl<S, T, A, B> Optic for Iso<S, T, A, B> {}

impl<S, T, A, B> Iso<S, T, A, B> {
    pub fn new(forward: fn(&S) -> A, backward: fn(B) -> T) -> Self {
        Self { forward, backward }
    }

    /// Extract the focus from a source.
    pub fn get(&self, s: &S) -> A {
        (self.forward)(s)
    }

    /// Construct a source from a focus value (the reverse direction).
    pub fn review(&self, b: B) -> T {
        (self.backward)(b)
    }

    /// Set the focus, discarding the old value.
    pub fn set(&self, _s: S, b: B) -> T {
        (self.backward)(b)
    }

    /// Profunctor encoding: transform a `P<A, B>` into a `P<S, T>`.
    ///
    /// Only requires `Profunctor` — no `Strong` or `Choice` needed.
    pub fn transform<P: Profunctor>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let fwd = self.forward;
        let bwd = self.backward;
        P::dimap(move |s: S| fwd(&s), bwd, pab)
    }

    /// Convert to a `Getter`.
    pub fn to_getter(&self) -> Getter<S, A> {
        Getter::new(self.forward)
    }

    /// Convert to a `Review`.
    pub fn to_review(&self) -> Review<T, B> {
        Review::new(self.backward)
    }

    /// Convert to a `Fold` (single-element).
    pub fn to_fold(&self) -> Fold<S, A>
    where
        S: 'static,
        A: 'static,
    {
        let fwd = self.forward;
        Fold::new(move |s| vec![fwd(s)])
    }
}

impl<S: Clone, T, A, B> Iso<S, T, A, B> {
    /// Modify the focus.
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T {
        (self.backward)(f((self.forward)(&s)))
    }

    /// Convert to a `ComposedLens` (uses boxed closures since iso's backward
    /// doesn't match lens's `fn(S, B) -> T` signature).
    pub fn to_lens(&self) -> crate::lens::ComposedLens<S, T, A, B>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let fwd = self.forward;
        let bwd = self.backward;
        crate::lens::ComposedLens::from_fns(Box::new(fwd), Box::new(move |_s, b| bwd(b)))
    }

    /// Convert to a `Setter`.
    pub fn to_setter(&self) -> crate::setter::Setter<S, T, A, B>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let fwd = self.forward;
        let bwd = self.backward;
        crate::setter::Setter::new(move |s: S, f: &dyn Fn(A) -> B| bwd(f(fwd(&s))))
    }

    /// Convert to a `Traversal` (single-element).
    pub fn to_traversal(&self) -> crate::traversal::Traversal<S, T, A, B>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let fwd = self.forward;
        let bwd = self.backward;
        crate::traversal::Traversal::new(move |s| vec![fwd(s)], move |s, f| bwd(f(fwd(&s))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_profunctor::FnP;
    use proptest::prelude::*;

    fn celsius_fahrenheit_iso() -> SimpleIso<f64, f64> {
        Iso::new(
            |c: &f64| c * 9.0 / 5.0 + 32.0,
            |f: f64| (f - 32.0) * 5.0 / 9.0,
        )
    }

    fn string_bytes_iso() -> SimpleIso<String, Vec<u8>> {
        Iso::new(
            |s: &String| s.clone().into_bytes(),
            |b: Vec<u8>| String::from_utf8(b).unwrap(),
        )
    }

    #[test]
    fn iso_get() {
        let iso = celsius_fahrenheit_iso();
        let result = iso.get(&100.0);
        assert!((result - 212.0).abs() < 1e-10);
    }

    #[test]
    fn iso_review() {
        let iso = celsius_fahrenheit_iso();
        let result = iso.review(212.0);
        assert!((result - 100.0).abs() < 1e-10);
    }

    #[test]
    fn iso_over() {
        let iso = celsius_fahrenheit_iso();
        // Convert 0°C to F, add 10 to F, convert back
        let result = iso.over(0.0, |f| f + 18.0);
        assert!((result - 10.0).abs() < 1e-10);
    }

    #[test]
    fn iso_set() {
        let iso = celsius_fahrenheit_iso();
        let result = iso.set(999.0, 32.0); // set F to 32 → 0°C
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn iso_transform_fnp() {
        let iso = string_bytes_iso();
        let upper: Box<dyn Fn(Vec<u8>) -> Vec<u8>> =
            Box::new(|bytes| bytes.into_iter().map(|b| b.to_ascii_uppercase()).collect());
        let f = iso.transform::<FnP>(upper);
        assert_eq!(f("hello".to_string()), "HELLO");
    }

    #[test]
    fn iso_to_lens() {
        let iso = string_bytes_iso();
        let lens = iso.to_lens();
        assert_eq!(lens.get(&"hi".to_string()), vec![b'h', b'i']);
        assert_eq!(lens.set("x".to_string(), vec![b'a', b'b']), "ab");
    }

    #[test]
    fn iso_to_getter() {
        let iso = string_bytes_iso();
        let getter = iso.to_getter();
        assert_eq!(getter.get(&"hi".to_string()), vec![b'h', b'i']);
    }

    #[test]
    fn iso_to_review() {
        let iso = string_bytes_iso();
        let review = iso.to_review();
        assert_eq!(review.review(vec![b'a', b'b']), "ab");
    }

    // Roundtrip law: backward(forward(s)) == s
    proptest! {
        #[test]
        fn law_roundtrip_forward_backward(bytes in prop::collection::vec(any::<u8>(), 0..20)) {
            let iso = string_bytes_iso();
            // Only test valid UTF-8 sequences
            if let Ok(s) = String::from_utf8(bytes) {
                let result = iso.review(iso.get(&s));
                prop_assert_eq!(result, s);
            }
        }
    }

    // Roundtrip law: forward(backward(b)) == b
    proptest! {
        #[test]
        fn law_roundtrip_backward_forward(s in "[a-z]{0,20}") {
            let iso = string_bytes_iso();
            let result = iso.get(&iso.review(iso.get(&s)));
            prop_assert_eq!(result, iso.get(&s));
        }
    }
}
