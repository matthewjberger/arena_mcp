<h1 align="center">Arena MCP</h1>

<p align="center">
  <a href="https://github.com/matthewjberger/arena_mcp"><img alt="github" src="https://img.shields.io/badge/github-matthewjberger/arena__mcp-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://github.com/matthewjberger/arena_mcp/blob/main/LICENSE-MIT"><img alt="license" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=for-the-badge&labelColor=555555" height="20"></a>
</p>

<p align="center"><strong>MCP server and desktop app for Arena PLM.</strong></p>

Arena MCP is an [MCP](https://modelcontextprotocol.io/) server that connects AI assistants to the [Arena PLM](https://www.arenasolutions.com/) REST API. Search items, explore BOMs, manage change orders, track quality processes, and more, all through structured tool calls. Includes a standalone desktop app with an embedded AI chat interface.

<img width="802" height="632" alt="Screenshot 2026-03-03 235425" src="https://github.com/user-attachments/assets/d90aeb77-c06e-44cd-a442-6b58de8808a2" />
<img width="802" height="632" alt="Screenshot 2026-03-03 235444" src="https://github.com/user-attachments/assets/5b1c97d4-e2a5-4521-b25d-a82952d7cf76" />

## Installation

```bash
cargo install --path .
```

## Quick Start

Add arena as an MCP server in your client configuration (Claude Desktop, Claude Code, etc.):

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

## Desktop App

The `app/` directory contains a standalone desktop application for interacting with Arena PLM. The MCP server is compiled directly into the app binary, so no separate installation is needed. You can also use the MCP server standalone with any MCP client (Claude Code, Claude Desktop, etc.).

### Features

- **Login screen**: Enter Arena credentials directly in the app (no environment variables needed)
- **AI chat**: Embedded Claude assistant with access to all Arena MCP tools
- **Items browser**: Search and browse items in a sortable table, view item details, files, and revisions
- **BOM tree**: Select an item and explore its Bill of Materials as a lazy-loading tree
- **Changes browser**: Search change orders and view affected items
- **File downloads**: Download files attached to items directly from the browser
- **Read-only mode**: Write operations (create, update, delete) are blocked by default; enable in Settings
- **Saved searches**: Save and recall search queries (stored in browser localStorage)
- **Logs viewer**: View the current session's log in-app, refresh on demand, or open an external log file for debugging

### Running

```
cd app
just run
```

### Logs

Each run creates a timestamped log file in `app/site/logs/` (e.g., `arena_plm_2026-03-04_14-30-00.log`). Logs include login events, Arena API calls, Claude CLI interactions, tool use, and errors. GPU-level noise (wgpu, naga) is filtered out so logs stay readable.

### Architecture

- **Backend** (`app/src/main.rs`): Nightshade desktop host with egui + webview + Claude CLI integration
- **Frontend** (`app/site/`): Leptos 0.8 CSR compiled to WASM, embedded in webview
- **Protocol** (`app/protocol/`): `#![no_std]` shared types for frontend-backend IPC
- **Arena API proxy**: Backend spawns a worker thread for direct Arena API access (browse views bypass Claude)
- **Claude CLI worker**: Spawned after login with credentials passed via environment variables to the MCP server
- **Embedded MCP server**: The arena MCP server is compiled into the binary and launched via `--mcp` flag when Claude needs it (no separate binary required)

## License

Dual-licensed under MIT ([LICENSE-MIT](LICENSE-MIT)) or Apache 2.0 ([LICENSE-APACHE](LICENSE-APACHE)).
