use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Archive reason for completed or paused nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveReason {
    Abgebrochen,
    Pausiert,
    Erledigt,
}

/// Node badge for special status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeBadge {
    Milestone,
    BossLevel,
}

/// Node color for visual distinction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeColor {
    Purple,
    Blue,
    Green,
    Orange,
    Pink,
    Teal,
}

/// Node type based on tree structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Root,
    Container,
    Leaf,
}

/// Node status based on progress and dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Locked,
    Active,
    Completed,
    Archived,
}

/// Bedtime regularity metric from Oura
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BedtimeRegularity {
    Good,
    Medium,
    Poor,
}

/// Toast notification type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToastType {
    Success,
    Error,
    Info,
}

/// Chat message role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

/// External resource linked to a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub title: String,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
}

/// Position in 2D space (x, y coordinates)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Node data as stored in Notion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotionNode {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    pub why: String,
    pub criteria: String,
    pub parent_id: Option<String>,
    pub depends_on_ids: Vec<String>,
    pub done: bool,
    pub archived: Option<ArchiveReason>,
    pub pinned: bool,
    pub badge: Option<NodeBadge>,
    pub color: Option<NodeColor>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Vec<Resource>,
    pub created_time: String,
}

/// Computed node with additional algorithmic fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedNode {
    #[serde(flatten)]
    pub base: NotionNode,

    pub node_type: NodeType,
    pub status: NodeStatus,
    pub progress: f64,
    pub depth: u32,
    pub children: Vec<ComputedNode>,
    pub dependents: Vec<ComputedNode>,
    pub position: Position,
    pub is_cycle_member: bool,
}

impl ComputedNode {
    pub fn id(&self) -> &str {
        &self.base.id
    }

    pub fn name(&self) -> &str {
        &self.base.name
    }

    pub fn parent_id(&self) -> Option<&str> {
        self.base.parent_id.as_deref()
    }

    pub fn depends_on_ids(&self) -> &[String] {
        &self.base.depends_on_ids
    }

    pub fn done(&self) -> bool {
        self.base.done
    }

    pub fn archived(&self) -> Option<&ArchiveReason> {
        self.base.archived.as_ref()
    }
}

/// Node for radial map layout
#[derive(Debug, Clone)]
pub struct MapNode {
    pub node: ComputedNode,
    pub x: f64,
    pub y: f64,
    pub ring: u32,
    pub sector_id: String,
    pub sector_color: String,
    pub sector_angle_start: f64,
    pub sector_angle_end: f64,
    pub sector_center: Position,
}

/// Oura sleep and readiness data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OuraData {
    pub readiness: u8,
    pub sleep_score: u8,
    pub sleep_duration: f64,
    pub bedtime_regularity: BedtimeRegularity,
    pub last_updated: String,
}

/// Oura authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OuraTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub acquired_at: u64,
}

impl OuraTokens {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.acquired_at + self.expires_in - 60
    }
}

/// Toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: String,
    pub message: String,
    pub toast_type: ToastType,
    pub duration: Option<u64>,
}

/// Chat message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
    pub suggested_nodes: Option<Vec<SuggestedNode>>,
    pub pending_edits: Option<Vec<NodeEdit>>,
    pub edits_approved: Option<bool>,
    pub duration_ms: Option<u64>,
}

/// Node suggested by AI assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedNode {
    pub temp_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub criteria: Option<String>,
    pub color: Option<NodeColor>,
    pub badge: Option<NodeBadge>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Option<Vec<Resource>>,
    pub parent_id: Option<String>,
    pub depends_on_ids: Option<Vec<String>>,
    pub children: Option<Vec<SuggestedNode>>,
}

/// Node edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEdit {
    pub node_id: String,
    pub node_name: Option<String>,
    pub updates: serde_json::Value,
}

/// Layout position overrides
pub type LayoutOverrides = HashMap<String, Position>;
