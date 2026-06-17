use std::io;

use k8s_metrics::v1beta1 as metricsv1;
use k8s_metrics_ext as k8s;
use k8s_metrics_kubeapi::KubeApi;
use kube::ResourceExt as _;
use prometheus_parse::Scrape;
use time::ext::NumericalStdDuration as _;
use tokio::sync::Mutex;

use k8s::metav1;
use k8s::resource::Quantity;
use k8s::TimeExt as _;

#[derive(Debug)]
pub struct MetricsCollector {
    kubeapi: KubeApi,
    scrapes: Mutex<Vec<String>>,
}

impl MetricsCollector {
    pub async fn new() -> kube::Result<Self> {
        let kubeapi = KubeApi::new().await?;
        let scrapes = Mutex::new(Vec::new());
        Ok(Self { kubeapi, scrapes })
    }

    pub async fn scrape_metrics(&self) {
        let nodes = self.kubeapi.list_nodes().await.unwrap_or_default();
        let mut metrics = String::new();
        for node in nodes {
            let name = node.name_any();
            if let Ok((cadvisor, resource)) = self
                .kubeapi
                .scrape_node_metrics(&name)
                .await
                .inspect_err(|err| tracing::error!(name, ?err, "Failed to scrape metrics for node"))
            {
                metrics.push_str(&cadvisor);
                metrics.push_str(&resource);
            }
        }
        self.scrapes.lock().await.push(metrics);
    }

    pub fn metrics_api_resource_list(&self) -> metav1::APIResourceList {
        self.kubeapi.metrics_api_resource_list()
    }

    #[expect(clippy::unused_async)]
    pub async fn nodes(&self) -> Vec<metricsv1::NodeMetrics> {
        // In a real implementation, you would collect actual metrics from the node
        // For now, we'll return mock data
        mock::nodes()
    }

    #[expect(clippy::unused_async)]
    pub async fn node(&self, node: &str) -> Option<metricsv1::NodeMetrics> {
        // In a real implementation, you would collect actual metrics from the node
        // For now, we'll return mock data
        (node != "node-5").then(|| mock::node(node.to_string()))
    }

    #[expect(clippy::unused_async)]
    pub async fn pods(&self, namespace: Option<String>) -> Vec<metricsv1::PodMetrics> {
        mock::pods(namespace)
    }

    #[expect(clippy::unused_async)]
    pub async fn pod(&self, name: &str, namespace: &str) -> Option<metricsv1::PodMetrics> {
        (name != "xyz").then(|| mock::pod(name.to_string(), namespace.to_string()))
    }

    pub async fn scrapes(&self) -> Option<String> {
        self.scrapes.lock().await.last().cloned()
    }

    pub async fn parse_scrape(&self) -> io::Result<Scrape> {
        let locked = self.scrapes.lock().await;
        let lines = locked
            .last()
            .map(|text| text.as_str())
            .unwrap_or_default()
            .lines()
            .map(|line| Ok(line.to_string()));
        // let text = locked.last().map(|text| text.as_str()).unwrap_or_default();
        // let lines = text.lines().map(|line| Ok(line.to_string()));
        Scrape::parse(lines)
    }
}

mod mock {
    use super::*;

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
