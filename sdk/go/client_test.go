package flamego

import (
	"context"
	"testing"
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
		Application: "test",
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

	if session.Application != "test" {
		t.Fatalf("Session application is not test: %v", session.Application)
	}

}
