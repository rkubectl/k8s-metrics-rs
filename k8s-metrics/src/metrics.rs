use std::time;

use super::*;

k8s::k8s_if_ge_1_37! {
    /// Stable resource metrics API, compiled for Kubernetes 1.37 and later.
    pub mod v1;
}
pub mod v1beta1;

mod duration;

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
