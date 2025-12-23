use super::*;

#[expect(dead_code)]
#[derive(Debug)]
pub(crate) struct Node {
    node: corev1::Node,
    last_polled: Instant,
}
