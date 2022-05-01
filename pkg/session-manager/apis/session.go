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

	"k8s.io/klog/v2"

	"github.com/google/uuid"
)

type SessionStatus int32

const (
	SSN_STATUS_OPEN SessionStatus = iota
	SSN_STATUS_CLOSED
)

type SessionInfo struct {
	CONID string
	ID    string

	mutex sync.Mutex
	cond  *sync.Cond

	Status SessionStatus

	Tasks           []*TaskInfo
	TaskIndex       map[string]*TaskInfo
	TaskStatusIndex map[TaskStatus]map[string]*TaskInfo
}

func NewSession(connID string) *SessionInfo {
	ssn := &SessionInfo{
		ID:    uuid.New().String(),
		CONID: connID,

		Status: SSN_STATUS_OPEN,

		TaskIndex:       make(map[string]*TaskInfo),
		TaskStatusIndex: make(map[TaskStatus]map[string]*TaskInfo),
	}

	ssn.cond = sync.NewCond(&ssn.mutex)

	return ssn
}

func (s *SessionInfo) Clone() *SessionInfo {
	ssn := &SessionInfo{
		ID:    s.ID,
		CONID: s.CONID,
		// Cloned info should not use cond
		cond:            nil,
		Status:          s.Status,
		Tasks:           []*TaskInfo{},
		TaskIndex:       make(map[string]*TaskInfo),
		TaskStatusIndex: make(map[TaskStatus]map[string]*TaskInfo),
	}

	for _, task := range s.Tasks {
		if err := ssn.addTask(task.Clone()); err != nil {
			klog.Errorf("Failed to add cloned task <%s> into session <%s>.", task.ID, ssn.ID)
		}
	}

	return ssn
}

func (s *SessionInfo) AddTask(task *TaskInfo) error {
	s.mutex.Lock()
	defer s.mutex.Unlock()

	return s.addTask(task)
}

func (s *SessionInfo) GetTask(id string) (*TaskInfo, error) {
	s.mutex.Lock()
	defer s.mutex.Unlock()

	if task, found := s.TaskIndex[id]; found {
		return task, nil
	}

	return nil, fmt.Errorf("failed to find task %s in session %s", id, s.ID)
}

func (s *SessionInfo) addTask(task *TaskInfo) error {
	s.Tasks = append(s.Tasks, task)
	s.TaskIndex[task.ID] = task

	if s.TaskStatusIndex[task.Status] == nil {
		s.TaskStatusIndex[task.Status] = make(map[string]*TaskInfo)
	}
	s.TaskStatusIndex[task.Status][task.ID] = task

	return nil
}

func (s *SessionInfo) removeTask(task *TaskInfo) error {
	for i, t := range s.Tasks {
		if t.ID == task.ID {
			s.Tasks[i] = s.Tasks[len(s.Tasks)-1]
			s.Tasks = s.Tasks[:len(s.Tasks)-1]
			break
		}
	}

	for _, v := range s.TaskStatusIndex {
		delete(v, task.ID)
	}

	delete(s.TaskIndex, task.ID)

	return nil
}

func (s *SessionInfo) UpdateTask(task *TaskInfo) error {
	s.mutex.Lock()
	defer s.mutex.Unlock()

	if err := s.removeTask(task); err != nil {
		return err
	}

	if err := s.addTask(task); err != nil {
		return err
	}

	return nil
}

func (s *SessionInfo) CompleteTask(id string, output string) error {
	task, err := s.GetTask(id)
	if err != nil {
		return err
	}

	return task.Complete(output, func(t *TaskInfo) error {
		return s.UpdateTask(t)
	})
}

func (s *SessionInfo) NextTask() *TaskInfo {
	s.mutex.Lock()
	defer s.mutex.Unlock()

	// Return first task
	for _, v := range s.TaskStatusIndex[TASK_STATUS_PENDING] {
		return v
	}

	return nil
}

func (s *SessionInfo) WaitTask(id string) error {
	task, err := s.GetTask(id)
	if err != nil {
		klog.Error("Failed to wait task %s: %v", id, err)
		return err
	}

	return task.Wait(func(t *TaskInfo) error {
		return s.UpdateTask(t)
	})
}

func (s *SessionInfo) Close() error {
	s.cond.L.Lock()
	defer s.cond.L.Unlock()

	s.Status = SSN_STATUS_CLOSED

	s.cond.Broadcast()

	return nil
}

func (s *SessionInfo) IsClosed() bool {
	s.cond.L.Lock()
	defer s.cond.L.Unlock()

	return s.Status == SSN_STATUS_CLOSED
}

type TaskStatus int32

const (
	TASK_STATUS_PENDING TaskStatus = iota
	TASK_STATUS_RUNNING
	TASK_STATUS_FAILED
	TASK_STATUS_SUCCEED
)

type TaskInfo struct {
	SSNID string
	ID    string

	Input  string
	Output string

	Cond *sync.Cond

	Status TaskStatus
}

func (t *TaskInfo) Clone() *TaskInfo {
	task := &TaskInfo{
		SSNID: t.SSNID,
		ID:    t.ID,

		Input:  t.Input,
		Output: t.Output,

		Cond: nil,

		Status: t.Status,
	}

	return task
}

func NewTask(ssnID string) *TaskInfo {
	task := &TaskInfo{
		ID:     uuid.New().String(),
		SSNID:  ssnID,
		Cond:   sync.NewCond(&sync.Mutex{}),
		Status: TASK_STATUS_PENDING,
	}

	return task
}

func (t *TaskInfo) IsCompleted() bool {
	return t.Status == TASK_STATUS_FAILED || t.Status == TASK_STATUS_SUCCEED
}

func (t *TaskInfo) IsFailed() bool {
	// TODO(k82cn): including terminated, abort and so on.
	return t.Status == TASK_STATUS_FAILED
}

func (t *TaskInfo) GetOutput() (string, error) {
	t.Cond.L.Lock()
	defer t.Cond.L.Unlock()

	for !t.IsCompleted() {
		klog.Infof("Waiting for task output <%s/%s>.",
			t.SSNID, t.ID)
		t.Cond.Wait()
	}

	if t.IsFailed() {
		return "", fmt.Errorf("task is failed, no output")
	}

	return t.Output, nil
}

func (t *TaskInfo) Complete(output string, cb func(task *TaskInfo) error) error {
	t.Cond.L.Lock()
	defer t.Cond.L.Unlock()

	t.Output = output
	t.Status = TASK_STATUS_SUCCEED

	if err := cb(t); err != nil {
		return err
	}

	t.Cond.Broadcast()

	return nil
}

func (t *TaskInfo) Wait(cb func(t *TaskInfo) error) error {
	t.Cond.L.Lock()
	defer t.Cond.L.Unlock()

	// Task is running
	t.Status = TASK_STATUS_RUNNING

	if err := cb(t); err != nil {
		return err
	}

	t.Cond.Wait()

	return nil
}
