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

package flamego

import (
	"context"
	"fmt"
	"net"
	"os"
)

const (
	FlameServiceManager = "FLAME_SERVICE_MANAGER"
)

// ApplicationContext represents the context for an application
type ApplicationContext struct {
	Name    string
	URL     *string
	Command *string
}

// SessionContext represents the context for a session
type SessionContext struct {
	SessionID   string
	Application ApplicationContext
	CommonData  *CommonData
}

// TaskContext represents the context for a task
type TaskContext struct {
	TaskID    string
	SessionID string
	Input     *TaskInput
}

// FlameService is the interface that must be implemented by Flame services
type FlameService interface {
	OnSessionEnter(ctx context.Context, sessionCtx SessionContext) error
	OnTaskInvoke(ctx context.Context, taskCtx TaskContext) (*TaskOutput, error)
	OnSessionLeave(ctx context.Context) error
}

// ShimService wraps a FlameService to implement the gRPC shim interface
type ShimService struct {
	service FlameService
}

// NewShimService creates a new ShimService
func NewShimService(service FlameService) *ShimService {
	return &ShimService{service: service}
}

// OnSessionEnter handles session enter events
func (s *ShimService) OnSessionEnter(ctx context.Context, sessionCtx SessionContext) error {
	// TODO: Implement actual gRPC handling
	return s.service.OnSessionEnter(ctx, sessionCtx)
}

// OnTaskInvoke handles task invoke events
func (s *ShimService) OnTaskInvoke(ctx context.Context, taskCtx TaskContext) (*TaskOutput, error) {
	// TODO: Implement actual gRPC handling
	return s.service.OnTaskInvoke(ctx, taskCtx)
}

// OnSessionLeave handles session leave events
func (s *ShimService) OnSessionLeave(ctx context.Context) error {
	// TODO: Implement actual gRPC handling
	return s.service.OnSessionLeave(ctx)
}

// Run starts the Flame service
func Run(service FlameService) error {
	// TODO: Implement actual gRPC server setup
	// This would involve:
	// 1. Setting up gRPC server
	// 2. Registering the shim service
	// 3. Starting the server
	// 4. Registering with the service manager

	_ = NewShimService(service)

	// For now, just log that the service is ready
	fmt.Printf("Flame service ready\n")

	// TODO: Replace with actual server implementation
	// This is a placeholder that keeps the service running
	select {}
}

// Helper function to get service manager address
func getServiceManagerAddr() string {
	if addr := os.Getenv(FlameServiceManager); addr != "" {
		return addr
	}
	return "localhost:8080"
}

// Helper function to create a TCP listener
func createListener(addr string) (net.Listener, error) {
	return net.Listen("tcp", addr)
}
