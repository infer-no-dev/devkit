//! Process isolation levels and namespaces

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    Process,
    Container,
    VirtualMachine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Namespace {
    Process,
    Network,
    Mount,
    User,
    Ipc,
    Uts,
}