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

package cache

import (
	"fmt"
	"sync"

	"k8s.io/klog/v2"

	"xflops.cn/flame/pkg/session-manager/apis"
)

type Cache struct {
	// TODO(k82cn): use multiple mutex to improve performance
	mutex sync.Mutex

	connections []*apis.ConnectionInfo
	connIndex   map[string]*apis.ConnectionInfo

	sessions []*apis.SessionInfo
	ssnIndex map[string]*apis.SessionInfo

	executors []*apis.ExecutorInfo
	exeIndex  map[string]*apis.ExecutorInfo
}

type Snapshot struct {
	Sessions     []*apis.SessionInfo
	SessionIndex map[string]*apis.SessionInfo

	Executors     []*apis.ExecutorInfo
	ExecutorIndex map[string]*apis.ExecutorInfo
}

func New() *Cache {
	c := &Cache{}

	c.ssnIndex = make(map[string]*apis.SessionInfo)
	c.exeIndex = make(map[string]*apis.ExecutorInfo)
	c.connIndex = make(map[string]*apis.ConnectionInfo)

	return c
}

func (c *Cache) AddTask(task *apis.TaskInfo) error {
	c.mutex.Lock()
	c.mutex.Unlock()

	ssn, found := c.ssnIndex[task.SSNID]
	if !found {
		return fmt.Errorf("failed to found session <%s> for task <%s>",
			task.SSNID, task.ID)
	}
	if err := ssn.AddTask(task); err != nil {
		return err
	}

	klog.Infof("Task <%s> was added into cache.", task.ID)

	return nil
}

func (c *Cache) AddSession(ssn *apis.SessionInfo) error {
	c.mutex.Lock()
	c.mutex.Unlock()

	if _, found := c.ssnIndex[ssn.ID]; found {
		return fmt.Errorf("duplicated session <%s> in connection <%s>",
			ssn.ID, ssn.CONID)
	}

	c.sessions = append(c.sessions, ssn)
	c.ssnIndex[ssn.ID] = ssn

	klog.Infof("Session <%s> was added into cache.", ssn.ID)

	return nil
}

func (c *Cache) AddConnection(conn *apis.ConnectionInfo) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	if _, found := c.connIndex[conn.ID]; found {
		return fmt.Errorf("duplicated connection <%s>", conn.ID)
	}

	c.connections = append(c.connections, conn)
	c.connIndex[conn.ID] = conn

	klog.Infof("Connection <%s> was added into cache.", conn.ID)

	return nil
}

func (c *Cache) GetTask(ssnID string, taskID string) (*apis.TaskInfo, error) {
	ssn, err := c.GetSession(ssnID)
	if err != nil {
		return nil, err
	}

	return ssn.GetTask(taskID)
}

func (c *Cache) GetSession(ssnID string) (*apis.SessionInfo, error) {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	if ssn, foundSSN := c.ssnIndex[ssnID]; foundSSN {
		return ssn, nil
	}

	return nil, fmt.Errorf("failed to find session %s", ssnID)
}

func (c *Cache) AddExecutor(exe *apis.ExecutorInfo) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	if _, found := c.exeIndex[exe.ID]; found {
		return fmt.Errorf("duplicated executor <%s>", exe.ID)
	}

	c.executors = append(c.executors, exe)
	c.exeIndex[exe.ID] = exe

	klog.Infof("Executor <%s> was added into cache.", exe.ID)

	return nil
}

func (c *Cache) RemoveExecutor(exe *apis.ExecutorInfo) error {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	for i, e := range c.executors {
		if e.ID == exe.ID {
			c.executors[i] = c.executors[len(c.executors)-1]
			c.executors = c.executors[:len(c.executors)-1]
			break
		}
	}

	delete(c.exeIndex, exe.ID)

	klog.Infof("Executor <%s> was removed frome cache.", exe.ID)

	return nil
}

func (c *Cache) GetExecutor(id string) (*apis.ExecutorInfo, error) {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	if exe, found := c.exeIndex[id]; found {
		return exe, nil
	}

	return nil, fmt.Errorf("failed to find executor %s", id)
}

func (c *Cache) SnapShot() *Snapshot {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	res := &Snapshot{
		SessionIndex:  make(map[string]*apis.SessionInfo),
		ExecutorIndex: make(map[string]*apis.ExecutorInfo),
	}

	for _, exe := range c.executors {
		ec := exe.Clone()
		res.Executors = append(res.Executors, ec)
		res.ExecutorIndex[ec.ID] = ec
	}

	for _, ssn := range c.sessions {
		sc := ssn.Clone()
		res.Sessions = append(res.Sessions, sc)
		res.SessionIndex[sc.ID] = sc
	}

	return res
}

func (c *Cache) Bind(exe *apis.ExecutorInfo, ssn *apis.SessionInfo) error {
	e, err := c.GetExecutor(exe.ID)
	if err != nil {
		return err
	}
	return e.Bind(ssn.ID)
}

func (c *Cache) GetConnection(id string) *apis.ConnectionInfo {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	if conn, found := c.connIndex[id]; found {
		return conn
	}

	klog.Warningf("Failed to find connection <%s> in cache.", id)

	return nil
}

func (c *Cache) ListSession() []*apis.SessionInfo {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	var res []*apis.SessionInfo

	for i := 0; i < 10 && i < len(c.sessions); i++ {
		res = append(res, c.sessions[i])
	}

	return res
}
