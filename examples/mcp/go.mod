module github.com/flame-sh/flame/examples/mcp

go 1.23.0

toolchain go1.24.4

require (
	github.com/flame-sh/flame/sdk/go v0.0.0-00010101000000-000000000000
	github.com/modelcontextprotocol/go-sdk v0.2.0
)

require (
	github.com/yosida95/uritemplate/v3 v3.0.2 // indirect
	golang.org/x/net v0.40.0 // indirect
	golang.org/x/sys v0.33.0 // indirect
	golang.org/x/text v0.25.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250528174236-200df99c418a // indirect
	google.golang.org/grpc v1.74.2 // indirect
	google.golang.org/protobuf v1.36.6 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/flame-sh/flame/sdk/go => ../../sdk/go
