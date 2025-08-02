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
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// LoadFlameContextFromFile loads a FlameContext from a file
func LoadFlameContextFromFile(filePath string) (*FlameContext, error) {
	if filePath == "" {
		// Use default path
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return nil, &FlameError{
				Code:    FlameErrorCodesInternal,
				Message: fmt.Sprintf("failed to get home directory: %v", err),
			}
		}
		filePath = filepath.Join(homeDir, ".flame", DefaultFlameConf)
	}

	// Check if file exists
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		return nil, &FlameError{
			Code:    FlameErrorCodesInvalidConfig,
			Message: fmt.Sprintf("<%s> is not a file", filePath),
		}
	}

	// Read file contents
	contents, err := os.ReadFile(filePath)
	if err != nil {
		return nil, &FlameError{
			Code:    FlameErrorCodesInternal,
			Message: fmt.Sprintf("failed to read file: %v", err),
		}
	}

	// Parse YAML
	var ctx FlameContext
	if err := yaml.Unmarshal(contents, &ctx); err != nil {
		return nil, &FlameError{
			Code:    FlameErrorCodesInternal,
			Message: fmt.Sprintf("failed to parse YAML: %v", err),
		}
	}

	return &ctx, nil
}

// SaveFlameContextToFile saves a FlameContext to a file
func SaveFlameContextToFile(ctx *FlameContext, filePath string) error {
	if filePath == "" {
		// Use default path
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return &FlameError{
				Code:    FlameErrorCodesInternal,
				Message: fmt.Sprintf("failed to get home directory: %v", err),
			}
		}
		filePath = filepath.Join(homeDir, ".flame", DefaultFlameConf)
	}

	// Ensure directory exists
	dir := filepath.Dir(filePath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return &FlameError{
			Code:    FlameErrorCodesInternal,
			Message: fmt.Sprintf("failed to create directory: %v", err),
		}
	}

	// Marshal to YAML
	contents, err := yaml.Marshal(ctx)
	if err != nil {
		return &FlameError{
			Code:    FlameErrorCodesInternal,
			Message: fmt.Sprintf("failed to marshal YAML: %v", err),
		}
	}

	// Write to file
	if err := os.WriteFile(filePath, contents, 0644); err != nil {
		return &FlameError{
			Code:    FlameErrorCodesInternal,
			Message: fmt.Sprintf("failed to write file: %v", err),
		}
	}

	return nil
}
