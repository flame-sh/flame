/*
Copyright 2025 The Flame Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package main

import (
	"context"
	"fmt"
	"log"
	"os/exec"
	"testing"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

func TestListTools(t *testing.T) {
	ctx := context.Background()

	// Create a new client, with no features.
	client := mcp.NewClient(&mcp.Implementation{Name: "flame-mcp-test", Version: "v1.0.0"}, nil)

	// Connect to a server over stdin/stdout
	transport := mcp.NewCommandTransport(exec.Command("go", "run", "main.go", "server.go", "types.go"))
	session, err := client.Connect(ctx, transport)
	if err != nil {
		log.Fatal(err)
	}
	defer session.Close()

	tools, err := session.ListTools(ctx, &mcp.ListToolsParams{})
	if err != nil {
		log.Fatal(err)
	}

	if len(tools.Tools) != 1 {
		t.Fatalf("Expected 1 tool, got %d", len(tools.Tools))
	}

	if tools.Tools[0].Name != "run_script" {
		t.Fatalf("Expected tool name to be 'run_script', got %s", tools.Tools[0].Name)
	}
}

func TestRunScript(t *testing.T) {
	ctx := context.Background()

	client := mcp.NewClient(&mcp.Implementation{Name: "flame-mcp-test", Version: "v1.0.0"}, nil)

	transport := mcp.NewCommandTransport(exec.Command("go", "run", "main.go", "server.go", "types.go"))
	session, err := client.Connect(ctx, transport)
	if err != nil {
		log.Fatal(err)
	}
	defer session.Close()

	tools, err := session.ListTools(ctx, &mcp.ListToolsParams{})
	if err != nil {
		log.Fatal(err)
	}

	tool := tools.Tools[0]
	params := &mcp.CallToolParams{
		Name:      tool.Name,
		Arguments: map[string]any{"session_id": "test", "language": "python", "code": "print('Hello, World!')"},
	}

	response, err := session.CallTool(ctx, params)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(response)
}
