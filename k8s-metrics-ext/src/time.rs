use k8s_openapi::chrono::Utc;

use super::*;

pub trait TimeExt {
    fn now() -> metav1::Time;
}

impl TimeExt for metav1::Time {
    fn now() -> metav1::Time {
        Self(Utc::now())
    }
}
