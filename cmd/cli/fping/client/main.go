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
	"time"

	"xflops.cn/flame/sdk/golang"
)

func main() {
	conn := golang.NewConnection()
	defer golang.CloseConnection(conn)

	ssn := conn.NewSession()
	defer conn.CloseSession(ssn)

	before := time.Now()
	task, err := ssn.SendInput([]byte("k82cn"))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Task <%s/%s> was created\n", task.SSNID, task.ID)
	output, err := ssn.RecvOutput(task)
	if err != nil {
		panic(err)
	}
	after := time.Now()

	rtt := after.Sub(before).Milliseconds()

	fmt.Printf("Task output is: %s (%d ms)\n", output.Output, rtt)
}
