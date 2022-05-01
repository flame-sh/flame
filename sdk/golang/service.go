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

package golang

import (
	"context"

	"k8s.io/klog/v2"

	"google.golang.org/grpc"
	"xflops.cn/flame/pkg/grpcs"
)

type Service interface {
	OnRegistered()
	OnUnregistered()

	OnSessionBound(*grpcs.Session)
	OnSessionUnbound(*grpcs.Session)

	OnTaskInvoke(input *grpcs.TaskInput) *grpcs.TaskOutput
}

type ServiceManager struct {
	Service Service
}

func (sm *ServiceManager) Run(name string) {
	conn, err := grpc.Dial(":8080", grpc.WithInsecure())
	if err != nil {
		klog.Errorf("Failed to connect to lsm: %s", err)
		return
	}
	defer conn.Close()

	c := grpcs.NewBackendClient(conn)
	executor := &grpcs.Executor{Metadata: &grpcs.Metadata{ID: name, Name: name}}
	exec, err := c.Register(context.Background(), executor)
	if err != nil {
		return
	}

	sm.Service.OnRegistered()

	for {
		ssn, err := c.Bind(context.Background(), exec)
		if err != nil || ssn == nil {
			break
		}

		sm.Service.OnSessionBound(ssn)

		taskClient, err := c.GetTask(context.Background(), exec)
		if taskClient == nil || err != nil {
			break
		}

		for {
			//handle task input
			input, err := taskClient.Recv()
			if err != nil || input == nil {
				break
			}
			output := sm.Service.OnTaskInvoke(input)

			res, err := c.CompleteTask(context.Background(), output)
			if err != nil || res == nil {
				klog.Errorf("Failed to complete task <%s>: %v", output.SessionID, err)
			}
		}

		sm.Service.OnSessionUnbound(ssn)
		res, err := c.Unbind(context.Background(), exec)
		if res == nil || err != nil {
			klog.Errorf("Failed to unbind from session <%s>: %v", ssn.Metadata.ID, err)
		} else if res.ErrCode != 0 {
			klog.Errorf("Failed to unbind from session <%s>: %v", ssn.Metadata.ID, res.Message)
		}
	}

	sm.Service.OnUnregistered()
	res, err := c.Unregister(context.Background(), exec)
	if err != nil || res == nil {
		klog.Error("Failed to unregister from LSM: %v", err)
	} else if res.ErrCode != 0 {
		klog.Errorf("Failed to unregister from LSM: %v", res.Message)
	}
}
