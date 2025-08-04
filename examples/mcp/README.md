# Flame MCP Service

This is a Model Context Protocol (MCP) service that enables remote execution of scripts via Flame at scale.

## Overview

The Flame MCP Service provides a secure interface for running scripts through the Flame execution platform. It's designed to work with AI agents and provides session-based isolation for security.

## Tools

### run_script

The `run_script` tool executes scripts via Flame. For security, it's recommended to deploy the Flame executor manager in a sandbox environment. Each agent session gets a dedicated Flame session to run all scripts, ensuring execution environments are not reused across sessions for proper isolation.

#### Parameters

The tool accepts the following parameters:

* **session_id**: The ID of the agent session with the user. The MCP maps this to a Flame session for secure script execution.
* **language**: The programming language of the script. Currently supports `python` and `shell`.
* **code**: The script code to execute. Must be UTF-8 encoded.

#### Response

The tool returns only the script output.

## Configuration

### Environment Variables

- `FLAME_ENDPOINT`: Flame service address (default: "http://127.0.0.1:8080")
- `FLAME_MCP_PORT`: MCP server port. If not set, stdio transport will be used

## Getting Started

### Building

```bash
cd examples/mcp
go mod tidy
go build -o flame-mcp-server
```

### Running

```bash
# Use stdio transport for MCP
./flame-mcp-server

# Use HTTP transport for MCP
FLAME_MCP_PORT=9090 ./flame-mcp-server

# With custom Flame address
FLAME_ENDPOINT=http://localhost:9090 ./flame-mcp-server
```

## Dependencies

- **Flame Go SDK**: For connecting to and interacting with Flame services
- **MCP Go SDK**: For MCP server implementation, JSON handling, and context management

## Error Handling

The service handles various error scenarios gracefully:

- Invalid request parameters
- Flame connection failures
- Session creation failures
- Task execution timeouts
- Script execution errors

All errors are returned in a consistent JSON format with descriptive error messages. 