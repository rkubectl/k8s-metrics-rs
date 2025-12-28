use std::fmt::Debug;

use k8s_metrics_ext as k8s;
use kube::api;
use prometheus_parse::Scrape;

use k8s::corev1;
use k8s::metav1;
use k8s::metricsv1;
use k8s::APIResourceExt as _;

pub struct KubeApi {
    get_params: api::GetParams,
    list_params: api::ListParams,
    client: kube::Client,
}

impl KubeApi {
    /// Create a KubeApi configured with a default Kubernetes client.
    ///
    /// On success, returns an initialized `KubeApi` wrapped in `kube::Result`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), kube::Error> {
    /// let api = k8s_metrics_kubeapi::KubeApi::new().await?;
    /// // use `api`...
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> kube::Result<Self> {
        kube::Client::try_default().await.map(Self::with_client)
    }

    /// Create a KubeApi backed by the provided Kubernetes client.
    ///
    /// The returned KubeApi is initialized with default `GetParams` and `ListParams`
    /// and uses `client` for all Kubernetes interactions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = kube::Client::try_default().await?;
    /// let api = KubeApi::with_client(client);
    /// // use `api` to list resources or scrape metrics
    /// # Ok(())
    /// # }
    /// ```
    fn with_client(client: kube::Client) -> Self {
        Self {
            get_params: api::GetParams::default(),
            list_params: api::ListParams::default(),
            client,
        }
    }

    /// Lists metadata for all Nodes in the cluster.
    ///
    /// # Returns
    ///
    /// A `kube::Result` containing a `Vec<api::PartialObjectMeta<corev1::Node>>` with one entry per discovered Node; the `Err` variant indicates a request or API error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use k8s_metrics_kubeapi::KubeApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = KubeApi::new().await?;
    /// let nodes = api.list_nodes().await?;
    /// println!("discovered {} nodes", nodes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_nodes(&self) -> kube::Result<Vec<api::PartialObjectMeta<corev1::Node>>> {
        let lp = self.list_params();
        self.nodes().list_metadata(lp).await.map(|list| list.items)
    }

    /// Retrieve metadata for all Pods accessible through the Kubernetes API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> kube::Result<()> {
    /// let api = KubeApi::new().await?;
    /// let pods = api.list_pods().await?;
    /// // each item contains PartialObjectMeta for a Pod
    /// assert!(pods.iter().all(|p| p.name.is_some() || p.uid.is_some()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// `Vec<api::PartialObjectMeta<corev1::Pod>>` containing the metadata for each Pod; the call fails with a `kube::Error` on error.
    pub async fn list_pods(&self) -> kube::Result<Vec<api::PartialObjectMeta<corev1::Pod>>> {
        let lp = self.list_params();
        self.pods().list_metadata(lp).await.map(|list| list.items)
    }

    /// Aggregates a node's cadvisor and resource metrics and parses them into a `Scrape`.
    ///
    /// Combines the cadvisor and resource metric endpoints for the given node and parses the
    /// concatenated metric lines into a `Scrape`.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn docs() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = KubeApi::new().await?;
    /// let scrape = api.scrape_node_metrics("node-1").await?;
    /// // use `scrape`...
    /// # Ok(()) }
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok(Scrape)` containing the parsed metrics on success, or a `kube::Error` if fetching or parsing fails.
    pub async fn scrape_node_metrics(&self, node: &str) -> kube::Result<Scrape> {
        let cadvisor = self.get_node_cadvisor_metrics(node).await?;
        let resource = self.get_node_resource_metrics(node).await?;
        let lines = cadvisor
            .lines()
            .chain(resource.lines())
            .map(|line| Ok(line.to_string()));
        Scrape::parse(lines).map_err(kube::Error::ReadEvents)
    }

    /// Constructs an APIResourceList for the metrics.k8s.io v1 API containing node and pod metric resources.
    ///
    /// The returned list's `group_version` is the metrics API group and version, and its `resources`
    /// contain the `NodeMetrics` and `PodMetrics` APIResource descriptors.
    ///
    /// # Examples
    ///
    /// ```
    /// let list = KubeApi::with_client(kube::Client::try_default().unwrap()).metrics_api_resource_list();
    /// assert!(list.group_version.starts_with(metricsv1::METRICS_API_GROUP));
    /// assert_eq!(list.resources.len(), 2);
    /// ```
    pub fn metrics_api_resource_list(&self) -> metav1::APIResourceList {
        metav1::APIResourceList {
            group_version: format!(
                "{}/{}",
                metricsv1::METRICS_API_GROUP,
                metricsv1::METRICS_API_VERSION
            ),
            resources: vec![
                metricsv1::NodeMetrics::api_resource(),
                metricsv1::PodMetrics::api_resource(),
            ],
        }
    }

    /// Fetches the raw response body from the Kubernetes API for the given request path.
    ///
    /// # Parameters
    ///
    /// - `name`: API request path or resource URL segment (for example `"/api/v1/nodes/<node>/proxy/metrics/cadvisor"`).
    ///
    /// # Returns
    ///
    /// A `String` containing the response body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(api: &crate::KubeApi) -> kube::Result<()> {
    /// let body = api.raw_get("/api/v1/version").await?;
    /// println!("{}", body);
    /// # Ok(()) }
    /// ```
    async fn raw_get(&self, name: impl AsRef<str>) -> kube::Result<String> {
        let gp = self.get_params();
        let request = api::Request::new("")
            .get(name.as_ref(), gp)
            .map_err(kube::Error::BuildRequest)?;
        self.client.request_text(request).await
    }

    /// Fetches the cAdvisor metrics endpoint for the specified node from the Kubernetes API.
    ///
    /// Returns the raw cAdvisor metrics text as a `String`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(api: &crate::KubeApi) -> kube::Result<()> {
    /// let metrics = api.get_node_cadvisor_metrics("node-1").await?;
    /// assert!(!metrics.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_node_cadvisor_metrics(&self, node: &str) -> kube::Result<String> {
        let name = format!("/api/v1/nodes/{node}/proxy/metrics/cadvisor");
        self.raw_get(&name).await
    }

    /// Fetches the raw text of the node "resource" metrics endpoint for the specified node.
    ///
    /// # Parameters
    ///
    /// - `node` - The name of the Kubernetes node whose resource metrics should be retrieved.
    ///
    /// # Returns
    ///
    /// A `String` containing the plaintext metrics scraped from the node's `/metrics/resource` endpoint.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> kube::Result<()> {
    ///     let api = KubeApi::new().await?;
    ///     let metrics_text = api.get_node_resource_metrics("node-1").await?;
    ///     println!("{}", &metrics_text[..std::cmp::min(80, metrics_text.len())]);
    ///     Ok(())
    /// }
    /// ```
    async fn get_node_resource_metrics(&self, node: &str) -> kube::Result<String> {
        let name = format!("/api/v1/nodes/{node}/proxy/metrics/resource");
        self.raw_get(&name).await
    }

    /// Returns an Api handle scoped to all Nodes using the configured Kubernetes client.
    ///
    /// # Examples
    ///
    /// ```
    /// let api = kube_api.nodes();
    /// let list = futures::executor::block_on(async { api.list(&Default::default()).await }).unwrap();
    /// assert!(list.items.iter().all(|n| n.name.is_some()));
    /// ```
    fn nodes(&self) -> api::Api<corev1::Node> {
        api::Api::all(self.client.clone())
    }

    /// Get an Api handle scoped to all Pods.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// let pods_api = kube_api.pods();
    
    /// ```
    fn pods(&self) -> api::Api<corev1::Pod> {
        api::Api::all(self.client.clone())
    }

    /// Accesses the configured GET query parameters used for API requests.
    ///
    /// # Returns
    ///
    /// A reference to the stored `api::GetParams`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // assume `kube_api` is an existing `KubeApi` instance
    /// let params: &k8s_metrics_ext::kube::api::GetParams = kube_api.get_params();
    /// ```
    fn get_params(&self) -> &api::GetParams {
        &self.get_params
    }

    /// Accesses the default list query parameters used by this API client.
    ///
    /// Returns a reference to the `api::ListParams` that will be applied to list requests.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a `kube_api: KubeApi`, obtain the default list parameters.
    /// let lp = kube_api.list_params();
    /// ```
    fn list_params(&self) -> &api::ListParams {
        &self.list_params
    }
}

impl Debug for KubeApi {
    /// Formats the `KubeApi` for debugging, showing `get_params` and `list_params` while redacting the `client`.
    ///
    /// The `client` field is displayed as the literal `"<kube::Client>"` to avoid exposing internal client details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // `api` is an instance of `KubeApi`
    /// let s = format!("{:?}", /* api */);
    /// assert!(s.contains("get_params"));
    /// assert!(s.contains("list_params"));
    /// assert!(s.contains("<kube::Client>"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KubeApi")
            .field("get_params", &self.get_params)
            .field("list_params", &self.list_params)
            .field("client", &"<kube::Client>")
            .finish()
    }
}