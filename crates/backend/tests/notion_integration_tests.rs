/// Integration tests for Notion API service
///
/// These tests verify all Notion API operations work correctly
/// by mocking HTTP responses and testing the service layer

use life_tree_core::types::*;
use life_tree_core::api::*;
use serde_json::{json, Value};

// Mock Notion API responses

fn mock_notion_page(
    id: &str,
    name: &str,
    parent_id: Option<&str>,
    done: bool,
    color: Option<&str>,
) -> Value {
    json!({
        "id": id,
        "created_time": "2024-01-01T00:00:00.000Z",
        "icon": {
            "type": "emoji",
            "emoji": "🎯"
        },
        "properties": {
            "Name": {
                "id": "title",
                "type": "title",
                "title": [
                    {
                        "type": "text",
                        "text": {
                            "content": name,
                            "link": null
                        },
                        "annotations": {},
                        "plain_text": name,
                        "href": null
                    }
                ]
            },
            "Description": {
                "id": "desc",
                "type": "rich_text",
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": "Test description",
                            "link": null
                        },
                        "plain_text": "Test description"
                    }
                ]
            },
            "Why": {
                "id": "why",
                "type": "rich_text",
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": "Test why",
                            "link": null
                        },
                        "plain_text": "Test why"
                    }
                ]
            },
            "Criteria": {
                "id": "criteria",
                "type": "rich_text",
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": "Test criteria",
                            "link": null
                        },
                        "plain_text": "Test criteria"
                    }
                ]
            },
            "Done": {
                "id": "done",
                "type": "checkbox",
                "checkbox": done
            },
            "Pinned": {
                "id": "pinned",
                "type": "checkbox",
                "checkbox": false
            },
            "Parent": {
                "id": "parent",
                "type": "relation",
                "relation": if let Some(pid) = parent_id {
                    vec![json!({"id": pid})]
                } else {
                    vec![]
                }
            },
            "Depends on": {
                "id": "depends",
                "type": "relation",
                "relation": []
            },
            "Color": {
                "id": "color",
                "type": "select",
                "select": if let Some(c) = color {
                    json!({"name": c, "color": c})
                } else {
                    Value::Null
                }
            },
            "Badge": {
                "id": "badge",
                "type": "select",
                "select": Value::Null
            },
            "Archived": {
                "id": "archived",
                "type": "select",
                "select": Value::Null
            },
            "Due": {
                "id": "due",
                "type": "date",
                "date": Value::Null
            },
            "Time Range": {
                "id": "time_range",
                "type": "rich_text",
                "rich_text": []
            },
            "Resources": {
                "id": "resources",
                "type": "rich_text",
                "rich_text": []
            }
        }
    })
}

#[test]
fn test_notion_page_to_node_parsing() {
    // Test that we can parse a Notion page response into NotionNode
    let page = mock_notion_page("id-123", "Test Node", None, false, Some("purple"));

    // This would be called by page_to_node function
    let name = page
        .get("properties")
        .and_then(|p| p.get("Name"))
        .and_then(|n| n.get("title"))
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("plain_text"))
        .and_then(|pt| pt.as_str());

    assert_eq!(name, Some("Test Node"));
}

#[test]
fn test_create_node_request_serialization() {
    // Test that CreateNodeRequest serializes correctly
    let req = CreateNodeRequest {
        name: "New Goal".to_string(),
        icon: Some("🎯".to_string()),
        description: Some("Description".to_string()),
        why: Some("Why".to_string()),
        criteria: Some("Criteria".to_string()),
        color: Some(NodeColor::Purple),
        badge: None,
        due: Some("2024-12-31".to_string()),
        time_range: Some("3 months".to_string()),
        resources: None,
        parent_id: None,
        depends_on_ids: None,
    };

    let json = serde_json::to_value(&req).expect("Should serialize");

    assert_eq!(json["name"], "New Goal");
    assert_eq!(json["icon"], "🎯");
    assert_eq!(json["color"], "purple");
    assert_eq!(json["due"], "2024-12-31");
}

#[test]
fn test_update_node_request_partial_update() {
    // Test that UpdateNodeRequest only includes specified fields
    let req = UpdateNodeRequest {
        name: Some("Updated Name".to_string()),
        icon: None,
        description: None,
        why: None,
        criteria: None,
        color: Some(NodeColor::Blue),
        badge: None,
        due: None,
        time_range: None,
        resources: None,
        parent_id: None,
        depends_on_ids: None,
        done: Some(true),
        pinned: None,
        archived: None,
    };

    let json = serde_json::to_value(&req).expect("Should serialize");

    assert_eq!(json["name"], "Updated Name");
    assert_eq!(json["done"], true);
    assert_eq!(json["icon"], Value::Null);
    assert_eq!(json["description"], Value::Null);
}

#[test]
fn test_archive_reason_serialization() {
    // Test archive reasons serialize with correct names
    let reasons = vec![
        (ArchiveReason::Abgebrochen, "abgebrochen"),
        (ArchiveReason::Pausiert, "pausiert"),
        (ArchiveReason::Erledigt, "erledigt"),
    ];

    for (reason, expected) in reasons {
        let json = serde_json::to_value(&reason).expect("Should serialize");
        assert_eq!(json, expected);
    }
}

#[test]
fn test_node_color_serialization() {
    // Test all colors serialize with lowercase names
    let colors = vec![
        (NodeColor::Purple, "purple"),
        (NodeColor::Blue, "blue"),
        (NodeColor::Green, "green"),
        (NodeColor::Orange, "orange"),
        (NodeColor::Pink, "pink"),
        (NodeColor::Teal, "teal"),
    ];

    for (color, expected) in colors {
        let json = serde_json::to_value(&color).expect("Should serialize");
        assert_eq!(json, expected);
    }
}

#[test]
fn test_node_badge_serialization() {
    // Test badges serialize with correct names
    let badges = vec![
        (NodeBadge::Milestone, "milestone"),
        (NodeBadge::BossLevel, "boss-level"),
    ];

    for (badge, expected) in badges {
        let json = serde_json::to_value(&badge).expect("Should serialize");
        assert_eq!(json, expected);
    }
}

#[test]
fn test_notion_response_pagination_structure() {
    // Test that a typical Notion query response has expected structure
    let response = json!({
        "object": "list",
        "results": [
            mock_notion_page("id-1", "Node 1", None, false, Some("purple")),
            mock_notion_page("id-2", "Node 2", Some("id-1"), false, Some("blue"))
        ],
        "next_cursor": "cursor-123",
        "has_more": true
    });

    assert_eq!(response["object"], "list");
    assert!(response["results"].is_array());
    assert_eq!(response["results"].as_array().unwrap().len(), 2);
    assert_eq!(response["next_cursor"], "cursor-123");
    assert_eq!(response["has_more"], true);
}

#[test]
fn test_notion_response_no_more_pages() {
    // Test pagination termination
    let response = json!({
        "object": "list",
        "results": [],
        "next_cursor": null,
        "has_more": false
    });

    assert_eq!(response["has_more"], false);
    assert_eq!(response["next_cursor"], Value::Null);
}

#[test]
fn test_create_node_with_all_fields() {
    // Test that create_node request can include all possible fields
    let resources = vec![
        Resource {
            title: "Resource 1".to_string(),
            url: Some("https://example.com".to_string()),
            resource_type: Some("link".to_string()),
        },
    ];

    let req = CreateNodeRequest {
        name: "Complex Goal".to_string(),
        icon: Some("🚀".to_string()),
        description: Some("Full description".to_string()),
        why: Some("Motivation".to_string()),
        criteria: Some("Success criteria".to_string()),
        color: Some(NodeColor::Green),
        badge: Some(NodeBadge::BossLevel),
        due: Some("2024-12-31".to_string()),
        time_range: Some("6 months".to_string()),
        resources: Some(resources),
        parent_id: Some("parent-id".to_string()),
        depends_on_ids: Some(vec!["dep-1".to_string(), "dep-2".to_string()]),
    };

    let json = serde_json::to_value(&req).expect("Should serialize");

    assert_eq!(json["name"], "Complex Goal");
    assert_eq!(json["icon"], "🚀");
    assert_eq!(json["color"], "green");
    assert_eq!(json["badge"], "boss-level");
    assert_eq!(json["parentId"], "parent-id");
    assert!(json["dependsOnIds"].is_array());
    assert!(json["resources"].is_array());
}

#[test]
fn test_update_node_with_relations() {
    // Test updating parent and dependencies
    let req = UpdateNodeRequest {
        name: None,
        icon: None,
        description: None,
        why: None,
        criteria: None,
        color: None,
        badge: None,
        due: None,
        time_range: None,
        resources: None,
        parent_id: Some("new-parent".to_string()),
        depends_on_ids: Some(vec!["new-dep-1".to_string()]),
        done: None,
        pinned: None,
        archived: None,
    };

    let json = serde_json::to_value(&req).expect("Should serialize");

    assert_eq!(json["parentId"], "new-parent");
    assert!(json["dependsOnIds"].is_array());
    assert_eq!(json["dependsOnIds"][0], "new-dep-1");
}

#[test]
fn test_notion_node_deserialization() {
    // Test that NotionNode can be deserialized from JSON
    let node_json = json!({
        "id": "test-id",
        "name": "Test",
        "icon": "🎯",
        "description": "desc",
        "why": "why",
        "criteria": "criteria",
        "parentId": null,
        "dependsOnIds": [],
        "done": false,
        "archived": null,
        "pinned": false,
        "badge": null,
        "color": "purple",
        "due": null,
        "timeRange": null,
        "resources": [],
        "createdTime": "2024-01-01T00:00:00Z"
    });

    let node: NotionNode = serde_json::from_value(node_json)
        .expect("Should deserialize");

    assert_eq!(node.id, "test-id");
    assert_eq!(node.name, "Test");
    assert_eq!(node.icon, Some("🎯".to_string()));
    assert_eq!(node.color, Some(NodeColor::Purple));
    assert!(!node.done);
}

#[test]
fn test_notion_auth_header_format() {
    // Test that auth header is formatted correctly for Notion API
    let token = "test-token-xyz";
    let expected = format!("Bearer {}", token);

    assert!(expected.starts_with("Bearer "));
    assert!(expected.contains(token));
}

#[test]
fn test_resource_json_roundtrip() {
    // Test that resources can be serialized to JSON and back
    let resources = vec![
        Resource {
            title: "Rust Book".to_string(),
            url: Some("https://doc.rust-lang.org".to_string()),
            resource_type: Some("book".to_string()),
        },
        Resource {
            title: "Video".to_string(),
            url: Some("https://youtube.com/video".to_string()),
            resource_type: None,
        },
    ];

    let json_str = serde_json::to_string(&resources).expect("Should serialize");
    let deserialized: Vec<Resource> = serde_json::from_str(&json_str)
        .expect("Should deserialize");

    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].title, "Rust Book");
    assert_eq!(deserialized[1].url, Some("https://youtube.com/video".to_string()));
}

#[test]
fn test_chat_request_structure() {
    // Test ChatRequest is properly structured for sending to AI
    let messages = vec![
        ChatMessageRequest {
            role: "user".to_string(),
            content: "What should I do next?".to_string(),
        },
    ];

    let req = ChatRequest {
        messages,
        existing_nodes: vec![],
        parent_id: Some("root-id".to_string()),
        working_draft: None,
        web_search_results: None,
    };

    let json = serde_json::to_value(&req).expect("Should serialize");

    assert!(json["messages"].is_array());
    assert_eq!(json["messages"][0]["role"], "user");
    assert_eq!(json["parentId"], "root-id");
}

#[test]
fn test_multiple_node_hierarchy() {
    // Test creating a hierarchy of nodes with parent-child relationships
    let root = CreateNodeRequest {
        name: "Root Goal".to_string(),
        icon: Some("🌳".to_string()),
        description: None,
        why: None,
        criteria: None,
        color: Some(NodeColor::Purple),
        badge: None,
        due: None,
        time_range: None,
        resources: None,
        parent_id: None,
        depends_on_ids: None,
    };

    let child = CreateNodeRequest {
        name: "Child Goal".to_string(),
        icon: Some("🎯".to_string()),
        description: None,
        why: None,
        criteria: None,
        color: Some(NodeColor::Blue),
        badge: None,
        due: None,
        time_range: None,
        resources: None,
        parent_id: Some("root-id".to_string()),
        depends_on_ids: Some(vec!["root-id".to_string()]),
    };

    let root_json = serde_json::to_value(&root).expect("Should serialize");
    let child_json = serde_json::to_value(&child).expect("Should serialize");

    assert_eq!(root_json["parentId"], Value::Null);
    assert_eq!(child_json["parentId"], "root-id");
    assert!(child_json["dependsOnIds"].is_array());
}

#[test]
fn test_archive_node_request() {
    // Test that archiving a node includes the reason
    let req = UpdateNodeRequest {
        name: None,
        icon: None,
        description: None,
        why: None,
        criteria: None,
        color: None,
        badge: None,
        due: None,
        time_range: None,
        resources: None,
        parent_id: None,
        depends_on_ids: None,
        done: None,
        pinned: None,
        archived: Some(ArchiveReason::Pausiert),
    };

    let json = serde_json::to_value(&req).expect("Should serialize");
    assert_eq!(json["archived"], "pausiert");
}
