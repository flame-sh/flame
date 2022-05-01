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

package backend

import (
	"context"
	"fmt"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/grpcs"
	"xflops.cn/flame/pkg/session-manager/apis"
	"xflops.cn/flame/pkg/session-manager/cache"
)

type Backend struct {
	grpcs.BackendServer

	cache *cache.Cache
}

func New(c *cache.Cache) *Backend {
	res := &Backend{
		cache: c,
	}

	return res
}

func (b *Backend) Register(ctx context.Context, executor *grpcs.Executor) (*grpcs.Executor, error) {
	exeInfo := apis.NewExecutor(executor.Metadata.ID)
	if err := b.cache.AddExecutor(exeInfo); err != nil {
		return nil, err
	}

	return executor, nil
}

func (b *Backend) Unregister(ctx context.Context, executor *grpcs.Executor) (*grpcs.Result, error) {
	exeInfo := apis.NewExecutor(executor.Metadata.ID)
	if err := b.cache.RemoveExecutor(exeInfo); err != nil {
		return &grpcs.Result{ErrCode: -1, Message: err.Error()}, nil
	}

	return &grpcs.Result{ErrCode: 0}, nil
}

func (b *Backend) Bind(ctx context.Context, executor *grpcs.Executor) (*grpcs.Session, error) {
	exe, err := b.cache.GetExecutor(executor.Metadata.ID)
	if err != nil {
		return nil, err
	}

	ssnID := exe.WaitBinding()
	ssnInfo, err := b.cache.GetSession(ssnID)
	if err != nil {
		return nil, err
	}

	ssn := &grpcs.Session{
		Metadata: &grpcs.Metadata{
			ID:       ssnID,
			OwnerRef: ssnInfo.CONID,
		}}

	return ssn, nil
}

func (b *Backend) Unbind(ctx context.Context, executor *grpcs.Executor) (*grpcs.Result, error) {
	exe, err := b.cache.GetExecutor(executor.Metadata.ID)
	if err != nil {
		return nil, err
	}

	if err := exe.Unbind(); err != nil {
		return &grpcs.Result{
			ErrCode: -1,
			Message: err.Error(),
		}, nil
	}

	return &grpcs.Result{ErrCode: 0}, nil
}

func (b *Backend) GetTask(executor *grpcs.Executor, server grpcs.Backend_GetTaskServer) error {
	for {
		exe, err := b.cache.GetExecutor(executor.Metadata.ID)
		if err != nil {
			return err
		}

		// TODO(k82cn): exe.GetBoundSession()
		if exe.Status != apis.EXE_STATUS_BOUND {
			return fmt.Errorf("no bound session")
		}

		ssn, err := b.cache.GetSession(exe.SSNID)
		if err != nil {
			klog.Errorf("Failed to get session <%s>.", exe.SSNID)
			break
		}

		if ssn.IsClosed() {
			break
		}

		task := ssn.NextTask()
		if task == nil {
			klog.Infof("No more task in session <%s>.", ssn.ID)
			break
		} else {
			klog.Infof("Handling task <%s/%s>.", ssn.ID, task.ID)
		}

		input := &grpcs.TaskInput{
			TaskID:    task.ID,
			SessionID: task.SSNID,
			Input:     task.Input,
		}

		if err := server.Send(input); err != nil {
			return err
		}

		if err := ssn.WaitTask(task.ID); err != nil {
			return err
		}
	}

	return nil
}

func (b *Backend) CompleteTask(ctx context.Context, output *grpcs.TaskOutput) (*grpcs.Result, error) {
	res := &grpcs.Result{ErrCode: 0}

	ssn, err := b.cache.GetSession(output.SessionID)
	if err != nil {
		return nil, err
	}

	if err := ssn.CompleteTask(output.TaskID, output.Output); err != nil {
		res.ErrCode = -1
		res.Message = err.Error()
	}

	return res, nil
}
