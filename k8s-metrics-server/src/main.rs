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

/// Starts the HTTP server, wires the shared MetricsCollector into Axum routes, and serves the
/// Kubernetes metrics API and health endpoints.
///
/// This function initializes tracing, constructs a MetricsCollector wrapped in an Arc, builds the
/// Axum router with discovery, node, and pod metrics endpoints mounted under the metrics API root,
/// binds a TCP listener on 0.0.0.0:8080, and runs the server until shutdown.
///
/// # Examples
///
/// ```no_run
/// // Run the server binary (no-op in tests)
/// // cargo run --bin k8s-metrics-server
/// ```
///
/// # Returns
///
/// `Ok(())` if the server initialized and ran without error; an error if initialization or binding
/// the listener failed.
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

/// Produce a Kubernetes List containing metrics for all nodes.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// # async fn example() {
/// let collector: Arc<MetricsCollector> = /* obtain collector */ unimplemented!();
/// let json_list = all_nodes(State(collector)).await;
/// // `json_list` is `Json<List<metricsv1::NodeMetrics>>`
/// # }
/// ```
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

/// Fetches metrics for the node with the given name.
///
/// Returns the node's metrics wrapped in `Json` on success, or a `NotFound<metricsv1::NodeMetrics>`
/// error when no metrics exist for the specified node.
///
/// # Examples
///
/// ```no_run
/// use axum::extract::{Path, State};
/// use std::sync::Arc;
///
/// // `collector` should be an `Arc<MetricsCollector>` available in scope.
/// let result = node(Path("node-1".to_string()), State(collector)).await;
/// match result {
///     Ok(json_metrics) => println!("received metrics: {:?}", json_metrics),
///     Err(not_found) => eprintln!("node not found: {:?}", not_found),
/// }
/// ```
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

/// Returns a Kubernetes-style list of PodMetrics for all namespaces.
///
/// The response is a JSON-wrapped `List<metricsv1::PodMetrics>` whose `items` are populated
/// from the shared `MetricsCollector`.
///
/// # Examples
///
/// ```
/// # use std::sync::Arc;
/// # use k8s_metrics::metricsv1;
/// # use k8s_metrics_server::MetricsCollector;
/// # use k8s_metrics_server::main::all_pods;
/// # use axum::extract::State;
/// # use axum::Json;
/// # tokio_test::block_on(async {
/// // Given an Arc<MetricsCollector> named `collector`:
/// // let response: Json<k8s_openapi::List<metricsv1::PodMetrics>> = all_pods(State(collector)).await;
/// // You can access the returned items via:
/// // let list = response.0;
/// // assert!(list.items.len() >= 0);
/// # });
/// ```
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

/// Returns a Kubernetes `List` of `PodMetrics` for the specified namespace.
///
/// The list's `metadata` is set to the default `ListMeta` and `items` contains all pod metrics
/// from the collector restricted to `namespace`.
///
/// # Returns
///
/// A `List<metricsv1::PodMetrics>` containing the pod metrics for the provided namespace.
///
/// # Examples
///
/// ```
/// # async fn example() {
/// use std::sync::Arc;
/// use axum::extract::{Path, State};
/// // `collector` must be an `Arc<MetricsCollector>` previously created.
/// let namespace = String::from("default");
/// let resp = all_namespaced_pods(Path(namespace), State(Arc::clone(&collector))).await;
/// // `resp` is `axum::Json<List<metricsv1::PodMetrics>>`
/// # }
/// ```
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

/// Fetches metrics for the specified pod in the given namespace and returns them as JSON.
///
/// Attempts to retrieve the PodMetrics for `pod` in `namespace`. On success the metrics are
/// returned serialized as Kubernetes `metrics.k8s.io/v1beta1::PodMetrics`; if the pod is not found
/// a Kubernetes-style `NotFound` response is returned.
///
/// # Returns
///
/// `Ok(Json(metricsv1::PodMetrics))` with the pod metrics, `Err(NotFound<metricsv1::PodMetrics>)` if no metrics exist for that pod.
///
/// # Examples
///
/// ```no_run
/// use axum::extract::{Path, State};
/// use axum::Json;
/// use std::sync::Arc;
/// // This example demonstrates the handler signature and expected types; running it requires
/// // a live MetricsCollector and Tokio runtime.
/// async fn call_handler_example(
///     path: Path<(String, String)>,
///     state: State<Arc<dyn crate::MetricsCollector>>,
/// ) -> Result<Json<metricsv1::PodMetrics>, crate::NotFound<metricsv1::PodMetrics>> {
///     crate::namespaced_pod(path, state).await
/// }
/// ```
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

/// Returns the collector's Kubernetes APIResourceList for the metrics API.
///
/// The handler responds with the APIResourceList describing available metric resources.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use axum::extract::State;
/// use k8s_metrics::MetricsCollector;
/// use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
///
/// // Assuming `collector` implements `MetricsCollector` and is constructed elsewhere:
/// // let collector: Arc<dyn MetricsCollector> = Arc::new(MyCollector::new());
/// // let resp = get_api_discovery(State(collector)).await;
/// // let list: metav1::APIResourceList = resp.0;
/// ```
async fn get_api_discovery(
    State(collector): State<Arc<MetricsCollector>>,
) -> Json<metav1::APIResourceList> {
    Json(collector.metrics_api_resource_list())
}

/// Provide a minimal HTTP liveness probe response.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let resp = crate::healthz().await;
/// assert_eq!(resp, "ok");
/// # });
/// ```
async fn healthz() -> &'static str {
    "ok"
}

struct NotFound<K> {
    name: String,
    resource: std::marker::PhantomData<K>,
}

impl<K> NotFound<K> {
    /// Constructs a NotFound error for the specified resource name.
    ///
    /// The generic resource type `K` is carried on the returned value so it can be converted
    /// into a Kubernetes-style `Status` for that resource.
    ///
    /// # Examples
    ///
    /// ```
    /// let nf = NotFound::<metricsv1::PodMetrics>::new("mypod".to_string());
    /// assert_eq!(nf.name, "mypod");
    /// ```
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
    /// Convert the `NotFound` marker into an HTTP 404 response containing a Kubernetes `metav1::Status` payload.
    ///
    /// The response uses HTTP status `404 NOT FOUND` and a `metav1::Status` whose `code` and message indicate the missing resource name and kind.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let resp = NotFound::<metricsv1::NodeMetrics>::new("node-1".into()).into_response();
    /// ```
    fn into_response(self) -> axum::response::Response {
        let code = http::StatusCode::NOT_FOUND;
        let status = metav1::Status {
            code: Some(code.as_u16() as i32),
            ..metav1::Status::not_found::<K>(self.name)
        };
        (code, Json(status)).into_response()
    }
}