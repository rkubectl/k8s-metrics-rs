use k8s_metrics::v1beta1;
use k8s_openapi::Resource as _;

k8s_openapi::k8s_if_ge_1_37! {
    use k8s_metrics::v1;
    use k8s_openapi::List;
    use k8s_openapi::ListableResource as _;
    use k8s_openapi::serde_json as json;

    #[test]
    fn stable_node_metrics() {
        assert_eq!(v1::METRICS_API_VERSION, "v1");
        assert_eq!(v1::NodeMetrics::API_VERSION, "metrics.k8s.io/v1");
        assert_eq!(v1::NodeMetrics::VERSION, "v1");
        assert_eq!(v1::NodeMetrics::KIND, "NodeMetrics");
        assert_eq!(v1::NodeMetrics::LIST_KIND, "NodeMetricsList");
        assert_eq!(
            <v1::NodeMetrics as kube::Resource>::url_path(&(), None),
            "/apis/metrics.k8s.io/v1/nodes"
        );
        let list: List<v1::NodeMetrics> = json::from_value(json::json!({
            "apiVersion": "metrics.k8s.io/v1",
            "kind": "NodeMetricsList",
            "metadata": {},
            "items": [{
                "metadata": {"name": "node"},
                "timestamp": "2026-11-02T10:00:00Z",
                "window": "15s",
                "usage": {"cpu": "100m", "memory": "1Ki"}
            }]
        }))
        .unwrap();
        assert_eq!(list.items[0].cpu().unwrap(), 0.1);
        assert_eq!(list.items[0].memory().unwrap(), 1024);
    }

    #[test]
    fn stable_pod_metrics() {
        assert_eq!(v1::PodMetrics::API_VERSION, "metrics.k8s.io/v1");
        assert_eq!(v1::PodMetrics::VERSION, "v1");
        assert_eq!(v1::PodMetrics::KIND, "PodMetrics");
        assert_eq!(v1::PodMetrics::LIST_KIND, "PodMetricsList");
        assert_eq!(
            <v1::PodMetrics as kube::Resource>::url_path(&(), Some("default")),
            "/apis/metrics.k8s.io/v1/namespaces/default/pods"
        );
        let list: List<v1::PodMetrics> = json::from_value(json::json!({
            "apiVersion": "metrics.k8s.io/v1",
            "kind": "PodMetricsList",
            "metadata": {},
            "items": [{
                "metadata": {"name": "pod", "namespace": "default"},
                "timestamp": "2026-11-02T10:00:00Z",
                "window": "15s",
                "containers": [
                    {"name": "app", "usage": {"cpu": "100m", "memory": "1Ki"}},
                    {"name": "sidecar", "usage": {"cpu": "100m", "memory": "1Ki"}}
                ]
            }]
        }))
        .unwrap();
        assert_eq!(list.items[0].cpu().unwrap(), 0.2);
        assert_eq!(list.items[0].memory().unwrap(), 2048);
    }
}

#[test]
fn beta_resource_paths_are_preserved() {
    assert_eq!(v1beta1::METRICS_API_VERSION, "v1beta1");
    assert_eq!(v1beta1::NodeMetrics::API_VERSION, "metrics.k8s.io/v1beta1");
    assert_eq!(v1beta1::PodMetrics::API_VERSION, "metrics.k8s.io/v1beta1");
    assert_eq!(
        <v1beta1::NodeMetrics as kube::Resource>::url_path(&(), None),
        "/apis/metrics.k8s.io/v1beta1/nodes"
    );
    assert_eq!(
        <v1beta1::PodMetrics as kube::Resource>::url_path(&(), Some("default")),
        "/apis/metrics.k8s.io/v1beta1/namespaces/default/pods"
    );
}
