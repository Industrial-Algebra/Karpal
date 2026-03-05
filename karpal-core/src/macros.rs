/// Monadic do-notation sugar.
///
/// Desugars sequential `x = expr` bindings into nested `Chain::chain` calls.
///
/// # Example
/// ```ignore
/// do_! { OptionF;
///     x = Some(1);
///     y = Some(x + 1);
///     OptionF::pure(x + y)
/// }
/// ```
#[macro_export]
macro_rules! do_ {
    // Terminal: just an expression (no more bindings)
    ($F:ty; $e:expr) => {
        $e
    };
    // Binding: x = expr; rest...
    ($F:ty; $x:ident = $e:expr; $($rest:tt)+) => {
        <$F as $crate::chain::Chain>::chain($e, |$x| {
            $crate::do_!($F; $($rest)+)
        })
    };
}

/// Applicative do-notation sugar.
///
/// Collects independent bindings and combines them with `ap`/`fmap`.
/// Supports 1–4 bindings followed by `yield expr`.
///
/// # Example
/// ```ignore
/// ado_! { OptionF;
///     x = Some(1);
///     y = Some(2);
///     yield x + y
/// }
/// ```
#[macro_export]
macro_rules! ado_ {
    // 1 binding
    ($F:ty; $x:ident = $e:expr; yield $body:expr) => {
        <$F as $crate::functor::Functor>::fmap($e, |$x| $body)
    };
    // 2 bindings
    ($F:ty; $x1:ident = $e1:expr; $x2:ident = $e2:expr; yield $body:expr) => {
        <$F as $crate::apply::Apply>::ap(
            <$F as $crate::functor::Functor>::fmap($e1, |$x1| move |$x2| $body),
            $e2,
        )
    };
    // 3 bindings
    ($F:ty;
        $x1:ident = $e1:expr;
        $x2:ident = $e2:expr;
        $x3:ident = $e3:expr;
        yield $body:expr
    ) => {
        <$F as $crate::apply::Apply>::ap(
            <$F as $crate::apply::Apply>::ap(
                <$F as $crate::functor::Functor>::fmap($e1, |$x1| move |$x2| move |$x3| $body),
                $e2,
            ),
            $e3,
        )
    };
    // 4 bindings
    ($F:ty;
        $x1:ident = $e1:expr;
        $x2:ident = $e2:expr;
        $x3:ident = $e3:expr;
        $x4:ident = $e4:expr;
        yield $body:expr
    ) => {
        <$F as $crate::apply::Apply>::ap(
            <$F as $crate::apply::Apply>::ap(
                <$F as $crate::apply::Apply>::ap(
                    <$F as $crate::functor::Functor>::fmap($e1, |$x1| {
                        move |$x2| move |$x3| move |$x4| $body
                    }),
                    $e2,
                ),
                $e3,
            ),
            $e4,
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::applicative::Applicative;
    use crate::hkt::OptionF;
    #[cfg(any(feature = "std", feature = "alloc"))]
    use crate::hkt::VecF;

    #[test]
    fn do_option_some() {
        let result = do_! { OptionF;
            x = Some(1);
            y = Some(x + 1);
            OptionF::pure(x + y)
        };
        assert_eq!(result, Some(3));
    }

    #[test]
    fn do_option_none() {
        let result: Option<i32> = do_! { OptionF;
            x = Some(1);
            _y = None::<i32>;
            OptionF::pure(x)
        };
        assert_eq!(result, None);
    }

    #[test]
    fn do_option_single() {
        let result = do_! { OptionF;
            Some(42)
        };
        assert_eq!(result, Some(42));
    }

    #[test]
    fn do_vec() {
        let result = do_! { VecF;
            x = vec![1, 2];
            y = vec![10, 20];
            VecF::pure(x + y)
        };
        assert_eq!(result, vec![11, 21, 12, 22]);
    }

    #[test]
    fn ado_option_1() {
        let result = ado_! { OptionF;
            x = Some(5);
            yield x * 2
        };
        assert_eq!(result, Some(10));
    }

    #[test]
    fn ado_option_2() {
        let result = ado_! { OptionF;
            x = Some(1);
            y = Some(2);
            yield x + y
        };
        assert_eq!(result, Some(3));
    }

    #[test]
    fn ado_option_2_none() {
        let result = ado_! { OptionF;
            x = Some(1);
            y = None::<i32>;
            yield x + y
        };
        assert_eq!(result, None);
    }

    #[test]
    fn ado_option_3() {
        let result = ado_! { OptionF;
            x = Some(1);
            y = Some(2);
            z = Some(3);
            yield x + y + z
        };
        assert_eq!(result, Some(6));
    }

    #[test]
    fn ado_option_4() {
        let result = ado_! { OptionF;
            a = Some(1);
            b = Some(2);
            c = Some(3);
            d = Some(4);
            yield a + b + c + d
        };
        assert_eq!(result, Some(10));
    }

    #[test]
    fn ado_vec_2() {
        let result = ado_! { VecF;
            x = vec![1, 2];
            y = vec![10, 20];
            yield x + y
        };
        assert_eq!(result, vec![11, 21, 12, 22]);
    }
}
