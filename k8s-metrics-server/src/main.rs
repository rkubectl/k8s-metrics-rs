use std::sync::Arc;
use std::time::Duration;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use k8s_metrics::v1beta1 as metricsv1;
// use k8s_metrics::v1beta1::{Container, NodeMetrics, PodMetrics, Usage};
use k8s_openapi::api::core::v1 as corev1;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
use k8s_openapi::{
    chrono::{DateTime, Utc},
    List,
};
use kube::api::{Api, ListParams};
use kube::ResourceExt as _;
use serde_json::{json, Value};

use ext::TimeExt;

mod ext;

struct MetricsCollector {
    client: Option<kube::Client>,
}

impl MetricsCollector {
    async fn new() -> Self {
        match kube::Client::try_default().await {
            Ok(client) => {
                println!("✓ Connected to Kubernetes cluster");
                Self {
                    client: Some(client),
                }
            }
            Err(e) => {
                println!("⚠ Could not connect to Kubernetes cluster: {}", e);
                println!("  Running in demo mode with mock data");
                Self { client: None }
            }
        }
    }

    async fn get_node_metrics(
        &self,
    ) -> Result<List<metricsv1::NodeMetrics>, Box<dyn std::error::Error + Send + Sync>> {
        match &self.client {
            Some(client) => {
                let nodes_api: Api<corev1::Node> = Api::all(client.clone());
                let nodes = nodes_api.list(&ListParams::default()).await?;

                let mut items = Vec::new();
                for node in nodes.items {
                    let node_name = node.name_any();

                    // In a real implementation, you would collect actual metrics from the node
                    // For now, we'll return mock data
                    let node_metrics = metricsv1::NodeMetrics {
                        metadata: metav1::ObjectMeta {
                            name: Some(node_name),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
                        usage: metricsv1::Usage {
                            cpu: Quantity("100m".to_string()),
                            memory: Quantity("200Mi".to_string()),
                        },
                    };

                    items.push(node_metrics);
                }

                Ok(List {
                    metadata: metav1::ListMeta::default(),
                    items,
                })
            }
            None => {
                // Demo mode - return mock data
                let items = vec![
                    metricsv1::NodeMetrics {
                        metadata: metav1::ObjectMeta {
                            name: Some("demo-node-1".to_string()),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
                        usage: metricsv1::Usage {
                            cpu: Quantity("150m".to_string()),
                            memory: Quantity("512Mi".to_string()),
                        },
                    },
                    metricsv1::NodeMetrics {
                        metadata: metav1::ObjectMeta {
                            name: Some("demo-node-2".to_string()),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
                        usage: metricsv1::Usage {
                            cpu: Quantity("200m".to_string()),
                            memory: Quantity("1Gi".to_string()),
                        },
                    },
                ];

                Ok(List {
                    metadata: metav1::ListMeta::default(),
                    items,
                })
            }
        }
    }

    async fn get_pod_metrics(
        &self,
        namespace: Option<String>,
    ) -> Result<List<metricsv1::PodMetrics>, Box<dyn std::error::Error + Send + Sync>> {
        match &self.client {
            Some(client) => {
                let pods_api: Api<corev1::Pod> = if let Some(ns) = namespace {
                    Api::namespaced(client.clone(), &ns)
                } else {
                    Api::all(client.clone())
                };

                let pods = pods_api.list(&ListParams::default()).await?;

                let mut items = Vec::new();
                for pod in pods.items {
                    let pod_name = pod.name_any();
                    let pod_namespace = pod.namespace().unwrap_or_default();

                    // Mock metrics for containers
                    let mut containers = Vec::new();
                    if let Some(spec) = pod.spec {
                        for container in spec.containers {
                            containers.push(metricsv1::Container {
                                name: container.name,
                                usage: metricsv1::Usage {
                                    cpu: Quantity("50m".to_string()),
                                    memory: Quantity("100Mi".to_string()),
                                },
                            });
                        }
                    }

                    let pod_metrics = metricsv1::PodMetrics {
                        metadata: metav1::ObjectMeta {
                            name: Some(pod_name),
                            namespace: Some(pod_namespace),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
                        containers,
                    };

                    items.push(pod_metrics);
                }

                Ok(List {
                    metadata: metav1::ListMeta::default(),
                    items,
                })
            }
            None => {
                // Demo mode - return mock data
                let demo_namespace = namespace.unwrap_or_else(|| "default".to_string());
                let items = vec![
                    metricsv1::PodMetrics {
                        metadata: metav1::ObjectMeta {
                            name: Some("demo-pod-1".to_string()),
                            namespace: Some(demo_namespace.clone()),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
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
                            namespace: Some(demo_namespace),
                            creation_timestamp: Some(metav1::Time::now()),
                            ..default()
                        },
                        timestamp: metav1::Time::now(),
                        window: Duration::from_secs(30),
                        containers: vec![metricsv1::Container {
                            name: "web-server".to_string(),
                            usage: metricsv1::Usage {
                                cpu: Quantity("75m".to_string()),
                                memory: Quantity("128Mi".to_string()),
                            },
                        }],
                    },
                ];

                Ok(List {
                    metadata: metav1::ListMeta::default(),
                    items,
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting k8s-metrics-server...");

    let collector = Arc::new(MetricsCollector::new().await);

    // Create axum router
    let app = Router::new()
        .route(
            "/apis/metrics.k8s.io/v1beta1/nodes",
            get(get_node_metrics_handler),
        )
        .route(
            "/apis/metrics.k8s.io/v1beta1/pods",
            get(get_all_pod_metrics_handler),
        )
        .route(
            "/apis/metrics.k8s.io/v1beta1/namespaces/:namespace/pods",
            get(get_namespaced_pod_metrics_handler),
        )
        .route("/apis/metrics.k8s.io/v1beta1", get(get_api_discovery))
        .route("/healthz", get(health_check))
        .with_state(collector);

    println!("Server running on http://0.0.0.0:8080");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler functions
async fn get_node_metrics_handler(
    axum::extract::State(collector): axum::extract::State<Arc<MetricsCollector>>,
) -> impl IntoResponse {
    match collector.get_node_metrics().await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => {
            eprintln!("Error getting node metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

async fn get_all_pod_metrics_handler(
    axum::extract::State(collector): axum::extract::State<Arc<MetricsCollector>>,
) -> impl IntoResponse {
    match collector.get_pod_metrics(None).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => {
            eprintln!("Error getting pod metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

async fn get_namespaced_pod_metrics_handler(
    Path(namespace): Path<String>,
    axum::extract::State(collector): axum::extract::State<Arc<MetricsCollector>>,
) -> impl IntoResponse {
    match collector.get_pod_metrics(Some(namespace)).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => {
            eprintln!("Error getting namespaced pod metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

async fn get_api_discovery() -> Json<Value> {
    Json(json!({
        "kind": "APIResourceList",
        "apiVersion": "v1",
        "groupVersion": "metrics.k8s.io/v1beta1",
        "resources": [
            {
                "name": "nodes",
                "singularName": "",
                "namespaced": false,
                "kind": "NodeMetrics",
                "verbs": ["get", "list"]
            },
            {
                "name": "pods",
                "singularName": "",
                "namespaced": true,
                "kind": "PodMetrics",
                "verbs": ["get", "list"]
            }
        ]
    }))
}

async fn health_check() -> &'static str {
    "ok"
}

fn default<T: Default>() -> T {
    T::default()
}
