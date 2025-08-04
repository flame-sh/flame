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
	"net/http"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

func main() {
	// Get Flame address from environment or use default
	flameEndpoint := os.Getenv(FlameEnvVar)
	if flameEndpoint == "" {
		flameEndpoint = DefaultFlameEndpoint
	}

	// Create Flame MCP server
	flameServer, err := NewFlameMCPServer(flameEndpoint)
	if err != nil {
		log.Fatalf("Failed to create Flame MCP server: %v", err)
	}
	defer flameServer.flameClient.Close()

	// Get port from environment or use default

	server := mcp.NewServer(&mcp.Implementation{Name: "flame-mcp"}, nil)
	mcp.AddTool(server, &mcp.Tool{Name: "run_script", Description: "Run a script"}, flameServer.RunScript)

	port := os.Getenv(MCPPortEnvVar)
	if port == "" {
		// Run the server over stdin/stdout, until the client disconnects
		if err := server.Run(context.Background(), mcp.NewStdioTransport()); err != nil {
			log.Fatal(err)
		}
	} else {
		handler := mcp.NewStreamableHTTPHandler(func(*http.Request) *mcp.Server {
			return server
		}, nil)

		host := fmt.Sprintf("0.0.0.0:%s", port)
		log.Printf("MCP handler listening at %s", host)
		if err := http.ListenAndServe(host, handler); err != nil {
			log.Fatal(err)
		}
	}

}
