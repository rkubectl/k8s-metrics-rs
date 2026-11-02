use k8s_openapi as k8s;

use serde::Deserialize;
use serde::Serialize;

use k8s::api::core::v1 as corev1;
use k8s::apimachinery::pkg::api::resource;
use k8s::apimachinery::pkg::apis::meta::v1 as metav1;
use k8s::jiff::Timestamp;

pub use metrics::v1beta1;
k8s::k8s_if_ge_1_37! {
    pub use metrics::v1;
}
pub use quantity::{QuantityExt, QuantityParseError};

pub mod custom_metrics;
pub mod external_metrics;
pub mod metrics;
pub mod quantity;

pub const METRICS_API_GROUP: &str = "metrics.k8s.io";

fn default<T: Default>() -> T {
    T::default()
}
