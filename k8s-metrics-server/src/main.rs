use std::sync::Arc;
use std::time::Instant;

use constcat::concat;
use k8s_metrics::v1beta1 as metricsv1;
use k8s_metrics_collector::MetricsCollector;
use k8s_metrics_ext as k8s;

use k8s::StatusExt as _;
use k8s::corev1;
use k8s::metav1;
use k8s::openapi::List;

use axum::extract::Path;
use axum::extract::State;
use axum::http;
use axum::{Json, Router, response::IntoResponse, routing::get};

mod node;
mod pod;

const METRICS_API_ROOT: &str = concat!("/apis/", metricsv1::METRICS_API_GROUP_VERSION);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("Starting k8s-metrics-server");

    let collector = MetricsCollector::new().await?;
    let collector = Arc::new(collector);

    // Create axum router
    let metrics = Router::new()
        .route("/", get(get_api_discovery))
        .route("/nodes", get(all_nodes))
        .route("/nodes/{node}", get(node))
        .route("/pods", get(all_pods))
        .route("/namespaces/{namespace}/pods", get(all_namespaced_pods))
        .route("/namespaces/{namespace}/pods/{pod}", get(namespaced_pod))
        .with_state(collector);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .nest(METRICS_API_ROOT, metrics);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    if let Ok(addr) = listener.local_addr() {
        tracing::info!("Listening on http://{addr}");
    }
    axum::serve(listener, app).await?;

    Ok(())
}

async fn all_nodes(
    State(collector): State<Arc<MetricsCollector>>,
) -> Json<List<metricsv1::NodeMetrics>> {
    let items = collector.nodes().await;
    let list = List {
        metadata: metav1::ListMeta::default(),
        items,
    };
    Json(list)
}

async fn node(
    Path(node): Path<String>,
    State(collector): State<Arc<MetricsCollector>>,
) -> Result<Json<metricsv1::NodeMetrics>, NotFound<metricsv1::NodeMetrics>> {
    collector
        .node(&node)
        .await
        .map(Json)
        .ok_or(NotFound::<metricsv1::NodeMetrics>::new(node))
}

async fn all_pods(
    State(collector): State<Arc<MetricsCollector>>,
) -> Json<List<metricsv1::PodMetrics>> {
    let items = collector.pods(None).await;
    let list = List {
        metadata: metav1::ListMeta::default(),
        items,
    };
    Json(list)
}

async fn all_namespaced_pods(
    Path(namespace): Path<String>,
    State(collector): State<Arc<MetricsCollector>>,
) -> Json<List<metricsv1::PodMetrics>> {
    let items = collector.pods(Some(namespace)).await;
    let list = List {
        metadata: metav1::ListMeta::default(),
        items,
    };
    Json(list)
}

async fn namespaced_pod(
    Path((namespace, pod)): Path<(String, String)>,
    State(collector): State<Arc<MetricsCollector>>,
) -> Result<Json<metricsv1::PodMetrics>, NotFound<metricsv1::PodMetrics>> {
    collector
        .pod(&pod, &namespace)
        .await
        .map(Json)
        .ok_or(NotFound::<metricsv1::PodMetrics>::new(pod))
}

async fn get_api_discovery(
    State(collector): State<Arc<MetricsCollector>>,
) -> Json<metav1::APIResourceList> {
    Json(collector.metrics_api_resource_list())
}

async fn healthz() -> &'static str {
    "ok"
}

struct NotFound<K> {
    name: String,
    resource: std::marker::PhantomData<K>,
}

impl<K> NotFound<K> {
    fn new(name: String) -> Self {
        Self {
            name,
            resource: std::marker::PhantomData,
        }
    }
}

impl<K> IntoResponse for NotFound<K>
where
    K: k8s::openapi::Resource,
{
    fn into_response(self) -> axum::response::Response {
        let code = http::StatusCode::NOT_FOUND;
        let status = metav1::Status {
            code: Some(code.as_u16() as i32),
            ..metav1::Status::not_found::<K>(self.name)
        };
        (code, Json(status)).into_response()
    }
}
