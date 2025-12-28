use k8s_openapi::chrono::Utc;

use super::*;

pub trait TimeExt {
    fn now() -> metav1::Time;
}

impl TimeExt for metav1::Time {
    /// Create a metav1::Time set to the current UTC time.
    ///
    /// # Examples
    ///
    /// ```
    /// // Requires `metav1` in scope: `use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;`
    /// let now = metav1::Time::now();
    /// ```
    fn now() -> metav1::Time {
        Self(Utc::now())
    }
}