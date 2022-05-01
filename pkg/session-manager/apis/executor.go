/*
  Copyright 2022 The Flame Authors.
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

package apis

import (
	"fmt"
	"sync"
)

const (
	EXE_STATUS_IDLE = iota
	EXE_STATUS_BOUND
)

type ExecutorInfo struct {
	ID     string
	Status int
	SSNID  string

	cond *sync.Cond
}

func (e *ExecutorInfo) Clone() *ExecutorInfo {
	return &ExecutorInfo{
		ID:     e.ID,
		SSNID:  e.SSNID,
		Status: e.Status,
	}
}

func (e *ExecutorInfo) WaitBinding() string {
	e.cond.L.Lock()
	defer e.cond.L.Unlock()

	for e.Status != EXE_STATUS_BOUND {
		e.cond.Wait()
	}

	return e.SSNID
}

func (e *ExecutorInfo) Unbind() error {
	e.cond.L.Lock()
	defer e.cond.L.Unlock()

	e.SSNID = ""
	e.Status = EXE_STATUS_IDLE

	e.cond.Broadcast()

	return nil
}

func (e *ExecutorInfo) Bind(ssnID string) error {
	e.cond.L.Lock()
	defer e.cond.L.Unlock()

	if len(e.SSNID) != 0 {
		return fmt.Errorf("failed to bind executor <%s> to session <%s>",
			e.ID, e.SSNID)
	}

	e.SSNID = ssnID
	e.Status = EXE_STATUS_BOUND

	e.cond.Broadcast()

	return nil
}

func NewExecutor(id string) *ExecutorInfo {
	exe := &ExecutorInfo{
		ID:     id,
		Status: EXE_STATUS_IDLE,
		cond:   sync.NewCond(&sync.Mutex{}),
	}

	return exe
}
