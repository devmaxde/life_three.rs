use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::services::notion_schema::{SchemaValidation, ValidationStatus};

/// Request to validate a Notion database schema
#[derive(Debug, Deserialize)]
pub struct ValidateSchemaRequest {
    pub database_id: String,
    pub api_key: String,
}

/// Response with schema validation results
#[derive(Debug, Serialize)]
pub struct ValidateSchemaResponse {
    pub validation: SchemaValidation,
    pub migration_guide: Option<MigrationGuide>,
}

/// Migration guide for fixing schema issues
#[derive(Debug, Serialize)]
pub struct MigrationGuide {
    pub summary: String,
    pub steps: Vec<MigrationStep>,
}

#[derive(Debug, Serialize)]
pub struct MigrationStep {
    pub action: String,
    pub field_name: String,
    pub field_type: String,
    pub description: String,
}

/// Mock validation endpoint (in real implementation, this would fetch from Notion API)
pub async fn validate_database_schema(
    Path(_database_id): Path<String>,
) -> impl IntoResponse {
    // This is a mock implementation. In production, you would:
    // 1. Fetch the database schema from Notion API using the database_id
    // 2. Call validate_schema with the actual properties

    // For now, return a helpful error message
    (
        StatusCode::OK,
        Json(json!({
            "message": "Schema validation endpoint",
            "instructions": {
                "step_1": "Get your Notion API key from https://www.notion.com/my-integrations",
                "step_2": "Create a POST request to /api/schema/validate",
                "step_3": "Include database_id and api_key in the request body",
                "example": {
                    "database_id": "your-database-id",
                    "api_key": "secret_xxxxx"
                },
                "note": "Database ID can be found in the Notion URL: notion.so/{database_id}?v=..."
            }
        })),
    )
}

/// Generate migration guide based on validation results
pub fn generate_migration_guide(validation: &SchemaValidation) -> Option<MigrationGuide> {
    if validation.summary.is_valid {
        return None;
    }

    let mut steps = Vec::new();

    // Add steps for missing required properties
    for prop in validation.properties.iter() {
        if prop.actual_type.is_none() && prop.name == "Name" {
            steps.push(MigrationStep {
                action: "CREATE".to_string(),
                field_name: "Name".to_string(),
                field_type: "title".to_string(),
                description: "Create a Title property called 'Name' as the primary field".to_string(),
            });
        }
    }

    // Add steps for wrong types
    for prop in validation.properties.iter() {
        if prop.expected_type != prop.actual_type.as_ref().unwrap_or(&String::new()).as_str() {
            steps.push(MigrationStep {
                action: "CONVERT".to_string(),
                field_name: prop.name.clone(),
                field_type: prop.expected_type.clone(),
                description: format!(
                    "Change property type from {} to {}",
                    prop.actual_type.as_ref().unwrap_or(&"unknown".to_string()),
                    prop.expected_type
                ),
            });
        }
    }

    // Add steps for missing optional properties
    for prop in validation.properties.iter() {
        if prop.actual_type.is_none() && prop.status == crate::services::notion_schema::ValidationStatus::Warning {
            steps.push(MigrationStep {
                action: "CREATE".to_string(),
                field_name: prop.name.clone(),
                field_type: prop.expected_type.clone(),
                description: format!(
                    "Create optional property '{}' of type {}",
                    prop.name, prop.expected_type
                ),
            });
        }
    }

    if steps.is_empty() {
        return None;
    }

    Some(MigrationGuide {
        summary: format!(
            "Found {} issues in schema. {} required fields missing, {} fields with wrong type.",
            validation.summary.missing + validation.summary.wrong_type,
            validation.summary.missing,
            validation.summary.wrong_type
        ),
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::notion_schema::{PropertyValidation, ValidationStatus, ValidationSummary};

    #[test]
    fn test_generate_migration_guide_valid_schema() {
        let validation = SchemaValidation {
            database_id: "db-123".to_string(),
            database_name: "Test".to_string(),
            properties: vec![],
            summary: ValidationSummary {
                total_expected: 14,
                total_found: 14,
                valid: 14,
                missing: 0,
                wrong_type: 0,
                warnings: 0,
                is_valid: true,
            },
        };

        let guide = generate_migration_guide(&validation);
        assert!(guide.is_none());
    }

    #[test]
    fn test_generate_migration_guide_invalid_schema() {
        let validation = SchemaValidation {
            database_id: "db-123".to_string(),
            database_name: "Test".to_string(),
            properties: vec![
                PropertyValidation {
                    name: "Name".to_string(),
                    expected_type: "title".to_string(),
                    actual_type: None,
                    status: ValidationStatus::Missing,
                    message: "Missing".to_string(),
                },
            ],
            summary: ValidationSummary {
                total_expected: 14,
                total_found: 0,
                valid: 0,
                missing: 1,
                wrong_type: 0,
                warnings: 0,
                is_valid: false,
            },
        };

        let guide = generate_migration_guide(&validation);
        assert!(guide.is_some());
        let guide = guide.unwrap();
        assert!(!guide.steps.is_empty());
    }
}
