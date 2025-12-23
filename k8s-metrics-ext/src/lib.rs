pub use k8s_metrics::metrics::v1beta1 as metricsv1;
pub use k8s_openapi as openapi;
pub use k8s_openapi::api::core::v1 as corev1;
pub use k8s_openapi::apimachinery::pkg::api::resource;
pub use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;

pub use time::TimeExt;

use openapi::Resource;

mod time;

pub trait PodMetricsExt {
    fn new(name: impl ToString, namespace: impl ToString) -> Self;
    fn api_resource() -> metav1::APIResource;
}

impl PodMetricsExt for metricsv1::PodMetrics {
    fn new(name: impl ToString, namespace: impl ToString) -> Self {
        let metadata = metav1::ObjectMeta::with_namespace(name, namespace);
        Self {
            metadata,
            ..default()
        }
    }
    fn api_resource() -> metav1::APIResource {
        metav1::APIResource {
            name: Self::URL_PATH_SEGMENT.to_string(),
            namespaced: true,
            kind: Self::KIND.to_string(),
            group: Some(Self::GROUP.to_string()),
            version: Some(Self::VERSION.to_string()),
            ..default()
        }
    }
}

pub trait NodeMetricsExt {
    fn new(name: impl ToString) -> Self;
    fn api_resource() -> metav1::APIResource;
}

pub trait ObjectMetaExt {
    fn new(name: impl ToString) -> Self;
    fn with_namespace(name: impl ToString, namespace: impl ToString) -> Self;
    fn created(self, ts: impl Into<Option<metav1::Time>>) -> Self;
}

impl ObjectMetaExt for metav1::ObjectMeta {
    fn new(name: impl ToString) -> Self {
        let name = Some(name.to_string());
        Self { name, ..default() }
    }

    fn with_namespace(name: impl ToString, namespace: impl ToString) -> Self {
        Self {
            namespace: Some(namespace.to_string()),
            ..Self::new(name)
        }
    }

    fn created(self, ts: impl Into<Option<metav1::Time>>) -> Self {
        Self {
            creation_timestamp: ts.into(),
            ..self
        }
    }
}

pub trait APIResourceExt {
    fn api_resource() -> metav1::APIResource;
}

impl APIResourceExt for metricsv1::PodMetrics {
    fn api_resource() -> metav1::APIResource {
        metav1::APIResource {
            name: Self::URL_PATH_SEGMENT.to_string(),
            namespaced: true,
            kind: Self::KIND.to_string(),
            // group: Some(Self::GROUP.to_string()),
            // version: Some(Self::VERSION.to_string()),
            verbs: vec!["get".to_string(), "list".to_string()],
            ..default()
        }
    }
}

impl APIResourceExt for metricsv1::NodeMetrics {
    fn api_resource() -> metav1::APIResource {
        metav1::APIResource {
            name: Self::URL_PATH_SEGMENT.to_string(),
            namespaced: false,
            kind: Self::KIND.to_string(),
            // group: Some(Self::GROUP.to_string()),
            // version: Some(Self::VERSION.to_string()),
            verbs: vec!["get".to_string(), "list".to_string()],
            ..default()
        }
    }
}

pub trait StatusExt {
    fn not_found<K>(name: impl ToString) -> Self
    where
        K: Resource;
}

impl StatusExt for metav1::Status {
    fn not_found<K>(name: impl ToString) -> Self
    where
        K: Resource,
    {
        let kind = K::URL_PATH_SEGMENT.to_string();
        let name = name.to_string();
        let code = 404;
        let message = format!(r#"{kind} "{name}" not found"#);
        let details = metav1::StatusDetails {
            name: Some(name),
            kind: Some(kind),
            ..default()
        };
        Self {
            code: Some(code),
            details: Some(details),
            message: Some(message),
            metadata: metav1::ListMeta::default(),
            reason: Some("NotFound".to_string()),
            status: Some("Failure".to_string()),
        }
    }
}

pub fn default<T: Default>() -> T {
    T::default()
}
