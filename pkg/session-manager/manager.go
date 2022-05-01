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

package manager

import (
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/grpcs"
	"xflops.cn/flame/pkg/session-manager/backend"
	"xflops.cn/flame/pkg/session-manager/cache"
	"xflops.cn/flame/pkg/session-manager/frontend"
	"xflops.cn/flame/pkg/session-manager/scheduler"
)

func Run() {
	lis, err := net.Listen("tcp", ":8080")
	if err != nil {
		klog.Error("Failed to listen on: %s", err)
		return
	}

	c := cache.New()

	// Start scheduler
	go scheduler.Run(c)

	// Start frontend & backend grpc service
	s := grpc.NewServer()
	grpcs.RegisterFrontendServer(s, frontend.New(c))
	grpcs.RegisterBackendServer(s, backend.New(c))

	reflection.Register(s)
	err = s.Serve(lis)
	if err != nil {
		klog.Error("Failed to start session manager: %s", err)
		return
	}

}
