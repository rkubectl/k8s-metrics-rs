use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub metadata: metav1::ObjectMeta,
    pub timestamp: metav1::Time,
    #[serde(with = "duration")]
    pub window: time::Duration,
    pub usage: Usage,
}

impl NodeMetrics {
    pub fn cpu(&self) -> Result<f64, QuantityParseError> {
        self.usage.cpu()
    }

    pub fn memory(&self) -> Result<i64, QuantityParseError> {
        self.usage.memory()
    }
}

impl k8s::Resource for NodeMetrics {
    const API_VERSION: &'static str = METRICS_API_GROUP_VERSION;
    const GROUP: &'static str = METRICS_API_GROUP;
    const KIND: &'static str = "NodeMetrics";
    const VERSION: &'static str = METRICS_API_VERSION;
    const URL_PATH_SEGMENT: &'static str = "nodes";
    type Scope = k8s::ClusterResourceScope;
}

impl k8s::ListableResource for NodeMetrics {
    const LIST_KIND: &'static str = "NodeMetricsList";
}

impl k8s::Metadata for NodeMetrics {
    type Ty = metav1::ObjectMeta;

    fn metadata(&self) -> &<Self as k8s::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut <Self as k8s::Metadata>::Ty {
        &mut self.metadata
    }
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            metadata: default(),
            timestamp: metav1::Time(Timestamp::default()),
            window: default(),
            usage: default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s::serde_json as json;

    const NODE: &str = r#"{
  "kind": "NodeMetrics",
  "apiVersion": "metrics.k8s.io/v1beta1",
  "metadata": {
    "name": "docker-desktop",
    "creationTimestamp": "2022-10-09T11:41:56Z",
    "labels": {
      "beta.kubernetes.io/arch": "arm64",
      "beta.kubernetes.io/os": "linux",
      "kubernetes.io/arch": "arm64",
      "kubernetes.io/hostname": "docker-desktop",
      "kubernetes.io/os": "linux",
      "node-role.kubernetes.io/control-plane": "",
      "node.kubernetes.io/exclude-from-external-load-balancers": ""
    }
  },
  "timestamp": "2022-10-09T11:41:45Z",
  "window": "23.5s",
  "usage": {
    "cpu": "196382978n",
    "memory": "1848836Ki"
  }
}"#;

    #[test]
    fn resource() {
        let node: NodeMetrics = json::from_str(NODE).unwrap();
        assert_eq!(node.metadata.name.as_deref(), Some("docker-desktop"));
        assert_eq!(node.window, time::Duration::from_secs_f64(23.5));
        assert_eq!(node.usage.cpu().unwrap(), 0.196382978);
        assert_eq!(node.usage.memory().unwrap(), 1893208064);
    }
}
