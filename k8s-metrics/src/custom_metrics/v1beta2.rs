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
