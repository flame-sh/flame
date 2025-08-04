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
	"encoding/json"
	"fmt"

	"github.com/modelcontextprotocol/go-sdk/mcp"

	flamego "github.com/flame-sh/flame/sdk/go"
)

// FlameMCPServer implements the MCP server for Flame script execution
type FlameMCPServer struct {
	flameClient *flamego.Connection
	sessions    map[string]*flamego.Session
}

// NewFlameMCPServer creates a new Flame MCP server
func NewFlameMCPServer(flameEndpoint string) (*FlameMCPServer, error) {
	client, err := flamego.Connect(flameEndpoint)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to Flame: %w", err)
	}

	return &FlameMCPServer{
		flameClient: client,
		sessions:    make(map[string]*flamego.Session),
	}, nil
}

// RunScript executes a script using Flame flmexec application
func (s *FlameMCPServer) RunScript(ctx context.Context, cc *mcp.ServerSession, req *mcp.CallToolParamsFor[RunScriptRequest]) (*mcp.CallToolResultFor[any], error) {
	// Check if the session already exists
	if _, ok := s.sessions[req.Arguments.SessionID]; !ok {
		// Create a session with flmexec application
		sessionAttrs := flamego.SessionAttributes{
			Application: "flmexec",
			Slots:       1,
			CommonData:  flamego.CommonData{},
		}
		session, err := s.flameClient.CreateSession(ctx, sessionAttrs)
		if err != nil {
			return nil, fmt.Errorf("failed to create session: %w", err)
		}
		s.sessions[req.Arguments.SessionID] = session
	}

	session := s.sessions[req.Arguments.SessionID]

	inputBytes, err := json.Marshal(Script{
		Input:    nil,
		Language: req.Arguments.Language,
		Code:     req.Arguments.Code,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to marshal script input: %w", err)
	}

	// Create task input
	taskInput := flamego.TaskInput(inputBytes)

	// Create a task informer to handle task updates
	taskInformer := &TaskInformer{
		outputChan: make(chan string, 1),
		errorChan:  make(chan error, 1),
	}

	// Run the task
	err = session.RunTask(ctx, taskInput, taskInformer)
	if err != nil {
		return nil, fmt.Errorf("failed to run task: %w", err)
	}

	// Wait for task completion
	select {
	case output := <-taskInformer.outputChan:
		return &mcp.CallToolResultFor[any]{
			Content: []mcp.Content{
				&mcp.TextContent{Text: output},
			},
		}, nil
	case err := <-taskInformer.errorChan:
		return nil, fmt.Errorf("task execution failed: %w", err)
	case <-ctx.Done():
		return nil, fmt.Errorf("task execution timed out")
	}
}

// TaskInformer implements flamego.TaskInformer interface
type TaskInformer struct {
	outputChan chan string
	errorChan  chan error
}

// OnUpdate handles task updates
func (t *TaskInformer) OnUpdate(task *flamego.Task) {
	switch task.State {
	case flamego.TaskStateSucceed:
		output := string(task.Output)
		t.outputChan <- output
	case flamego.TaskStateFailed:
		t.errorChan <- fmt.Errorf("task failed with state: %s", task.State.String())
	}
}

// OnError handles task errors
func (t *TaskInformer) OnError(err flamego.FlameError) {
	t.errorChan <- fmt.Errorf("flame error: %s", err.Message)
}
