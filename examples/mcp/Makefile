.PHONY: build run test clean docker-build docker-run

# Build the MCP server
build:
	go mod tidy
	go build -o flame-mcp-server

# Run the MCP server
run: build
	FLAME_ADDR=$(FLAME_ADDR) ./flame-mcp-server

# Test the MCP server
test:
	go test -v .

# Clean build artifacts
clean:
	rm -f flame-mcp-server

# Build Docker image
docker-build:
	docker build -t flame-mcp-server .

# Run with Docker
docker-run: docker-build
	docker run -p 8080:8080 \
		-e FLAME_ADDR=$(FLAME_ADDR) \
		-e PORT=8080 \
		flame-mcp-server

# Show help
help:
	@echo "Available targets:"
	@echo "  build           - Build the MCP server"
	@echo "  run             - Build and run the MCP server"
	@echo "  test            - Run tests"
	@echo "  clean           - Clean build artifacts"
	@echo "  docker-build    - Build Docker image"
	@echo "  docker-run      - Run with Docker"
	@echo "  help            - Show this help" 