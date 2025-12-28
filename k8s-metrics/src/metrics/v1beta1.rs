use constcat::concat;

use super::*;

pub use node::NodeMetrics;
pub use pod::PodMetrics;

pub const METRICS_API_GROUP: &str = "metrics.k8s.io";
pub const METRICS_API_VERSION: &str = "v1beta1";
pub const METRICS_API_GROUP_VERSION: &str = concat!(METRICS_API_GROUP, "/", METRICS_API_VERSION);

mod duration;
mod node;
mod pod;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub cpu: resource::Quantity,
    pub memory: resource::Quantity,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub usage: Usage,
}

impl Usage {
    pub fn cpu(&self) -> Result<f64, QuantityParseError> {
        self.cpu.to_f64()
    }

    pub fn memory(&self) -> Result<i64, QuantityParseError> {
        self.memory.to_memory()
    }
}

impl Container {
    pub fn cpu(&self) -> Result<f64, QuantityParseError> {
        self.usage.cpu()
    }

    pub fn memory(&self) -> Result<i64, QuantityParseError> {
        self.usage.memory()
    }
}

#[cfg(test)]
mod tests;
