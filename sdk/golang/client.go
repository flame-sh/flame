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
	"fmt"

	"google.golang.org/grpc"
	"xflops.cn/flame/pkg/grpcs"
)

var _conn *grpc.ClientConn
var _client grpcs.FrontendClient

func NewConnection() *Connection {
	if _conn == nil {
		var err error
		_conn, err = grpc.Dial(":8080", grpc.WithInsecure())
		if err != nil {
			fmt.Printf("Failed to connect to server: %v\n", err)
			return nil
		}
		_client = grpcs.NewFrontendClient(_conn)
	}

	// TODO (k82cn): build client with certification
	client := &grpcs.Client{Metadata: &grpcs.Metadata{Name: "localhost"}}
	conn, err := _client.NewConnection(context.Background(), client)
	if err != nil {
		fmt.Printf("Failed to create connection: %v\n", err)
		return nil
	}
	return &Connection{conn}
}

func CloseConnection(conn *Connection) {
}

type Connection struct {
	connection *grpcs.Connection
}

func (c *Connection) NewSession() *Session {
	ssn, err := _client.NewSession(context.Background(), c.connection)
	if err != nil {
		return nil
	}
	return &Session{ssn}
}

func (c *Connection) CloseSession(ssn *Session) {

}

type Session struct {
	ssn *grpcs.Session
}

func (s *Session) SendInput(input []byte) (*Task, error) {
	in := &grpcs.TaskInput{
		SessionID: s.ssn.Metadata.ID,
		Input:     string(input),
	}

	task, err := _client.SendInput(context.Background(), in)
	if err != nil {
		return nil, err
	}

	return &Task{
		task:  task,
		ID:    task.Metadata.ID,
		SSNID: task.Metadata.OwnerRef}, nil
}

func (s *Session) RecvOutput(task *Task) (*grpcs.TaskOutput, error) {
	output, err := _client.RecvOutput(context.Background(), task.task)
	if err != nil {
		return nil, err
	}
	return output, nil
}

type Task struct {
	task *grpcs.Task

	ID    string
	SSNID string
}
