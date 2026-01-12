use std::marker::PhantomData;

use super::*;

/// `MetricIdentifier` identifies a metric by name and, optionally, selector
///
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricIdentifier {
    /// name is the name of the given metric
    ///
    pub name: String,
    /// selector represents the label selector that could be used to select
    /// this metric, and will generally just be the selector passed in to
    /// the query used to fetch this metric.
    /// When left blank, only the metric's Name will be used to gather metrics.
    /// +optional
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<metav1::LabelSelector>,
}

impl MetricIdentifier {
    pub fn new(name: impl ToString) -> Self {
        let name = name.to_string();
        let selector = None;
        Self { name, selector }
    }
}

/// `MetricValue` is the metric value for some object
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct MetricValue<M> {
    pub metadata: metav1::ObjectMeta,

    /// a reference to the described object
    ///
    pub described_object: corev1::ObjectReference,

    pub metric: MetricIdentifier,

    /// indicates the time at which the metrics were produced
    ///
    pub timestamp: metav1::Time,

    /// indicates the window ([Timestamp-Window, Timestamp]) from
    /// which these metrics were calculated, when returning rate
    /// metrics calculated from cumulative metrics (or zero for
    /// non-calculated instantaneous metrics).
    ///
    pub window_seconds: i64, // `json:"windowSeconds,omitempty" protobuf:"bytes,4,opt,name=windowSeconds"`

    /// the value of the metric for this
    ///
    pub value: resource::Quantity, // `json:"value" protobuf:"bytes,5,name=value"`

    #[serde(skip)]
    pub phantom: PhantomData<M>,
}

impl<M: k8s::Resource> k8s::Resource for MetricValue<M> {
    const API_VERSION: &'static str = "custom.metrics.k8s.io/v1beta2";
    const GROUP: &'static str = "custom.metrics.k8s.io";
    const KIND: &'static str = M::KIND;
    const VERSION: &'static str = "v1beta2";
    const URL_PATH_SEGMENT: &'static str = M::URL_PATH_SEGMENT;
    type Scope = M::Scope;
}

impl<M: k8s::Metadata> k8s::Metadata for MetricValue<M> {
    type Ty = metav1::ObjectMeta;

    fn metadata(&self) -> &<Self as k8s_openapi::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut <Self as k8s_openapi::Metadata>::Ty {
        &mut self.metadata
    }
}

impl<M> MetricValue<M>
where
    M: k8s::Metadata<Ty = metav1::ObjectMeta>,
{
    /// Create new `MetricValue` for given `object` and `namespace`
    ///
    pub fn new(name: impl ToString, namespace: impl ToString, object: impl ToString) -> Self {
        let name = name.to_string();
        let namespace = namespace.to_string();
        let object_name = object.to_string();

        let metadata = metav1::ObjectMeta {
            name: Some(name.clone()),
            namespace: Some(namespace.clone()),
            ..default()
        };

        let described_object = corev1::ObjectReference {
            name: Some(object_name),
            namespace: Some(namespace),
            api_version: Some(M::API_VERSION.to_string()),
            kind: Some(M::KIND.to_string()),
            ..default()
        };

        let metric = MetricIdentifier::new(name);

        let timestamp = metav1::Time(Timestamp::default());

        Self {
            metadata,
            described_object,
            metric,
            timestamp,
            window_seconds: default(),
            value: default(),
            phantom: PhantomData,
        }
    }

    /// Create `MetricValue` describing object by its `corev1::ObjectReference`
    ///
    pub fn with_object_ref(name: impl ToString, object_ref: &corev1::ObjectReference) -> Self {
        let name = name.to_string();

        let metadata = metav1::ObjectMeta {
            name: Some(name.clone()),
            namespace: object_ref.namespace.clone(),
            ..default()
        };
        let described_object = object_ref.clone();
        let metric = MetricIdentifier::new(name);
        let timestamp = metav1::Time(Timestamp::default());

        Self {
            metadata,
            described_object,
            metric,
            timestamp,
            window_seconds: default(),
            value: default(),
            phantom: PhantomData,
        }
    }

    /// Create `MetricValue` describing `object`
    ///
    pub fn with_object(name: impl ToString, object: &M) -> Self {
        let object_ref = object_ref(object);
        Self::with_object_ref(name, &object_ref)
    }

    /// Set timestamp for this `MetricValue`
    ///
    pub fn timestamp(self, timestamp: Timestamp) -> Self {
        let timestamp = metav1::Time(timestamp);
        Self { timestamp, ..self }
    }
}

impl<M: k8s::ListableResource> k8s::ListableResource for MetricValue<M> {
    const LIST_KIND: &'static str = "MetricValueList";
}

pub type MetricValueList<M> = k8s::List<MetricValue<M>>;

fn object_ref<K>(object: &K) -> corev1::ObjectReference
where
    K: k8s::Metadata<Ty = metav1::ObjectMeta>,
{
    corev1::ObjectReference {
        name: object.metadata().name.clone(),
        namespace: object.metadata().namespace.clone(),
        api_version: Some(K::API_VERSION.to_string()),
        kind: Some(K::KIND.to_string()),
        uid: object.metadata().uid.clone(),
        resource_version: object.metadata().resource_version.clone(),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use k8s::Resource as _;

    use super::*;

    #[test]
    fn metric_identifier_new() {
        let metric = MetricIdentifier::new("cpu_usage");

        assert_eq!(metric.name, "cpu_usage");
        assert!(metric.selector.is_none());
    }

    #[test]
    fn metric_identifier_new_with_string() {
        let metric = MetricIdentifier::new("memory_usage".to_string());

        assert_eq!(metric.name, "memory_usage");
        assert!(metric.selector.is_none());
    }

    #[test]
    fn metric_identifier_new_empty_name() {
        let metric = MetricIdentifier::new("");

        assert_eq!(metric.name, "");
        assert!(metric.selector.is_none());
    }

    #[test]
    fn metric_value_new() {
        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::new("cpu_usage", "default", "test-pod");

        assert_eq!(metric_value.metadata.name.unwrap(), "cpu_usage");
        assert_eq!(metric_value.metadata.namespace.unwrap(), "default");

        assert_eq!(metric_value.described_object.name.unwrap(), "test-pod");
        assert_eq!(metric_value.described_object.namespace.unwrap(), "default");
        assert_eq!(
            metric_value.described_object.api_version.unwrap(),
            corev1::Pod::API_VERSION
        );
        assert_eq!(
            metric_value.described_object.kind.unwrap(),
            corev1::Pod::KIND
        );

        assert_eq!(metric_value.metric.name, "cpu_usage");
        assert!(metric_value.metric.selector.is_none());

        assert_eq!(metric_value.window_seconds, 0);
        assert_eq!(metric_value.value, resource::Quantity::default());
    }

    #[test]
    fn metric_value_new_with_string_inputs() {
        let metric_value: MetricValue<corev1::Pod> = MetricValue::new(
            "memory_usage".to_string(),
            "kube-system".to_string(),
            "nginx-pod".to_string(),
        );

        assert_eq!(metric_value.metadata.name.unwrap(), "memory_usage");
        assert_eq!(metric_value.metadata.namespace.unwrap(), "kube-system");

        assert_eq!(metric_value.described_object.name.unwrap(), "nginx-pod");
        assert_eq!(
            metric_value.described_object.namespace.unwrap(),
            "kube-system"
        );
        assert_eq!(
            metric_value.described_object.api_version.unwrap(),
            corev1::Pod::API_VERSION
        );
        assert_eq!(
            metric_value.described_object.kind.unwrap(),
            corev1::Pod::KIND
        );

        assert_eq!(metric_value.metric.name, "memory_usage");
    }

    #[test]
    fn metric_value_new_empty_inputs() {
        let metric_value: MetricValue<corev1::Pod> = MetricValue::new("", "", "");

        assert_eq!(metric_value.metadata.name.unwrap(), "");
        assert_eq!(metric_value.metadata.namespace.unwrap(), "");

        assert_eq!(metric_value.described_object.name.unwrap(), "");
        assert_eq!(metric_value.described_object.namespace.unwrap(), "");

        assert_eq!(metric_value.metric.name, "");
    }

    #[test]
    fn metric_value_with_object_ref() {
        let pod = corev1::Pod {
            metadata: metav1::ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("production".to_string()),
                uid: Some("12345-67890".to_string()),
                resource_version: Some("123".to_string()),
                ..default()
            },
            ..default()
        };
        let object_ref = object_ref(&pod);

        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::with_object_ref("network_bytes", &object_ref);

        assert_eq!(metric_value.metadata.name.unwrap(), "network_bytes");
        assert_eq!(metric_value.metadata.namespace.unwrap(), "production");

        assert_eq!(metric_value.described_object.name.unwrap(), "test-pod");
        assert_eq!(
            metric_value.described_object.namespace.unwrap(),
            "production"
        );
        assert_eq!(
            metric_value.described_object.api_version.unwrap(),
            corev1::Pod::API_VERSION
        );
        assert_eq!(
            metric_value.described_object.kind.unwrap(),
            corev1::Pod::KIND
        );
        assert_eq!(metric_value.described_object.uid.unwrap(), "12345-67890");
        assert_eq!(
            metric_value.described_object.resource_version.unwrap(),
            "123"
        );

        assert_eq!(metric_value.metric.name, "network_bytes");
        assert!(metric_value.metric.selector.is_none());

        assert_eq!(metric_value.window_seconds, 0);
        assert_eq!(metric_value.value, resource::Quantity::default());
    }

    #[test]
    fn metric_value_with_object_ref_no_namespace() {
        let node = corev1::Node {
            metadata: metav1::ObjectMeta {
                name: Some("cluster-resource".to_string()),
                namespace: None,
                ..default()
            },
            ..default()
        };
        let object_ref = object_ref(&node);

        let metric_value: MetricValue<corev1::Node> =
            MetricValue::with_object_ref("disk_usage", &object_ref);

        assert_eq!(metric_value.metadata.name.unwrap(), "disk_usage");
        assert!(metric_value.metadata.namespace.is_none());

        assert_eq!(
            metric_value.described_object.name.unwrap(),
            "cluster-resource"
        );
        assert!(metric_value.described_object.namespace.is_none());
        assert_eq!(
            metric_value.described_object.api_version.unwrap(),
            corev1::Node::API_VERSION
        );
        assert_eq!(
            metric_value.described_object.kind.unwrap(),
            corev1::Node::KIND
        );

        assert_eq!(metric_value.metric.name, "disk_usage");
    }

    #[test]
    fn metric_value_with_object_ref_with_string_name() {
        let pod = corev1::Pod {
            metadata: metav1::ObjectMeta {
                name: Some("example-pod".to_string()),
                namespace: Some("test".to_string()),
                ..default()
            },
            ..default()
        };
        let object_ref = object_ref(&pod);

        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::with_object_ref("custom_metric".to_string(), &object_ref);

        assert_eq!(metric_value.metadata.name.unwrap(), "custom_metric");
        assert_eq!(metric_value.metric.name, "custom_metric");
    }

    #[test]
    fn metric_value_with_object() {
        let pod = corev1::Pod {
            metadata: metav1::ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("development".to_string()),
                uid: Some("abc123".to_string()),
                resource_version: Some("456".to_string()),
                ..default()
            },
            ..default()
        };

        let metric_value: MetricValue<corev1::Pod> = MetricValue::with_object("pod_restarts", &pod);

        assert_eq!(metric_value.metadata.name.unwrap(), "pod_restarts");
        assert_eq!(metric_value.metadata.namespace.unwrap(), "development");

        assert_eq!(metric_value.described_object.name.unwrap(), "test-pod");
        assert_eq!(
            metric_value.described_object.namespace.unwrap(),
            "development"
        );
        assert_eq!(
            metric_value.described_object.api_version.unwrap(),
            corev1::Pod::API_VERSION
        );
        assert_eq!(
            metric_value.described_object.kind.unwrap(),
            corev1::Pod::KIND
        );
        assert_eq!(metric_value.described_object.uid.unwrap(), "abc123");
        assert_eq!(
            metric_value.described_object.resource_version.unwrap(),
            "456"
        );

        assert_eq!(metric_value.metric.name, "pod_restarts");
        assert!(metric_value.metric.selector.is_none());

        assert_eq!(metric_value.window_seconds, 0);
        assert_eq!(metric_value.value, resource::Quantity::default());
    }

    #[test]
    fn metric_value_with_object_minimal_metadata() {
        let pod = corev1::Pod {
            metadata: metav1::ObjectMeta {
                name: Some("minimal-pod".to_string()),
                ..default()
            },
            ..default()
        };

        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::with_object("errors_per_second", &pod);

        assert_eq!(metric_value.metadata.name.unwrap(), "errors_per_second");
        assert!(metric_value.metadata.namespace.is_none());

        assert_eq!(metric_value.described_object.name.unwrap(), "minimal-pod");
        assert!(metric_value.described_object.namespace.is_none());

        assert_eq!(metric_value.metric.name, "errors_per_second");
    }

    #[test]
    fn metric_value_timestamp() {
        let timestamp = Timestamp::now();
        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::new("cpu_usage", "default", "test-pod").timestamp(timestamp);

        assert_eq!(metric_value.timestamp, metav1::Time(timestamp));
    }

    #[test]
    fn metric_value_chaining_constructors() {
        let pod = corev1::Pod {
            metadata: metav1::ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("staging".to_string()),
                ..default()
            },
            ..default()
        };
        let object_ref = object_ref(&pod);

        let timestamp = Timestamp::now();
        let metric_value: MetricValue<corev1::Pod> =
            MetricValue::with_object_ref("requests_per_minute", &object_ref).timestamp(timestamp);

        assert_eq!(metric_value.metadata.name.unwrap(), "requests_per_minute");
        assert_eq!(metric_value.timestamp, metav1::Time(timestamp));
    }
}
