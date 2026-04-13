use crate::types::*;
use serde::{Deserialize, Serialize};

/// Request to create a new node
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
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
}

/// Request to update an existing node
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
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
    pub done: Option<bool>,
    pub pinned: Option<bool>,
    pub archived: Option<ArchiveReason>,
}

/// Request to suggest node fields
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestFieldsRequest {
    pub name: String,
    pub parent_name: Option<String>,
    pub sibling_names: Option<Vec<String>>,
}

/// Response with suggested fields
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestFieldsResponse {
    pub icon: Option<String>,
    pub color: Option<NodeColor>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub time_range: Option<String>,
    pub badge: Option<NodeBadge>,
}

/// Chat message in request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRequest {
    pub role: String,
    pub content: String,
}

/// Request for chat with AI
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub messages: Vec<ChatMessageRequest>,
    pub existing_nodes: Vec<NotionNode>,
    pub parent_id: Option<String>,
    pub working_draft: Option<serde_json::Value>,
    pub web_search_results: Option<serde_json::Value>,
}

/// SSE event from chat endpoint
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    Text { content: String },
    ToolText { content: String },
    Done {
        suggested_nodes: Option<Vec<SuggestedNode>>,
        suggested_parent_id: Option<String>,
        node_edits: Option<Vec<NodeEdit>>,
        truncated: bool,
    },
    Error { message: String },
}

/// Web search request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchRequest {
    pub query: String,
}

/// Web search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

/// Web search response
#[derive(Debug, Serialize)]
pub struct WebSearchResponse {
    pub answer: Option<String>,
    pub results: Vec<SearchResult>,
}

/// Oura data response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OuraDataResponse {
    #[serde(flatten)]
    pub data: Option<OuraData>,
    pub connected: Option<bool>,
}
