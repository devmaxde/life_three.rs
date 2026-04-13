use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expected property name and type in the Notion database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedProperty {
    pub name: String,
    pub property_type: PropertyType,
    pub required: bool,
    pub description: String,
}

/// Notion property types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Title,
    RichText,
    Relation,
    Select,
    MultiSelect,
    Checkbox,
    Date,
    Number,
    Email,
    Url,
    Formula,
    Rollup,
}

/// Actual property from Notion database
#[derive(Debug, Clone, Deserialize)]
pub struct NotionProperty {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(default)]
    pub select: Option<NotionSelect>,
    #[serde(default)]
    pub relation: Option<NotionRelation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionSelect {
    pub options: Vec<NotionSelectOption>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionSelectOption {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionRelation {
    pub database_id: String,
}

/// Validation result for a property
#[derive(Debug, Clone, Serialize)]
pub struct PropertyValidation {
    pub name: String,
    pub expected_type: String,
    pub actual_type: Option<String>,
    pub status: ValidationStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Ok,
    Missing,
    WrongType,
    Warning,
}

/// Overall schema validation result
#[derive(Debug, Clone, Serialize)]
pub struct SchemaValidation {
    pub database_id: String,
    pub database_name: String,
    pub properties: Vec<PropertyValidation>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationSummary {
    pub total_expected: usize,
    pub total_found: usize,
    pub valid: usize,
    pub missing: usize,
    pub wrong_type: usize,
    pub warnings: usize,
    pub is_valid: bool,
}

/// Get expected schema for life-tree database
pub fn get_expected_schema() -> Vec<ExpectedProperty> {
    vec![
        ExpectedProperty {
            name: "Name".to_string(),
            property_type: PropertyType::Title,
            required: true,
            description: "Node name/title (primary key)".to_string(),
        },
        ExpectedProperty {
            name: "Description".to_string(),
            property_type: PropertyType::RichText,
            required: false,
            description: "Node description".to_string(),
        },
        ExpectedProperty {
            name: "Why".to_string(),
            property_type: PropertyType::RichText,
            required: false,
            description: "Reason/motivation for the node".to_string(),
        },
        ExpectedProperty {
            name: "Criteria".to_string(),
            property_type: PropertyType::RichText,
            required: false,
            description: "Success criteria for completion".to_string(),
        },
        ExpectedProperty {
            name: "Time Range".to_string(),
            property_type: PropertyType::RichText,
            required: false,
            description: "Estimated time to complete".to_string(),
        },
        ExpectedProperty {
            name: "Resources".to_string(),
            property_type: PropertyType::RichText,
            required: false,
            description: "JSON array of resources: {title, url?, type?}".to_string(),
        },
        ExpectedProperty {
            name: "Parent".to_string(),
            property_type: PropertyType::Relation,
            required: false,
            description: "Parent node (single relation)".to_string(),
        },
        ExpectedProperty {
            name: "Depends on".to_string(),
            property_type: PropertyType::Relation,
            required: false,
            description: "Dependency nodes (multi relation)".to_string(),
        },
        ExpectedProperty {
            name: "Done".to_string(),
            property_type: PropertyType::Checkbox,
            required: false,
            description: "Completion flag".to_string(),
        },
        ExpectedProperty {
            name: "Pinned".to_string(),
            property_type: PropertyType::Checkbox,
            required: false,
            description: "Pinned/featured flag".to_string(),
        },
        ExpectedProperty {
            name: "Color".to_string(),
            property_type: PropertyType::Select,
            required: false,
            description: "Node color: purple, blue, green, orange, pink, teal".to_string(),
        },
        ExpectedProperty {
            name: "Badge".to_string(),
            property_type: PropertyType::Select,
            required: false,
            description: "Special badge: milestone, boss-level".to_string(),
        },
        ExpectedProperty {
            name: "Archived".to_string(),
            property_type: PropertyType::Select,
            required: false,
            description: "Archive reason: abgebrochen, pausiert, erledigt".to_string(),
        },
        ExpectedProperty {
            name: "Due".to_string(),
            property_type: PropertyType::Date,
            required: false,
            description: "Due date".to_string(),
        },
    ]
}

/// Validate Notion database schema against expected schema
pub fn validate_schema(
    database_name: String,
    database_id: String,
    actual_properties: Vec<NotionProperty>,
) -> SchemaValidation {
    let expected = get_expected_schema();
    let expected_map: HashMap<String, &ExpectedProperty> = expected
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    let actual_map: HashMap<String, &NotionProperty> = actual_properties
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    let mut validations = Vec::new();
    let mut valid_count = 0;
    let mut missing_count = 0;
    let mut wrong_type_count = 0;

    // Check each expected property
    for expected_prop in &expected {
        if let Some(actual_prop) = actual_map.get(&expected_prop.name) {
            let actual_type = normalize_property_type(&actual_prop.property_type);
            let expected_type = property_type_to_string(&expected_prop.property_type);

            if actual_type == expected_type {
                validations.push(PropertyValidation {
                    name: expected_prop.name.clone(),
                    expected_type: expected_type.clone(),
                    actual_type: Some(actual_type.clone()),
                    status: ValidationStatus::Ok,
                    message: "✓ Property found and correct type".to_string(),
                });
                valid_count += 1;
            } else {
                validations.push(PropertyValidation {
                    name: expected_prop.name.clone(),
                    expected_type: expected_type.clone(),
                    actual_type: Some(actual_type.clone()),
                    status: ValidationStatus::WrongType,
                    message: format!(
                        "✗ Wrong type. Expected {}, found {}",
                        expected_type, actual_type
                    ),
                });
                wrong_type_count += 1;
            }
        } else {
            validations.push(PropertyValidation {
                name: expected_prop.name.clone(),
                expected_type: property_type_to_string(&expected_prop.property_type),
                actual_type: None,
                status: if expected_prop.required {
                    ValidationStatus::Missing
                } else {
                    ValidationStatus::Warning
                },
                message: if expected_prop.required {
                    "✗ REQUIRED property missing".to_string()
                } else {
                    "⚠ Optional property missing".to_string()
                },
            });

            if expected_prop.required {
                missing_count += 1;
            }
        }
    }

    let is_valid = missing_count == 0 && wrong_type_count == 0;

    let summary = ValidationSummary {
        total_expected: expected.len(),
        total_found: actual_map.len(),
        valid: valid_count,
        missing: missing_count,
        wrong_type: wrong_type_count,
        warnings: validations
            .iter()
            .filter(|v| v.status == ValidationStatus::Warning)
            .count(),
        is_valid,
    };

    SchemaValidation {
        database_id,
        database_name,
        properties: validations,
        summary,
    }
}

fn normalize_property_type(notion_type: &str) -> String {
    match notion_type {
        "title" => "title".to_string(),
        "rich_text" => "rich_text".to_string(),
        "relation" => "relation".to_string(),
        "select" => "select".to_string(),
        "multi_select" => "multi_select".to_string(),
        "checkbox" => "checkbox".to_string(),
        "date" => "date".to_string(),
        "number" => "number".to_string(),
        "email" => "email".to_string(),
        "url" => "url".to_string(),
        t => t.to_string(),
    }
}

fn property_type_to_string(pt: &PropertyType) -> String {
    match pt {
        PropertyType::Title => "title".to_string(),
        PropertyType::RichText => "rich_text".to_string(),
        PropertyType::Relation => "relation".to_string(),
        PropertyType::Select => "select".to_string(),
        PropertyType::MultiSelect => "multi_select".to_string(),
        PropertyType::Checkbox => "checkbox".to_string(),
        PropertyType::Date => "date".to_string(),
        PropertyType::Number => "number".to_string(),
        PropertyType::Email => "email".to_string(),
        PropertyType::Url => "url".to_string(),
        PropertyType::Formula => "formula".to_string(),
        PropertyType::Rollup => "rollup".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_schema_has_required_fields() {
        let schema = get_expected_schema();
        let names: Vec<&str> = schema.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"Name"));
        assert!(names.contains(&"Parent"));
        assert!(names.contains(&"Depends on"));
        assert!(names.contains(&"Done"));
        assert!(names.contains(&"Color"));
    }

    #[test]
    fn test_validation_empty_database() {
        let validation = validate_schema(
            "Test DB".to_string(),
            "db-123".to_string(),
            vec![],
        );

        assert!(!validation.summary.is_valid);
        assert_eq!(validation.summary.missing, 1); // Name is required
        assert!(validation.summary.warnings > 0); // Many optional fields missing
    }

    #[test]
    fn test_validation_correct_property() {
        let props = vec![
            NotionProperty {
                id: "title-id".to_string(),
                name: "Name".to_string(),
                property_type: "title".to_string(),
                select: None,
                relation: None,
            },
        ];

        let validation = validate_schema(
            "Test DB".to_string(),
            "db-123".to_string(),
            props,
        );

        let name_validation = validation
            .properties
            .iter()
            .find(|p| p.name == "Name")
            .unwrap();

        assert_eq!(name_validation.status, ValidationStatus::Ok);
    }

    #[test]
    fn test_validation_wrong_type() {
        let props = vec![
            NotionProperty {
                id: "name-id".to_string(),
                name: "Name".to_string(),
                property_type: "rich_text".to_string(), // Wrong! Should be title
                select: None,
                relation: None,
            },
        ];

        let validation = validate_schema(
            "Test DB".to_string(),
            "db-123".to_string(),
            props,
        );

        let name_validation = validation
            .properties
            .iter()
            .find(|p| p.name == "Name")
            .unwrap();

        assert_eq!(name_validation.status, ValidationStatus::WrongType);
    }
}
