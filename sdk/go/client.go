/*
Copyright 2025 The Flame Authors.
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

package flamego

import (
	"context"
	"net/url"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"

	rpc "github.com/flame-sh/flame/sdk/go/rpc"
)

// Connection represents a connection to the Flame service
type Connection struct {
	addr  string
	conn  *grpc.ClientConn
	front rpc.FrontendClient
}

// Connect establishes a connection to the Flame service
func Connect(addr string) (*Connection, error) {
	if addr == "" {
		return nil, &FlameError{
			Code:    FlameErrorCodesInvalidConfig,
			Message: "address cannot be empty",
		}
	}

	parsedAddr, err := url.Parse(addr)
	if err != nil {
		return nil, err
	}

	conn, err := grpc.NewClient(parsedAddr.Host, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, err
	}

	conn.WaitForStateChange(context.Background(), connectivity.Ready)

	return &Connection{addr: addr, conn: conn, front: rpc.NewFrontendClient(conn)}, nil
}

// Close closes the connection
func (c *Connection) Close() error {
	return c.conn.Close()
}

// SessionAttributes represents attributes for creating a session
type SessionAttributes struct {
	Application string
	Slots       int32
	CommonData  CommonData
}

// ApplicationAttributes represents attributes for an application
type ApplicationAttributes struct {
	Name             string
	Shim             Shim
	Image            string
	Command          string
	Arguments        []string
	Environments     map[string]string
	WorkingDirectory string
}

// Application represents an application
type Application struct {
	Name         ApplicationID
	Attributes   ApplicationAttributes
	State        ApplicationState
	CreationTime time.Time
}

// Session represents a session
type Session struct {
	client *Connection

	ID           SessionID
	Slots        int32
	Application  string
	CreationTime time.Time
	State        SessionState
	Pending      int32
	Running      int32
	Succeed      int32
	Failed       int32
}

// Task represents a task
type Task struct {
	ID     TaskID
	SsnID  SessionID
	State  TaskState
	Input  TaskInput
	Output TaskOutput
}

// IsCompleted returns true if the task is completed
func (t *Task) IsCompleted() bool {
	return t.State == TaskStateSucceed || t.State == TaskStateFailed
}

// TaskInformer is an interface for task updates
type TaskInformer interface {
	OnUpdate(task *Task)
	OnError(err FlameError)
}

// CreateSession creates a new session
func (c *Connection) CreateSession(ctx context.Context, attrs SessionAttributes) (*Session, error) {
	sessionSpec := &rpc.SessionSpec{
		Application: attrs.Application,
		Slots:       attrs.Slots,
		CommonData:  attrs.CommonData,
	}

	session, err := c.front.CreateSession(ctx, &rpc.CreateSessionRequest{
		Session: sessionSpec,
	})
	if err != nil {
		return nil, err
	}

	return &Session{
		client:       c,
		ID:           SessionID(session.Metadata.Id),
		Slots:        session.Spec.Slots,
		Pending:      int32(session.Status.Pending),
		Running:      int32(session.Status.Running),
		Succeed:      int32(session.Status.Succeed),
		Failed:       int32(session.Status.Failed),
		Application:  session.Spec.Application,
		CreationTime: time.Unix(session.Status.CreationTime, 0),
		State:        SessionState(session.Status.State),
	}, nil
}

// ListSession lists all sessions
func (c *Connection) ListSession(ctx context.Context) ([]*Session, error) {
	sessionList, err := c.front.ListSession(ctx, &rpc.ListSessionRequest{})
	if err != nil {
		return nil, err
	}

	sessionPtrs := make([]*Session, 0, len(sessionList.Sessions))
	for _, session := range sessionList.Sessions {
		sessionPtrs = append(sessionPtrs, &Session{
			ID:      SessionID(session.Metadata.Id),
			Slots:   session.Spec.Slots,
			Pending: int32(session.Status.Pending),
			Running: int32(session.Status.Running),
			Succeed: int32(session.Status.Succeed),
			Failed:  int32(session.Status.Failed),
		})
	}

	return sessionPtrs, nil
}

// RegisterApplication registers a new application
func (c *Connection) RegisterApplication(ctx context.Context, name string, app ApplicationAttributes) error {
	envs := make([]*rpc.Environment, 0, len(app.Environments))
	for k, v := range app.Environments {
		envs = append(envs, &rpc.Environment{
			Name:  k,
			Value: v,
		})
	}
	appSpec := &rpc.ApplicationSpec{
		Shim:             rpc.Shim(app.Shim),
		Image:            &app.Image,
		Command:          &app.Command,
		Arguments:        app.Arguments,
		Environments:     envs,
		WorkingDirectory: &app.WorkingDirectory,
	}

	_, err := c.front.RegisterApplication(ctx, &rpc.RegisterApplicationRequest{
		Application: appSpec,
	})
	return err
}

// ListApplication lists all applications
func (c *Connection) ListApplication(ctx context.Context) ([]*Application, error) {
	appList, err := c.front.ListApplication(ctx, &rpc.ListApplicationRequest{})
	if err != nil {
		return nil, err
	}

	appPtrs := make([]*Application, 0, len(appList.Applications))
	for _, app := range appList.Applications {
		envs := make(map[string]string)
		for _, env := range app.Spec.Environments {
			envs[env.Name] = env.Value
		}

		appPtrs = append(appPtrs, &Application{
			Name: ApplicationID(app.Metadata.Id),
			Attributes: ApplicationAttributes{
				Name:             app.Metadata.Name,
				Shim:             Shim(app.Spec.Shim),
				Image:            *app.Spec.Image,
				Command:          *app.Spec.Command,
				Arguments:        app.Spec.Arguments,
				Environments:     envs,
				WorkingDirectory: *app.Spec.WorkingDirectory,
			},
			State:        ApplicationState(app.Status.State),
			CreationTime: time.Unix(app.Status.CreationTime, 0),
		})
	}
	return appPtrs, nil
}

// CreateTask creates a new task in the session
func (s *Session) CreateTask(ctx context.Context, input TaskInput) (*Task, error) {
	taskSpec := &rpc.TaskSpec{
		SessionId: string(s.ID),
		Input:     input,
	}

	task, err := s.client.front.CreateTask(ctx, &rpc.CreateTaskRequest{
		Task: taskSpec,
	})
	if err != nil {
		return nil, err
	}

	return &Task{
		ID:    TaskID(task.Metadata.Id),
		SsnID: s.ID,
		State: TaskState(task.Status.State),
		Input: input,
	}, nil
}

// GetTask gets a task by ID
func (s *Session) GetTask(ctx context.Context, id TaskID) (*Task, error) {
	task, err := s.client.front.GetTask(ctx, &rpc.GetTaskRequest{
		TaskId:    string(id),
		SessionId: string(s.ID),
	})
	if err != nil {
		return nil, err
	}

	return &Task{
		ID:     TaskID(task.Metadata.Id),
		SsnID:  s.ID,
		State:  TaskState(task.Status.State),
		Input:  task.Spec.Input,
		Output: task.Spec.Output,
	}, nil
}

// RunTask runs a task with the given input
func (s *Session) RunTask(ctx context.Context, input TaskInput, informer TaskInformer) error {
	task, err := s.CreateTask(ctx, input)
	if err != nil {
		return err
	}

	return s.WatchTask(ctx, task.ID, informer)
}

// WatchTask watches a task for updates
func (s *Session) WatchTask(ctx context.Context, taskID TaskID, informer TaskInformer) error {
	stream, err := s.client.front.WatchTask(ctx, &rpc.WatchTaskRequest{
		SessionId: string(s.ID),
		TaskId:    string(taskID),
	})
	if err != nil {
		return err
	}

	for {
		task, err := stream.Recv()
		if err != nil {
			if informer != nil {
				informer.OnError(FlameError{
					Code:    FlameErrorCodesInternal,
					Message: err.Error(),
				})
			}
			return err
		}

		if informer != nil {
			informer.OnUpdate(&Task{
				ID:     TaskID(task.Metadata.Id),
				SsnID:  s.ID,
				State:  TaskState(task.Status.State),
				Input:  task.Spec.Input,
				Output: task.Spec.Output,
			})
		}

		if task.Status.State == rpc.TaskState_Succeed || task.Status.State == rpc.TaskState_Failed {
			break
		}
	}

	return nil
}

// Close closes the session
func (s *Session) Close(ctx context.Context) error {
	_, err := s.client.front.CloseSession(ctx, &rpc.CloseSessionRequest{
		SessionId: string(s.ID),
	})
	if err != nil {
		return err
	}

	return nil
}
