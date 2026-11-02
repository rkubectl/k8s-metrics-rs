# Kubernetes Metrics API Resource definitions

Portions of the code are copied from other projects, like kdash

Resource metrics are available as `k8s_metrics::v1` for the stable API introduced
in Kubernetes 1.37, and `k8s_metrics::v1beta1` for existing metrics-server deployments.
The `v1` module is exposed only when `k8s-openapi` selects Kubernetes 1.37 or later.
Choose the version served by your metrics provider. Both versions expose the same
fields and usage helpers.

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
