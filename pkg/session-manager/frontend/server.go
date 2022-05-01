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

package frontend

import (
	"context"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/grpcs"
	"xflops.cn/flame/pkg/session-manager/apis"
	"xflops.cn/flame/pkg/session-manager/cache"
)

type Frontend struct {
	grpcs.FrontendServer

	cache *cache.Cache
}

func New(cache *cache.Cache) *Frontend {
	res := &Frontend{
		cache: cache,
	}

	return res
}

func (f *Frontend) NewConnection(ctx context.Context, client *grpcs.Client) (*grpcs.Connection, error) {
	// Check client's certification
	c := apis.NewConnection()
	if err := f.cache.AddConnection(c); err != nil {
		return nil, err
	}

	klog.Infof("Add connection <%v> into cache.", c.ID)

	return &grpcs.Connection{Metadata: &grpcs.Metadata{ID: c.ID}}, nil
}

func (f *Frontend) CloseConnection(ctx context.Context, conn *grpcs.Connection) (*grpcs.Result, error) {
	c := f.cache.GetConnection(conn.Metadata.ID)

	// TODO(k82cn): close sessions

	if err := c.Close(); err != nil {
		return &grpcs.Result{ErrCode: -1, Message: err.Error()}, nil
	}

	return &grpcs.Result{ErrCode: 0}, nil
}

func (f *Frontend) NewSession(ctx context.Context, connection *grpcs.Connection) (*grpcs.Session, error) {
	ssnInfo := apis.NewSession(connection.Metadata.ID)
	if err := f.cache.AddSession(ssnInfo); err != nil {
		return nil, err
	}

	ssn := &grpcs.Session{Metadata: &grpcs.Metadata{ID: ssnInfo.ID, OwnerRef: ssnInfo.CONID}}

	return ssn, nil
}

func (f *Frontend) CloseSession(ctx context.Context, ssn *grpcs.Session) (*grpcs.Result, error) {
	ssnInfo, err := f.cache.GetSession(ssn.Metadata.ID)
	if err != nil {
		return &grpcs.Result{ErrCode: -1, Message: err.Error()}, nil
	}

	// TODO(k82cn): unbind executors

	if err := ssnInfo.Close(); err != nil {
		return &grpcs.Result{ErrCode: -1, Message: err.Error()}, nil
	}
	return &grpcs.Result{ErrCode: 0}, nil
}

func (f *Frontend) ListSession(context.Context, *grpcs.Client) (*grpcs.SessionList, error) {
	var res []*grpcs.Session

	for _, s := range f.cache.ListSession() {
		res = append(res, &grpcs.Session{Metadata: &grpcs.Metadata{ID: s.ID}})
	}

	return &grpcs.SessionList{Sessions: res}, nil
}

func (f *Frontend) SendInput(context context.Context, input *grpcs.TaskInput) (*grpcs.Task, error) {
	taskInfo := apis.NewTask(input.SessionID)
	taskInfo.Input = input.Input

	if err := f.cache.AddTask(taskInfo); err != nil {
		return nil, err
	}

	task := &grpcs.Task{
		Metadata: &grpcs.Metadata{
			ID:       taskInfo.ID,
			OwnerRef: taskInfo.SSNID,
		},
	}

	return task, nil
}

func (f *Frontend) RecvOutput(ctx context.Context, task *grpcs.Task) (*grpcs.TaskOutput, error) {
	taskInfo, err := f.cache.GetTask(task.Metadata.OwnerRef, task.Metadata.ID)
	if err != nil {
		return nil, err
	}

	data, err := taskInfo.GetOutput()
	if err != nil {
		return nil, err
	}

	output := &grpcs.TaskOutput{
		TaskID:    task.Metadata.ID,
		SessionID: task.Metadata.OwnerRef,
		Output:    data,
	}

	return output, nil
}

func (f *Frontend) Run(grpcs.Frontend_RunServer) error {
	panic("not implemented yet")
}
