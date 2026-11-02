use std::collections::BTreeMap;
use std::marker::PhantomData;

use super::*;

/// `ExternalMetricValue` is a metric value for external metric
///
/// A single metric value is identified by metric name and a set of string labels.
/// For one metric there can be multiple values with different sets of labels.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct ExternalMetricValue<M> {
    /// Local metadata for kube integration; metric values have no wire metadata.
    #[serde(skip)]
    pub metadata: metav1::ObjectMeta,

    /// the name of the metric
    ///
    pub metric_name: String, // `json:"metricName" protobuf:"bytes,1,name=metricName"`

    /// a set of labels that identify a single time series for the metric
    ///
    pub metric_labels: BTreeMap<String, String>, // `json:"metricLabels" protobuf:"bytes,2,rep,name=metricLabels"`

    /// indicates the time at which the metrics were produced
    ///
    pub timestamp: metav1::Time, // `json:"timestamp" protobuf:"bytes,3,name=timestamp"`

    /// indicates the window ([Timestamp-Window, Timestamp]) from
    /// which these metrics were calculated, when returning rate
    /// metrics calculated from cumulative metrics (or zero for
    /// non-calculated instantaneous metrics).
    ///
    #[serde(rename = "window", skip_serializing_if = "Option::is_none")]
    pub window_seconds: Option<i64>,

    /// the value of the metric
    ///
    pub value: resource::Quantity, // `json:"value" protobuf:"bytes,5,name=value"`

    #[serde(skip)]
    phantom: PhantomData<M>,
}

impl<M: ExternalMetric> k8s::Resource for ExternalMetricValue<M> {
    const API_VERSION: &'static str = "external.metrics.k8s.io/v1beta1";
    const GROUP: &'static str = "external.metrics.k8s.io";
    const KIND: &'static str = M::KIND;
    const VERSION: &'static str = "v1beta1";
    const URL_PATH_SEGMENT: &'static str = M::URL_PATH_SEGMENT;
    type Scope = k8s::NamespaceResourceScope;
}

impl<M: ExternalMetric> k8s::Metadata for ExternalMetricValue<M> {
    type Ty = metav1::ObjectMeta;

    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

impl<M: ExternalMetric> k8s::ListableResource for ExternalMetricValue<M> {
    const LIST_KIND: &'static str = "ExternalMetricValueList";
}

pub type ExternalMetricValueList<M> = k8s::List<ExternalMetricValue<M>>;

#[cfg(test)]
mod tests {
    use super::*;

    use k8s::serde_json as json;

    struct Queue;

    impl ExternalMetric for Queue {
        const KIND: &'static str = "ExternalMetricValue";
        const URL_PATH_SEGMENT: &'static str = "queue_messages";
    }

    // Kubernetes 1.37 external_metrics/v1beta1 ExternalMetricValue wire shape.
    const METRIC: &str = r#"{
        "metricName":"queue_messages",
        "metricLabels":{"queue":"jobs"},
        "timestamp":"2026-11-02T10:00:00Z",
        "window":30,
        "value":"10"
    }"#;

    #[test]
    fn upstream_metric_value_round_trip() {
        let item: json::Value = json::from_str(METRIC).unwrap();
        let response = json::json!({
            "apiVersion":"external.metrics.k8s.io/v1beta1",
            "kind":"ExternalMetricValueList",
            "metadata":{},
            "items":[item.clone()]
        });
        let list: ExternalMetricValueList<Queue> = json::from_value(response).unwrap();
        let metric = &list.items[0];
        assert_eq!(metric.metric_name, "queue_messages");
        assert_eq!(metric.window_seconds, Some(30));
        assert_eq!(json::to_value(metric).unwrap(), item);
    }

    #[test]
    fn optional_window_preserves_absent_and_zero() {
        let mut item: json::Value = json::from_str(METRIC).unwrap();
        item.as_object_mut().unwrap().remove("window");
        let metric: ExternalMetricValue<Queue> = json::from_value(item.clone()).unwrap();
        assert_eq!(metric.window_seconds, None);
        assert!(json::to_value(metric).unwrap().get("window").is_none());

        item["window"] = json::json!(null);
        let metric: ExternalMetricValue<Queue> = json::from_value(item.clone()).unwrap();
        assert_eq!(metric.window_seconds, None);

        item["window"] = json::json!(0);
        let metric: ExternalMetricValue<Queue> = json::from_value(item.clone()).unwrap();
        assert_eq!(metric.window_seconds, Some(0));
        assert_eq!(json::to_value(metric).unwrap(), item);
    }

    #[tokio::test]
    async fn namespaced_kube_api_uses_the_metric_endpoint() {
        // Construct the client without sending a request.
        let config = kube::Config::new("http://localhost".parse().unwrap());
        let client = kube::Client::try_from(config).unwrap();
        let api = kube::Api::<ExternalMetricValue<Queue>>::namespaced(client, "default");
        assert_eq!(
            api.resource_url(),
            "/apis/external.metrics.k8s.io/v1beta1/namespaces/default/queue_messages"
        );
    }
}
