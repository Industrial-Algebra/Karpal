#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod codensity;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod cofree;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod coyoneda;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod day;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod density;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod free;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod free_alt;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod free_ap;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod freer;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod lan;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod ran;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod yoneda;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use codensity::{Codensity, CodensityF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use cofree::{Cofree, CofreeF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use coyoneda::{Coyoneda, CoyonedaF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use day::{Day, DayF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use density::{Density, DensityF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use free::{Free, FreeF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use free_alt::{FreeAlt, FreeAltF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use free_ap::{FreeAp, FreeApF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use freer::{Freer, FreerF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use lan::{Lan, LanF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use ran::{Ran, RanMapped};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use yoneda::{Yoneda, YonedaF};
