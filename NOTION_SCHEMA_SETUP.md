# Notion Database Schema Setup Guide

This guide will help you set up your Notion database with the correct schema for the Life Achievement Tree application.

## Expected Database Schema

Your Notion database should have the following properties:

### Required Properties

| Property Name | Type | Description |
|--------------|------|-------------|
| **Name** | Title | Primary field - Node name/title |

### Optional Properties

| Property Name | Type | Description |
|--------------|------|-------------|
| Description | Rich Text | Node description or summary |
| Why | Rich Text | Reason/motivation for pursuing this goal |
| Criteria | Rich Text | Success criteria for completion |
| Time Range | Rich Text | Estimated time to complete (e.g., "2-3 weeks") |
| Resources | Rich Text | JSON array: `[{"title":"Name","url":"...","type":"..."}]` |
| Parent | Relation | Link to parent node (hierarchical) |
| Depends on | Relation | Links to dependency nodes |
| Done | Checkbox | Mark as completed |
| Pinned | Checkbox | Featured/highlighted in hub view |
| Color | Select | Node color theme |
| Badge | Select | Special badge (milestone, boss-level) |
| Archived | Select | Archive reason (abgebrochen, pausiert, erledigt) |
| Due | Date | Due date for completion |

## Color Select Options

Create a Select property with these options:

- `purple` 🟣
- `blue` 🔵
- `green` 🟢
- `orange` 🟠
- `pink` 🌸
- `teal` 🔷

## Badge Select Options

Create a Select property with these options:

- `milestone` 🏅
- `boss-level` ⭐

## Archive Reason Select Options

Create a Select property with these options:

- `abgebrochen` (Broken/Abandoned)
- `pausiert` (Paused)
- `erledigt` (Completed/Done)

## Step-by-Step Setup

### 1. Create the Notion Database

1. Go to [Notion.so](https://www.notion.so)
2. Click "Add a page" and select "Database"
3. Choose "Table" view
4. Name it something like "Life Tree" or "Skill Tree"

### 2. Create Properties

The database will start with a "Name" property (Title type) - this is required, so keep it.

Add the following properties by clicking "+" in the property header:

#### Text Properties (Rich Text)
- Description
- Why
- Criteria
- Time Range
- Resources

#### Relation Properties
- Parent (single: "Many to one")
- Depends on (many: "Many to many")

**Note:** When creating relations, point both to the same database (the Life Tree database itself).

#### Checkbox Properties
- Done
- Pinned

#### Select Properties
- **Color** with options: purple, blue, green, orange, pink, teal
- **Badge** with options: milestone, boss-level
- **Archived** with options: abgebrochen, pausiert, erledigt

#### Date Property
- Due

### 3. Add Sample Data

Create a root node:
- **Name:** "My Life Tree"
- **Done:** Unchecked
- **Color:** (Choose one)
- **Description:** "Welcome to my life achievement tree!"

Create a child node:
- **Name:** "Learn Rust"
- **Parent:** [Link to "My Life Tree"]
- **Done:** Unchecked
- **Why:** "Build efficient backend systems"
- **Color:** `blue`
- **Time Range:** "3 months"

### 4. Get Your Database ID

Your Notion database URL looks like:
```
https://www.notion.so/[workspace-name]/[DATABASE_ID]?v=[view-id]
```

The DATABASE_ID is the long alphanumeric string. Copy it - you'll need it.

### 5. Get Your Notion API Key

1. Go to [Notion Integrations](https://www.notion.com/my-integrations)
2. Click "Create new integration"
3. Name it "Life Tree"
4. Copy the "Internal Integration Token" (starts with `secret_`)

### 6. Share Database with Integration

1. In Notion, open your Life Tree database
2. Click "Share" (top right)
3. Find your integration in the list and add it with "Editor" access

### 7. Validate Your Schema

Run the schema validator to check if everything is correct:

```bash
# Start the backend
cargo run --bin life-tree-backend

# In another terminal, validate the database
curl -X POST http://localhost:8080/api/schema/validate/[DATABASE_ID]

# Example:
curl -X POST http://localhost:8080/api/schema/validate/a1b2c3d4e5f6g7h8
```

Expected successful response:
```json
{
  "message": "Schema validation endpoint",
  "instructions": {
    "step_1": "Get your Notion API key...",
    ...
  }
}
```

## Troubleshooting

### Property Type Mismatches

If you see "Wrong type" errors:

**Problem:** Property created as "Text" but should be "Rich Text"
**Solution:** Delete the property and recreate it with the correct type

**Problem:** Relation created pointing to wrong database
**Solution:** Delete the relation and create a new one pointing to your Life Tree database

### Missing Relations

If "Parent" or "Depends on" are missing:

**Solution:** Create Relation properties:
1. Click "+" in property header
2. Select "Relation"
3. Set "Parent" → "One to many" → Point to this database
4. Set "Depends on" → "Many to many" → Point to this database

### Select Options Missing

If color/badge/archived options don't match:

**Solution:** Edit the Select property and ensure exact names:
- `purple`, `blue`, `green`, `orange`, `pink`, `teal` (lowercase)
- `milestone`, `boss-level` (lowercase with hyphen)
- `abgebrochen`, `pausiert`, `erledigt` (lowercase, German)

## Production Checklist

- [ ] Name property exists and is Title type
- [ ] All optional properties created with correct types
- [ ] Color select has all 6 options
- [ ] Badge select has both options
- [ ] Archived select has all 3 German options
- [ ] Parent relation points to same database
- [ ] Depends on relation points to same database
- [ ] Integration has "Editor" access to database
- [ ] Test data created (at least one root + one child)
- [ ] Schema validation passes

## Database Properties Quick Reference

```json
{
  "database_properties": [
    {
      "name": "Name",
      "type": "title"
    },
    {
      "name": "Description",
      "type": "rich_text"
    },
    {
      "name": "Why",
      "type": "rich_text"
    },
    {
      "name": "Criteria",
      "type": "rich_text"
    },
    {
      "name": "Time Range",
      "type": "rich_text"
    },
    {
      "name": "Resources",
      "type": "rich_text",
      "format": "json_array"
    },
    {
      "name": "Parent",
      "type": "relation",
      "targets": ["same_database"]
    },
    {
      "name": "Depends on",
      "type": "relation",
      "targets": ["same_database"]
    },
    {
      "name": "Done",
      "type": "checkbox"
    },
    {
      "name": "Pinned",
      "type": "checkbox"
    },
    {
      "name": "Color",
      "type": "select",
      "options": ["purple", "blue", "green", "orange", "pink", "teal"]
    },
    {
      "name": "Badge",
      "type": "select",
      "options": ["milestone", "boss-level"]
    },
    {
      "name": "Archived",
      "type": "select",
      "options": ["abgebrochen", "pausiert", "erledigt"]
    },
    {
      "name": "Due",
      "type": "date"
    }
  ]
}
```

## Next Steps

Once your database schema is validated:

1. Backend will implement full Notion CRUD operations
2. Frontend will fetch and display your nodes
3. You can start adding your life goals and skill tree!

---

**Need help?** Check the test examples in `crates/backend/src/services/notion_schema.rs` to see how schema validation works.
