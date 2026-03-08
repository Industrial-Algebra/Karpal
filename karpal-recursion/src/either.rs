/// A simple sum type for use in apomorphism and other schemes.
///
/// `Left` typically represents early termination (an already-computed `Fix`),
/// while `Right` represents continuation (a seed to unfold further).
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Eliminate an `Either` by providing handlers for both cases.
    pub fn either<T>(self, f: impl FnOnce(L) -> T, g: impl FnOnce(R) -> T) -> T {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => g(r),
        }
    }

    /// Map over the `Left` value, leaving `Right` unchanged.
    pub fn map_left<L2>(self, f: impl FnOnce(L) -> L2) -> Either<L2, R> {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Map over the `Right` value, leaving `Left` unchanged.
    pub fn map_right<R2>(self, f: impl FnOnce(R) -> R2) -> Either<L, R2> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn either_left() {
        let e: Either<i32, &str> = Either::Left(42);
        let result = e.either(|x| x.to_string(), |s| s.to_string());
        assert_eq!(result, "42");
    }

    #[test]
    fn either_right() {
        let e: Either<i32, &str> = Either::Right("hello");
        let result = e.either(|x| x.to_string(), |s| s.to_string());
        assert_eq!(result, "hello");
    }

    #[test]
    fn map_left() {
        let e: Either<i32, &str> = Either::Left(10);
        let mapped = e.map_left(|x| x * 2);
        match mapped {
            Either::Left(v) => assert_eq!(v, 20),
            Either::Right(_) => panic!("expected Left"),
        }
    }

    #[test]
    fn map_right() {
        let e: Either<i32, i32> = Either::Right(5);
        let mapped = e.map_right(|x| x + 1);
        match mapped {
            Either::Right(v) => assert_eq!(v, 6),
            Either::Left(_) => panic!("expected Right"),
        }
    }

    #[test]
    fn map_left_on_right_is_noop() {
        let e: Either<i32, &str> = Either::Right("hi");
        let mapped = e.map_left(|x| x * 100);
        match mapped {
            Either::Right(s) => assert_eq!(s, "hi"),
            Either::Left(_) => panic!("expected Right"),
        }
    }

    #[test]
    fn map_right_on_left_is_noop() {
        let e: Either<i32, i32> = Either::Left(7);
        let mapped = e.map_right(|x| x * 100);
        match mapped {
            Either::Left(v) => assert_eq!(v, 7),
            Either::Right(_) => panic!("expected Left"),
        }
    }
}
