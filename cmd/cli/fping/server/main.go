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

package main

import (
	"fmt"
	"os"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/grpcs"
	"xflops.cn/flame/sdk/golang"
)

type MyService struct {
}

func (m *MyService) OnRegistered() {
	fmt.Println("OnRegistered")
}

func (m *MyService) OnUnregistered() {
	fmt.Println("OnUnregistered")
}

func (m *MyService) OnSessionBound(*grpcs.Session) {
	fmt.Println("OnSessionBound")
}

func (m *MyService) OnSessionUnbound(*grpcs.Session) {
	fmt.Println("OnSessionUnBound")
}

func (m *MyService) OnTaskInvoke(input *grpcs.TaskInput) *grpcs.TaskOutput {
	fmt.Println("OnTaskInvoke")

	output := &grpcs.TaskOutput{
		TaskID:    input.TaskID,
		SessionID: input.SessionID,
		Output:    fmt.Sprintf("Hello %s!", input.Input),
	}

	return output
}

func main() {
	klog.InitFlags(nil)

	svcMgr := &golang.ServiceManager{Service: &MyService{}}

	exeName, err := os.Hostname()
	if err != nil {
		klog.Warningln("Failed to get hostname for executor")
		exeName = "localhost"
	}

	svcMgr.Run(exeName)
}
