/*
Copyright 2023 The Flame Authors.
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
	"testing"
)

func TestSessionStateString(t *testing.T) {
	tests := []struct {
		state SessionState
		want  string
	}{
		{SessionStateOpen, "Open"},
		{SessionStateClosed, "Closed"},
		{SessionState(99), "Unknown"},
	}

	for _, tt := range tests {
		if got := tt.state.String(); got != tt.want {
			t.Errorf("SessionState.String() = %v, want %v", got, tt.want)
		}
	}
}

func TestTaskStateString(t *testing.T) {
	tests := []struct {
		state TaskState
		want  string
	}{
		{TaskStatePending, "Pending"},
		{TaskStateRunning, "Running"},
		{TaskStateSucceed, "Succeed"},
		{TaskStateFailed, "Failed"},
		{TaskState(99), "Unknown"},
	}

	for _, tt := range tests {
		if got := tt.state.String(); got != tt.want {
			t.Errorf("TaskState.String() = %v, want %v", got, tt.want)
		}
	}
}

func TestApplicationStateString(t *testing.T) {
	tests := []struct {
		state ApplicationState
		want  string
	}{
		{ApplicationStateEnabled, "Enabled"},
		{ApplicationStateDisabled, "Disabled"},
		{ApplicationState(99), "Unknown"},
	}

	for _, tt := range tests {
		if got := tt.state.String(); got != tt.want {
			t.Errorf("ApplicationState.String() = %v, want %v", got, tt.want)
		}
	}
}

func TestShimString(t *testing.T) {
	tests := []struct {
		shim Shim
		want string
	}{
		{ShimLog, "Log"},
		{ShimStdio, "Stdio"},
		{ShimWasm, "Wasm"},
		{ShimShell, "Shell"},
		{ShimGrpc, "Grpc"},
		{Shim(99), "Unknown"},
	}

	for _, tt := range tests {
		if got := tt.shim.String(); got != tt.want {
			t.Errorf("Shim.String() = %v, want %v", got, tt.want)
		}
	}
}

func TestFlameError(t *testing.T) {
	err := &FlameError{
		Code:    FlameErrorCodesInvalidConfig,
		Message: "resource not found",
	}
	if err.Code != FlameErrorCodesInvalidConfig {
		t.Errorf("Expected error code 'InvalidConfig', got %s", err.Code)
	}
	if err.Message != "resource not found" {
		t.Errorf("Expected error message 'resource not found', got %s", err.Message)
	}

	errorStr := err.Error()
	expected := "resource not found (0)"
	if errorStr != expected {
		t.Errorf("Expected error string '%s', got '%s'", expected, errorStr)
	}
}

func TestDefaultFlameContext(t *testing.T) {
	ctx := DefaultFlameContext()
	if ctx.Name != DefaultContextName {
		t.Errorf("Expected name '%s', got '%s'", DefaultContextName, ctx.Name)
	}
	if ctx.Endpoint != DefaultFlameEndpoint {
		t.Errorf("Expected endpoint '%s', got '%s'", DefaultFlameEndpoint, ctx.Endpoint)
	}
}

func TestFlameContextString(t *testing.T) {
	ctx := &FlameContext{
		Name:     "test",
		Endpoint: "http://localhost:8080",
	}
	expected := "name: test, endpoint: http://localhost:8080"
	if ctx.String() != expected {
		t.Errorf("Expected '%s', got '%s'", expected, ctx.String())
	}
}
