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

package scheduler

import (
	"time"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/session-manager/apis"
	"xflops.cn/flame/pkg/session-manager/cache"
)

func Run(cache *cache.Cache) {
	klog.Infof("Starting scheduler ....")
	defer klog.Infof("Scheduler exit.")

	for {
		time.Sleep(time.Millisecond * 1)

		// TODO(k82cn): enable cond to trigger scheduling
		snapshot := cache.SnapShot()

		exeMap := map[string][]*apis.ExecutorInfo{}
		var idleExe []*apis.ExecutorInfo

		for _, exe := range snapshot.Executors {
			if len(exe.SSNID) == 0 {
				idleExe = append(idleExe, exe)
				continue
			}

			exes, found := exeMap[exe.SSNID]
			if !found {
				exes = []*apis.ExecutorInfo{}
			}

			exes = append(exes, exe)
			exeMap[exe.SSNID] = exes
		}

		if len(idleExe) != 0 {
			klog.V(3).Infof("There're <%d> idle executors.", len(idleExe))
		}

		for _, exe := range idleExe {
			for _, ssn := range snapshot.Sessions {
				if len(ssn.TaskStatusIndex[apis.TASK_STATUS_PENDING]) > len(exeMap[ssn.ID]) {
					if err := cache.Bind(exe, ssn); err != nil {
						klog.Errorf("Failed to bind executor <%s> to session <%s>: %v.", exe.ID, ssn.ID, err)
						continue
					}
					exeMap[ssn.ID] = append(exeMap[ssn.ID], exe)
					klog.V(3).Infof("Bound executor <%s> to session <%s>.", exe.ID, ssn.ID)
				}
			}
		}
	}
}
