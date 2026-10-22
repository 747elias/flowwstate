use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub grounded: bool,
    pub scope: String,
    pub kind: String,
    pub builtin: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InboxItem {
    pub id: String,
    pub source: String,
    pub hash: String,
    pub size: u64,
    pub status: String,
    pub target: String,
    pub reason: String,
    pub question: String,
    pub excerpt: String,
    pub origin: String,
    pub method: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub modified_ns: u64,
    pub archive_depth: u32,
    pub unpack_directory: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SortingRule {
    pub id: String,
    pub label: String,
    pub pattern: String,
    pub extension: String,
    pub target: String,
    pub instruction: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SortingExample {
    pub id: String,
    pub filename: String,
    pub excerpt: String,
    pub target: String,
    pub hash: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Activity {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub target: String,
    pub hash: String,
    pub detail: String,
    pub item_id: String,
    pub created_at: u64,
    pub reverted: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionProposal {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub target: String,
    pub hash: String,
    pub message: String,
    pub agent: Option<Agent>,
    pub status: String,
    pub created_at: u64,
    pub chat_id: String,
}
