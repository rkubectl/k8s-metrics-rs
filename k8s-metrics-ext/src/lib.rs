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
    /// Creates a `PodMetrics` value whose metadata has the given name and namespace; all other fields are set to their defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::metricsv1;
    ///
    /// let pm = metricsv1::PodMetrics::new("mypod", "myns");
    /// assert_eq!(pm.metadata.name.as_deref(), Some("mypod"));
    /// assert_eq!(pm.metadata.namespace.as_deref(), Some("myns"));
    /// ```
    fn new(name: impl ToString, namespace: impl ToString) -> Self {
        let metadata = metav1::ObjectMeta::with_namespace(name, namespace);
        Self {
            metadata,
            ..default()
        }
    }
    /// Returns the Kubernetes APIResource descriptor for this metrics resource.
    ///
    /// The descriptor includes the resource name (URL path segment), that it is namespaced,
    /// its kind, and its API group and version.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::metricsv1;
    /// use crate::metav1;
    ///
    /// let r: metav1::APIResource = metricsv1::PodMetrics::api_resource();
    /// assert_eq!(r.name, metricsv1::PodMetrics::URL_PATH_SEGMENT);
    /// assert!(r.namespaced);
    /// assert_eq!(r.kind, metricsv1::PodMetrics::KIND);
    /// assert_eq!(r.group.as_deref(), Some(metricsv1::PodMetrics::GROUP));
    /// assert_eq!(r.version.as_deref(), Some(metricsv1::PodMetrics::VERSION));
    /// ```
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
    /// Creates an `ObjectMeta` with `name` set and all other fields left as defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// let meta = metav1::ObjectMeta::new("my-pod");
    /// assert_eq!(meta.name.as_deref(), Some("my-pod"));
    /// ```
    fn new(name: impl ToString) -> Self {
        let name = Some(name.to_string());
        Self { name, ..default() }
    }

    /// Creates an `ObjectMeta` with the given name and namespace.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```rust
    
    /// use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
    
    ///
    
    /// let meta = metav1::ObjectMeta::with_namespace("mypod", "default");
    
    /// assert_eq!(meta.name.as_deref(), Some("mypod"));
    
    /// assert_eq!(meta.namespace.as_deref(), Some("default"));
    
    /// ```
    fn with_namespace(name: impl ToString, namespace: impl ToString) -> Self {
        Self {
            namespace: Some(namespace.to_string()),
            ..Self::new(name)
        }
    }

    /// Sets the object's creation timestamp and returns the updated `ObjectMeta`.
    ///
    /// The `ts` argument can be a `metav1::Time`, `Option<metav1::Time>`, or `None` (via `Into<Option<_>>`).
    /// Passing `None` clears the creation timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
    ///
    /// let t = metav1::Time::default();
    /// let meta = metav1::ObjectMeta::default().created(Some(t.clone()));
    /// assert_eq!(meta.creation_timestamp, Some(t));
    /// ```
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
    /// API resource descriptor for PodMetrics.
    ///
    /// Produces a metav1::APIResource with the resource `name` set to `Self::URL_PATH_SEGMENT`,
    /// `namespaced` set to `true`, `kind` set to `Self::KIND`, `verbs` containing `["get", "list"]`,
    /// and other fields set to their defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// let api = <metricsv1::PodMetrics as APIResourceExt>::api_resource();
    /// assert!(api.namespaced);
    /// assert_eq!(api.verbs, vec!["get".to_string(), "list".to_string()]);
    /// ```
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
    /// Provides the Kubernetes API resource descriptor for this type.
    ///
    /// The returned descriptor uses the type's URL path segment as the resource name, marks the
    /// resource as cluster-scoped, sets the kind to the type's Kind, and advertises the `get` and
    /// `list` verbs.
    ///
    /// # Examples
    ///
    /// ```
    /// let res = metricsv1::NodeMetrics::api_resource();
    /// assert_eq!(res.namespaced, false);
    /// assert_eq!(res.kind, metricsv1::NodeMetrics::KIND.to_string());
    /// assert_eq!(res.verbs, vec!["get".to_string(), "list".to_string()]);
    /// ```
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
    /// Constructs a Kubernetes `Status` representing a NotFound error for the specified resource name.
    ///
    /// The returned `Status` has HTTP code `404`, reason `"NotFound"`, status `"Failure"`,
    /// a human-readable message of the form `<kind> "<name>" not found`, and `details` containing
    /// the resource `kind` and `name`.
    ///
    /// # Examples
    ///
    /// ```
    /// use k8s_openapi::api::core::v1::Pod;
    /// use k8s_openapi::apimachinery::pkg::apis::meta::v1::Status;
    ///
    /// let s = Status::not_found::<Pod>("mypod");
    /// assert_eq!(s.code, Some(404));
    /// assert_eq!(s.reason.as_deref(), Some("NotFound"));
    /// assert_eq!(s.status.as_deref(), Some("Failure"));
    /// assert!(s.message.unwrap().contains(r#"Pod "mypod" not found"#));
    /// assert_eq!(s.details.unwrap().name.as_deref(), Some("mypod"));
    /// ```
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

/// Return the default value for the given type.
///
/// # Examples
///
/// ```
/// let x: i32 = default::<i32>();
/// assert_eq!(x, 0);
/// ```
pub fn default<T: Default>() -> T {
    T::default()
}