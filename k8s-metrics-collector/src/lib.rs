use k8s_metrics::v1beta1 as metricsv1;
use k8s_metrics_ext as k8s;
use k8s_metrics_kubeapi::KubeApi;
use prometheus_parse::Scrape;
use time::ext::NumericalStdDuration as _;

use k8s::metav1;
use k8s::resource::Quantity;
use k8s::TimeExt as _;

#[derive(Debug)]
pub struct MetricsCollector {
    kubeapi: KubeApi,
    scrapes: Vec<Scrape>,
}

impl MetricsCollector {
    /// Create a new `MetricsCollector` connected to the Kubernetes metrics API.
    ///
    /// Initializes an underlying `KubeApi` and returns a `MetricsCollector` with an empty
    /// `scrapes` list on success.
    ///
    /// # Returns
    ///
    /// - `Ok(MetricsCollector)` with an initialized `KubeApi` and an empty `scrapes` vector.
    /// - `Err(kube::Error)` if initialization of the `KubeApi` fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Run in a Tokio runtime or similar executor:
    /// # use k8s_metrics_collector::MetricsCollector;
    /// # fn _run() {
    /// let collector = tokio::runtime::Runtime::new()
    ///     .unwrap()
    ///     .block_on(MetricsCollector::new())
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn new() -> kube::Result<Self> {
        let kubeapi = KubeApi::new().await?;
        Ok(Self {
            kubeapi,
            scrapes: Vec::new(),
        })
    }

    /// Retrieves the Kubernetes metrics API resource list.
    ///
    /// # Returns
    /// `metav1::APIResourceList` describing the metrics API resources available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Assume `collector` is an initialized `MetricsCollector`.
    /// let resources = collector.metrics_api_resource_list();
    /// // `resources` contains API resource metadata for metrics endpoints.
    /// ```
    pub fn metrics_api_resource_list(&self) -> metav1::APIResourceList {
        self.kubeapi.metrics_api_resource_list()
    }

    /// Provide mocked node metrics for the cluster.
    ///
    /// # Returns
    ///
    /// `Vec<metricsv1::NodeMetrics>` containing mock NodeMetrics entries.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn fetch_mock_nodes() {
    ///     let collector = MetricsCollector::new().await.unwrap();
    ///     let nodes = collector.nodes().await;
    ///     assert!(!nodes.is_empty());
    /// }
    /// ```
    #[expect(clippy::unused_async)]
    pub async fn nodes(&self) -> Vec<metricsv1::NodeMetrics> {
        // In a real implementation, you would collect actual metrics from the node
        // For now, we'll return mock data
        mock::nodes()
    }

    /// Fetches mock metrics for a single node by name.
    ///
    /// Returns `Some(metricsv1::NodeMetrics)` for the given node name, or `None` when the node name is `"node-5"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use k8s_metrics_collector::MetricsCollector;
    /// # async fn example(collector: &MetricsCollector) {
    /// let some_metrics = collector.node("node-1").await;
    /// assert!(some_metrics.is_some());
    ///
    /// let no_metrics = collector.node("node-5").await;
    /// assert!(no_metrics.is_none());
    /// # }
    /// ```
    #[expect(clippy::unused_async)]
    pub async fn node(&self, node: &str) -> Option<metricsv1::NodeMetrics> {
        // In a real implementation, you would collect actual metrics from the node
        // For now, we'll return mock data
        (node != "node-5").then(|| mock::node(node.to_string()))
    }

    /// Provides mocked PodMetrics for a given namespace.
    ///
    /// If `namespace` is `None`, the mock data is produced for the `"default"` namespace.
    ///
    /// # Parameters
    ///
    /// - `namespace`: Optional namespace to produce pod metrics for; `None` defaults to `"default"`.
    ///
    /// # Returns
    ///
    /// A `Vec<metricsv1::PodMetrics>` containing mocked pod metrics for the requested namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// // Call from a synchronous test harness:
    /// // let pods = futures::executor::block_on(collector.pods(Some("kube-system".to_string())));
    /// // assert!(!pods.is_empty());
    /// ```
    #[expect(clippy::unused_async)]
    pub async fn pods(&self, namespace: Option<String>) -> Vec<metricsv1::PodMetrics> {
        mock::pods(namespace)
    }

    /// Retrieves mock metrics for a pod in the given namespace.
    ///
    /// Returns `Some(metricsv1::PodMetrics)` containing mock container usage when `name` is not `"xyz"`,
    /// and `None` when `name` equals `"xyz"`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use k8s_metrics_collector::MetricsCollector;
    /// # async fn example() -> Option<()> {
    /// let collector = MetricsCollector::new().await.unwrap();
    /// let m = collector.pod("my-pod", "default").await;
    /// assert!(m.is_some());
    ///
    /// let none = collector.pod("xyz", "default").await;
    /// assert!(none.is_none());
    /// # Some(()) }
    /// ```
    #[expect(clippy::unused_async)]
    pub async fn pod(&self, name: &str, namespace: &str) -> Option<metricsv1::PodMetrics> {
        (name != "xyz").then(|| mock::pod(name.to_string(), namespace.to_string()))
    }

    /// Returns a reference to the most recent Prometheus scrape record.
    ///
    /// # Returns
    /// `Some(&Scrape)` with the latest scrape, `None` if no scrapes are recorded.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given an existing `collector: MetricsCollector`
    /// if let Some(latest) = collector.scrapes() {
    ///     // use `latest` (type: &Scrape)
    ///     let _ = latest;
    /// } else {
    ///     // no scrapes available
    /// }
    /// ```
    pub fn scrapes(&self) -> Option<&Scrape> {
        self.scrapes.last()
    }
}

mod mock {
    use super::*;

    /// Provides mocked node metrics for two demo nodes.
    ///
    /// Each `NodeMetrics` contains metadata (name and creation timestamp), the current
    /// timestamp, a 30-second window, and CPU/memory usage quantities.
    ///
    /// # Examples
    ///
    /// ```
    /// let nodes = crate::mock::nodes();
    /// assert_eq!(nodes.len(), 2);
    /// assert_eq!(nodes[0].metadata.name.as_deref(), Some("demo-node-1"));
    /// assert_eq!(nodes[1].metadata.name.as_deref(), Some("demo-node-2"));
    /// ```
    pub(super) fn nodes() -> Vec<metricsv1::NodeMetrics> {
        vec![
            metricsv1::NodeMetrics {
                metadata: metav1::ObjectMeta {
                    name: Some("demo-node-1".to_string()),
                    creation_timestamp: Some(metav1::Time::now()),
                    ..k8s::default()
                },
                timestamp: metav1::Time::now(),
                window: 30.std_seconds(),
                usage: metricsv1::Usage {
                    cpu: Quantity("150m".to_string()),
                    memory: Quantity("512Mi".to_string()),
                },
            },
            metricsv1::NodeMetrics {
                metadata: metav1::ObjectMeta {
                    name: Some("demo-node-2".to_string()),
                    creation_timestamp: Some(metav1::Time::now()),
                    ..k8s::default()
                },
                timestamp: metav1::Time::now(),
                window: 30.std_seconds(),
                usage: metricsv1::Usage {
                    cpu: Quantity("200m".to_string()),
                    memory: Quantity("1Gi".to_string()),
                },
            },
        ]
    }

    /// Creates a mock `NodeMetrics` for the given node name.
    ///
    /// The returned `NodeMetrics` contains populated metadata (including `name` and current
    /// creation/timestamp), a 30-second window, and fixed CPU/memory usage values.
    ///
    /// # Examples
    ///
    /// ```
    /// let nm = node("node-1".to_string());
    /// assert_eq!(nm.metadata.name.as_deref(), Some("node-1"));
    /// ```
    pub(super) fn node(name: String) -> metricsv1::NodeMetrics {
        metricsv1::NodeMetrics {
            metadata: metav1::ObjectMeta {
                name: Some(name),
                creation_timestamp: Some(metav1::Time::now()),
                ..k8s::default()
            },
            timestamp: metav1::Time::now(),
            window: 30.std_seconds(),
            usage: metricsv1::Usage {
                cpu: Quantity("100m".to_string()),
                memory: Quantity("200Mi".to_string()),
            },
        }
    }

    /// Produces a small set of mocked PodMetrics for the given namespace.
    ///
    /// If `namespace` is `None`, the metrics are generated for the `"default"` namespace.
    /// The returned vector contains two sample pods with container CPU and memory usage values.
    ///
    /// # Parameters
    ///
    /// - `namespace`: Optional namespace for the generated PodMetrics; uses `"default"` when `None`.
    ///
    /// # Returns
    ///
    /// A `Vec<metricsv1::PodMetrics>` containing mocked metrics for two pods in the resolved namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let pods = pods(Some("kube-system".to_string()));
    /// assert!(pods.iter().all(|p| p.metadata.namespace.as_deref() == Some("kube-system")));
    /// ```
    pub(super) fn pods(namespace: Option<String>) -> Vec<metricsv1::PodMetrics> {
        let namespace = namespace.unwrap_or_else(|| "default".to_string());
        vec![
            metricsv1::PodMetrics {
                metadata: metav1::ObjectMeta {
                    name: Some("demo-pod-1".to_string()),
                    namespace: Some(namespace.clone()),
                    creation_timestamp: Some(metav1::Time::now()),
                    ..k8s::default()
                },
                timestamp: metav1::Time::now(),
                window: 30.std_seconds(),
                containers: vec![
                    metricsv1::Container {
                        name: "app-container".to_string(),
                        usage: metricsv1::Usage {
                            cpu: Quantity("25m".to_string()),
                            memory: Quantity("64Mi".to_string()),
                        },
                    },
                    metricsv1::Container {
                        name: "sidecar-container".to_string(),
                        usage: metricsv1::Usage {
                            cpu: Quantity("10m".to_string()),
                            memory: Quantity("32Mi".to_string()),
                        },
                    },
                ],
            },
            metricsv1::PodMetrics {
                metadata: metav1::ObjectMeta {
                    name: Some("demo-pod-2".to_string()),
                    namespace: Some(namespace.clone()),
                    creation_timestamp: Some(metav1::Time::now()),
                    ..k8s::default()
                },
                timestamp: metav1::Time::now(),
                window: 30.std_seconds(),
                containers: vec![metricsv1::Container {
                    name: "web-server".to_string(),
                    usage: metricsv1::Usage {
                        cpu: Quantity("75m".to_string()),
                        memory: Quantity("128Mi".to_string()),
                    },
                }],
            },
        ]
    }

    /// Creates a PodMetrics object for the given pod name and namespace populated with a single
    /// container named "web-server" using mock CPU and memory usage values.
    ///
    /// The returned `PodMetrics` includes metadata (name, namespace, creation timestamp),
    /// a current timestamp, a 30-second window, and one container with CPU = "75m" and memory = "128Mi".
    ///
    /// # Examples
    ///
    /// ```
    /// let pm = pod("my-pod".to_string(), "default".to_string());
    /// assert_eq!(pm.metadata.name.as_deref(), Some("my-pod"));
    /// assert_eq!(pm.metadata.namespace.as_deref(), Some("default"));
    /// assert_eq!(pm.containers.len(), 1);
    /// assert_eq!(pm.containers[0].name, "web-server");
    /// assert_eq!(pm.containers[0].usage.cpu.0, "75m");
    /// assert_eq!(pm.containers[0].usage.memory.0, "128Mi");
    /// ```
    pub(super) fn pod(name: String, namespace: String) -> metricsv1::PodMetrics {
        metricsv1::PodMetrics {
            metadata: metav1::ObjectMeta {
                name: Some(name),
                namespace: Some(namespace),
                creation_timestamp: Some(metav1::Time::now()),
                ..k8s::default()
            },
            timestamp: metav1::Time::now(),
            window: 30.std_seconds(),
            containers: vec![metricsv1::Container {
                name: "web-server".to_string(),
                usage: metricsv1::Usage {
                    cpu: Quantity("75m".to_string()),
                    memory: Quantity("128Mi".to_string()),
                },
            }],
        }
    }
}