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
    pub async fn new() -> kube::Result<Self> {
        kube::Client::try_default().await.map(Self::with_client)
    }

    fn with_client(client: kube::Client) -> Self {
        Self {
            get_params: api::GetParams::default(),
            list_params: api::ListParams::default(),
            client,
        }
    }

    pub async fn list_nodes(&self) -> kube::Result<Vec<api::PartialObjectMeta<corev1::Node>>> {
        let lp = self.list_params();
        self.nodes().list_metadata(lp).await.map(|list| list.items)
    }

    pub async fn list_pods(&self) -> kube::Result<Vec<api::PartialObjectMeta<corev1::Pod>>> {
        let lp = self.list_params();
        self.pods().list_metadata(lp).await.map(|list| list.items)
    }

    pub async fn scrape_node_metrics(&self, node: &str) -> kube::Result<Scrape> {
        let cadvisor = self.get_node_cadvisor_metrics(node).await?;
        let resource = self.get_node_resource_metrics(node).await?;
        let lines = cadvisor
            .lines()
            .chain(resource.lines())
            .map(|line| Ok(line.to_string()));
        Scrape::parse(lines).map_err(kube::Error::ReadEvents)
    }

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

    async fn raw_get(&self, name: impl AsRef<str>) -> kube::Result<String> {
        let gp = self.get_params();
        let request = api::Request::new("")
            .get(name.as_ref(), gp)
            .map_err(kube::Error::BuildRequest)?;
        self.client.request_text(request).await
    }

    async fn get_node_cadvisor_metrics(&self, node: &str) -> kube::Result<String> {
        let name = format!("/api/v1/nodes/{node}/proxy/metrics/cadvisor");
        self.raw_get(&name).await
    }

    async fn get_node_resource_metrics(&self, node: &str) -> kube::Result<String> {
        let name = format!("/api/v1/nodes/{node}/proxy/metrics/resource");
        self.raw_get(&name).await
    }

    fn nodes(&self) -> api::Api<corev1::Node> {
        api::Api::all(self.client.clone())
    }

    fn pods(&self) -> api::Api<corev1::Pod> {
        api::Api::all(self.client.clone())
    }

    fn get_params(&self) -> &api::GetParams {
        &self.get_params
    }

    fn list_params(&self) -> &api::ListParams {
        &self.list_params
    }
}

impl Debug for KubeApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KubeApi")
            .field("get_params", &self.get_params)
            .field("list_params", &self.list_params)
            .field("client", &"<kube::Client>")
            .finish()
    }
}
