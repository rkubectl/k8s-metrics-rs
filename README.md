# Kubernetes Metrics API Resource definitions

Portions of the code are copied from other projects, like kdash

Resource metrics are available as `k8s_metrics::v1` for the stable API introduced
in Kubernetes 1.37, and `k8s_metrics::v1beta1` for existing metrics-server deployments.
The `v1` module is exposed only when `k8s-openapi` selects Kubernetes 1.37 or later.
Choose the version served by your metrics provider. Both versions expose the same
fields and usage helpers.

Metric windows use Go's `time.Duration` syntax and must fit its nonnegative
64-bit nanosecond range. The codec checks arithmetic and writes decimal seconds
with nanosecond precision. It accepts both microsecond symbols (`µs` and `μs`),
compound values, and fractional units without relying on `go-parse-duration`.
Jiff's native friendly parser was assessed, but its grammar differs: it rejects
valid Go inputs such as `0`, `.5s`, `1.5h30m`, and repeated units, while accepting
non-Go forms such as `1 hour`. It is therefore not a drop-in wire-format parser.

# Usage example

```rust
use k8s_metrics::v1beta1 as metricsv1;
use kube::api;

async fn pod_metrics(client: &kube::Client, namespace: &str) -> kube::Result<Vec<metricsv1::PodMetrics>> {
    let lp = api::ListParams::default();
    api::Api::<metricsv1::PodMetrics>::namespaced(client.clone(), namespace)
        .list(&lp)
        .await
        .map(|list| list.items)
}
```
