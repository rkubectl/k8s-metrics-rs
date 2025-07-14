use super::*;

pub(crate) trait TimeExt {
    fn now() -> metav1::Time {
        metav1::Time(Utc::now())
    }
}

impl TimeExt for metav1::Time {}
