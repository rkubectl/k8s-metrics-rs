use super::*;

#[expect(dead_code)]
#[derive(Debug)]
pub(crate) struct Pod {
    pod: corev1::Pod,
    last_polled: Instant,
}
