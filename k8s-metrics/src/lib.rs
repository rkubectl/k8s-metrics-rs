// #![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
// #![warn(clippy::use_self)]
// #![warn(clippy::map_flatten)]
// #![warn(clippy::map_unwrap_or)]
// #![warn(clippy::flat_map_option)]
// #![warn(deprecated_in_future)]
// #![warn(future_incompatible)]
// #![warn(noop_method_call)]
// #![warn(unreachable_pub)]
// #![warn(missing_debug_implementations)]
// #![warn(rust_2018_compatibility)]
// #![warn(rust_2021_compatibility)]
// #![warn(rust_2018_idioms)]
// #![warn(unused)]
// #![deny(warnings)]

use std::time;

use k8s_openapi as k8s;

use serde::{Deserialize, Serialize};

use k8s::api::core::v1 as corev1;
use k8s::apimachinery::pkg::api::resource;
use k8s::apimachinery::pkg::apis::meta::v1 as metav1;
use k8s::chrono::{DateTime, Utc};

pub use metrics::v1beta1;
pub use quantity::{QuantityExt, QuantityParseError};

pub mod custom_metrics;
pub mod external_metrics;
pub mod metrics;
pub mod quantity;

/// Obtain the default value for a type.

///

/// Returns the default value for `T`.

///

/// # Examples

///

/// ```

/// let x: i32 = default::<i32>();

/// assert_eq!(x, 0);

///

/// let s: String = default();

/// assert_eq!(s, String::new());

/// ```
fn default<T: Default>() -> T {
    T::default()
}