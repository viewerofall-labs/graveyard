use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    ListServices,
    StartService(String),
    StopService(String),
    RestartService(String),
    GetStatus(String),
    GetDetailedStatus(String),
    EnableService(String),
    DisableService(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    ServiceList {
        services: Vec<ServiceInfo>,
        groups: Vec<Group>,
    },
    ServiceStatus(ServiceInfo),
    DetailedStatus(String),
    Success(String),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub services: Vec<String>, // service names that belong to this group
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Active => write!(f, "active"),
            ServiceStatus::Inactive => write!(f, "inactive"),
            ServiceStatus::Failed => write!(f, "failed"),
            ServiceStatus::Unknown => write!(f, "unknown"),
        }
    }
}
