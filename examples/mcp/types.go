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

const (
	FlameEnvVar          = "FLAME_ENDPOINT"
	DefaultFlameEndpoint = "http://127.0.0.1:8080"
	MCPPortEnvVar        = "FLAME_MCP_PORT"
)

// RunScriptRequest represents the request parameters for run_script
type RunScriptRequest struct {
	/// The session ID to use for the script.
	SessionID string `json:"session_id"`
	/// The language of the script.
	Language string `json:"language"`
	/// The code of the script.
	Code string `json:"code"`
}

type Script struct {
	/// The input to the script.
	Input []byte `json:"input,omitempty"`
	/// The language of the script.
	Language string `json:"language"`
	/// The code of the script.
	Code string `json:"code"`
}

// RunScriptResponse represents the response from run_script
type RunScriptResponse struct {
	Output string `json:"output"`
	Error  string `json:"error,omitempty"`
}
