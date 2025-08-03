package flamego

import (
	"context"
	"testing"
)

const (
	DefaultFlameTestApp = "flmtest"
)

func TestConnect(t *testing.T) {
	conn, err := Connect(DefaultFlameEndpoint)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()
}

func TestSession(t *testing.T) {
	conn, err := Connect(DefaultFlameEndpoint)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(context.Background(), SessionAttributes{
		Application: DefaultFlameTestApp,
		Slots:       1,
		CommonData:  []byte("test"),
	})

	if err != nil {
		t.Fatalf("Failed to create session: %v", err)
	}
	defer session.Close(context.Background())

	if session.State != SessionStateOpen {
		t.Fatalf("Session state is not open: %v", session.State)
	}

	if session.Application != DefaultFlameTestApp {
		t.Fatalf("Session application is not %s: %v", DefaultFlameTestApp, session.Application)
	}

}

func TestTasks(t *testing.T) {
	conn, err := Connect(DefaultFlameEndpoint)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(context.Background(), SessionAttributes{
		Application: DefaultFlameTestApp,
		Slots:       1,
		CommonData:  []byte("test"),
	})
	if err != nil {
		t.Fatalf("Failed to create session: %v", err)
	}
	defer session.Close(context.Background())

	for i := 0; i < 10; i++ {
		err = session.RunTask(context.Background(), nil, nil)
		if err != nil {
			t.Fatalf("Failed to run task: %v", err)
		}
	}
}
