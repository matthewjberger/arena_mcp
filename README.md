# arena

MCP server for the Arena PLM REST API.

## Installation

```bash
cargo install --path .
```

## Quick Start

Add arena as an MCP server in your client configuration:

```json
{
  "mcpServers": {
    "arena": {
      "command": "arena",
      "env": {
        "ARENA_EMAIL": "you@company.com",
        "ARENA_PASSWORD": "your-password",
        "ARENA_WORKSPACE_ID": "123456789"
      }
    }
  }
}
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ARENA_EMAIL` | Yes | Arena account email |
| `ARENA_PASSWORD` | Yes | Arena account password |
| `ARENA_WORKSPACE_ID` | No | Workspace ID (required if account has multiple workspaces) |
| `ARENA_BASE_URL` | No | API base URL (default: `https://api.arenasolutions.com/v1`) |

## MCP Tools

### Items

| Tool | Description |
|------|-------------|
| `search_items` | Search items by name, number, description, category, lifecycle phase |
| `get_item` | Get a single item by GUID with full detail |
| `create_item` | Create a new item |
| `update_item` | Update an existing item |
| `delete_item` | Delete an item |
| `get_item_sourcing` | Get sourcing entries for an item |
| `get_item_compliance` | Get compliance requirements for an item |
| `get_item_references` | Get references for an item |
| `get_item_quality` | Get quality processes associated with an item |
| `item_lifecycle_phase_change` | Change an item's lifecycle phase |

### BOMs

| Tool | Description |
|------|-------------|
| `get_bom` | Get the Bill of Materials for an item |
| `get_where_used` | Get parent assemblies that contain an item |
| `create_bom_line` | Add a line to an item's BOM |
| `update_bom_line` | Update a BOM line |
| `delete_bom_line` | Remove a BOM line |

### Changes

| Tool | Description |
|------|-------------|
| `search_changes` | Search change orders by number, title, status |
| `get_change` | Get a single change order by GUID |
| `get_change_affected_items` | Get items affected by a change |
| `create_change` | Create a new change order |
| `update_change` | Update an existing change order |
| `change_change_status` | Change the status of a change order |
| `add_change_affected_item` | Add an item to a change's affected items |
| `remove_change_affected_item` | Remove an item from a change's affected items |
| `get_change_files` | Get files associated with a change |
| `get_change_implementation_statuses` | Get implementation statuses for a change |

### Revisions & Files

| Tool | Description |
|------|-------------|
| `get_item_revisions` | Get revision history for an item |
| `get_item_files` | Get files associated with an item |
| `get_item_file_content` | Get the content of an item's file |
| `search_files` | Search for files |
| `get_file` | Get a single file by GUID |

### Requests

| Tool | Description |
|------|-------------|
| `search_requests` | Search change requests by number, title, status |
| `get_request` | Get a single request by GUID |
| `create_request` | Create a new change request |
| `update_request` | Update an existing request |
| `change_request_status` | Change the status of a request |
| `get_request_items` | Get items associated with a request |

### Suppliers

| Tool | Description |
|------|-------------|
| `search_suppliers` | Search suppliers by name |
| `get_supplier` | Get a single supplier by GUID |
| `create_supplier` | Create a new supplier |
| `update_supplier` | Update an existing supplier |
| `search_supplier_items` | Search supplier items |

### Quality

| Tool | Description |
|------|-------------|
| `search_quality_processes` | Search quality processes by number, name, status |
| `get_quality_process` | Get a single quality process by GUID |
| `get_quality_process_steps` | Get steps for a quality process |
| `change_quality_status` | Change the status of a quality process |

### Tickets

| Tool | Description |
|------|-------------|
| `search_tickets` | Search tickets |
| `get_ticket` | Get a single ticket by GUID |
| `create_ticket` | Create a new ticket |

### Training

| Tool | Description |
|------|-------------|
| `search_training_plans` | Search training plans |
| `get_training_plan` | Get a single training plan by GUID |
| `get_training_plan_records` | Get records for a training plan |

### Settings

| Tool | Description |
|------|-------------|
| `get_lifecycle_phases` | Get all lifecycle phases |
| `get_item_categories` | Get all item categories |
| `get_change_categories` | Get all change categories |
| `get_item_number_formats` | Get item number formats |

## Authentication

Arena uses session-based authentication. The server automatically:

1. Logs in with email/password on first API call
2. Caches the session token
3. Re-authenticates on 401 responses
4. Logs out when the server shuts down

## Frontend App

The `app/` directory contains a web frontend for browsing Arena data. See `app/README.md` for details.

## License

MIT
