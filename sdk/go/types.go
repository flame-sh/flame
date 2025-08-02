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
	"fmt"
)

// Type aliases
type TaskID string
type SessionID string
type ApplicationID string
type Message []byte
type TaskInput Message
type TaskOutput Message
type CommonData Message

// Constants
const (
	DefaultFlameConf     = "flame-conf.yaml"
	DefaultContextName   = "flame"
	DefaultFlameEndpoint = "http://127.0.0.1:8080"
)

// SessionState represents the state of a session
type SessionState int

const (
	SessionStateOpen SessionState = iota
	SessionStateClosed
)

func (s SessionState) String() string {
	switch s {
	case SessionStateOpen:
		return "Open"
	case SessionStateClosed:
		return "Closed"
	default:
		return "Unknown"
	}
}

// TaskState represents the state of a task
type TaskState int

const (
	TaskStatePending TaskState = iota
	TaskStateRunning
	TaskStateSucceed
	TaskStateFailed
)

func (t TaskState) String() string {
	switch t {
	case TaskStatePending:
		return "Pending"
	case TaskStateRunning:
		return "Running"
	case TaskStateSucceed:
		return "Succeed"
	case TaskStateFailed:
		return "Failed"
	default:
		return "Unknown"
	}
}

// ApplicationState represents the state of an application
type ApplicationState int

const (
	ApplicationStateEnabled ApplicationState = iota
	ApplicationStateDisabled
)

func (a ApplicationState) String() string {
	switch a {
	case ApplicationStateEnabled:
		return "Enabled"
	case ApplicationStateDisabled:
		return "Disabled"
	default:
		return "Unknown"
	}
}

// Shim represents the type of shim
type Shim int

const (
	ShimLog Shim = iota
	ShimStdio
	ShimWasm
	ShimShell
	ShimGrpc
)

func (s Shim) String() string {
	switch s {
	case ShimLog:
		return "Log"
	case ShimStdio:
		return "Stdio"
	case ShimWasm:
		return "Wasm"
	case ShimShell:
		return "Shell"
	case ShimGrpc:
		return "Grpc"
	default:
		return "Unknown"
	}
}

type FlameErrorCodes int

const (
	FlameErrorCodesInvalidConfig FlameErrorCodes = iota
	FlameErrorCodesInvalidState
	FlameErrorCodesInvalidArgument
	FlameErrorCodesInternal
)

func (f FlameErrorCodes) String() string {
	switch f {
	case FlameErrorCodesInvalidConfig:
		return "InvalidConfig"
	case FlameErrorCodesInvalidState:
		return "InvalidState"
	case FlameErrorCodesInvalidArgument:
		return "InvalidArgument"
	case FlameErrorCodesInternal:
		return "Internal"
	default:
		return "Unknown"
	}
}

// FlameError represents errors in the Flame SDK
type FlameError struct {
	Code    FlameErrorCodes
	Message string
}

func (e *FlameError) Error() string {
	return fmt.Sprintf("%s (%d)", e.Message, e.Code)
}

// FlameContext represents the Flame context configuration
type FlameContext struct {
	Name     string `yaml:"name"`
	Endpoint string `yaml:"endpoint"`
}

func (c *FlameContext) String() string {
	return fmt.Sprintf("name: %s, endpoint: %s", c.Name, c.Endpoint)
}

// DefaultFlameContext returns a default FlameContext
func DefaultFlameContext() *FlameContext {
	return &FlameContext{
		Name:     DefaultContextName,
		Endpoint: DefaultFlameEndpoint,
	}
}
