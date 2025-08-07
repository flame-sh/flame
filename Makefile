# Docker image configuration
DOCKER_REGISTRY ?= xflops
FSM_TAG ?= $(shell cargo get --entry session_manager/ package.version --pretty)
FEM_TAG ?= $(shell cargo get --entry executor_manager/ package.version --pretty)
CONSOLE_TAG ?= latest

# Docker image names
FSM_IMAGE = $(DOCKER_REGISTRY)/flame-session-manager
FEM_IMAGE = $(DOCKER_REGISTRY)/flame-executor-manager
CONSOLE_IMAGE = $(DOCKER_REGISTRY)/flame-console

# Dockerfile paths
FSM_DOCKERFILE = docker/Dockerfile.fsm
FEM_DOCKERFILE = docker/Dockerfile.fem
CONSOLE_DOCKERFILE = docker/Dockerfile.console

# Default target
.PHONY: help build docker-build docker-push docker-release docker-clean update_protos init sdk-go-build sdk-go-test sdk-go-clean

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: update_protos ## Build the Rust project
	cargo build

init: ## Install required tools
	cargo install cargo-get --force

update_protos: ## Update protobuf files
	@cp rpc/protos/frontend.proto sdk/go/protos
	@cp rpc/protos/types.proto sdk/go/protos
	@cp rpc/protos/shim.proto sdk/go/protos
	@echo "Copied protobuf files to sdk/go/protos"

	@cp rpc/protos/frontend.proto sdk/rust/protos
	@cp rpc/protos/types.proto sdk/rust/protos
	@cp rpc/protos/shim.proto sdk/rust/protos
	@echo "Copied protobuf files to sdk/rust/protos"

	@cp rpc/protos/frontend.proto sdk/python/protos
	@cp rpc/protos/types.proto sdk/python/protos
	@cp rpc/protos/shim.proto sdk/python/protos
	@echo "Copied protobuf files to sdk/python/protos"

sdk-python-generate: update_protos ## Generate the Python protobuf files
	cd sdk/python && make build-protos

sdk-python-test: update_protos ## Test the Python SDK
	cd sdk/python && make test

sdk-python-clean: ## Clean Python SDK build artifacts
	cd sdk/python && make clean

sdk-python: sdk-python-generate sdk-python-test ## Build and test the Python SDK

# Go SDK targets
sdk-go-generate: update_protos ## Generate the Go protobuf files
	@export PATH="$(go env GOPATH)/bin:${PATH}"
	cd sdk/go && protoc --proto_path=protos \
		--go_out=./rpc \
		--go_opt=paths=source_relative \
		--go-grpc_out=./rpc \
		--go-grpc_opt=paths=source_relative \
		protos/*.proto

sdk-go-test: update_protos ## Test the Go SDK
	cd sdk/go && go test -v ./...

sdk-go-clean: ## Clean Go SDK build artifacts
	cd sdk/go && go clean -cache -testcache

sdk-go: sdk-go-build sdk-go-test ## Build and test the Go SDK

# Docker build targets
docker-build-fsm: update_protos ## Build session manager Docker image
	docker build -t $(FSM_IMAGE):$(FSM_TAG) -f $(FSM_DOCKERFILE) .
	docker tag $(FSM_IMAGE):$(FSM_TAG) $(FSM_IMAGE):latest

docker-build-fem: update_protos ## Build executor manager Docker image
	docker build -t $(FEM_IMAGE):$(FEM_TAG) -f $(FEM_DOCKERFILE) .
	docker tag $(FEM_IMAGE):$(FEM_TAG) $(FEM_IMAGE):latest

docker-build-console: update_protos ## Build console Docker image
	docker build -t $(CONSOLE_IMAGE):$(CONSOLE_TAG) -f $(CONSOLE_DOCKERFILE) .

docker-build: docker-build-fsm docker-build-fem docker-build-console ## Build all Docker images

# Docker push targets
docker-push-fsm: docker-build-fsm ## Push session manager Docker image
	docker push $(FSM_IMAGE):$(FSM_TAG)
	docker push $(FSM_IMAGE):latest

docker-push-fem: docker-build-fem ## Push executor manager Docker image
	docker push $(FEM_IMAGE):$(FEM_TAG)
	docker push $(FEM_IMAGE):latest

docker-push-console: docker-build-console ## Push console Docker image
	docker push $(CONSOLE_IMAGE):$(CONSOLE_TAG)

docker-push: docker-push-fsm docker-push-fem docker-push-console ## Push all Docker images

# Release targets
docker-release: init docker-build docker-push ## Build and push all images for release

ci-image: update_protos ## Build images for CI (without version tags)
	docker build -t $(FSM_IMAGE) -f $(FSM_DOCKERFILE) .
	docker build -t $(FEM_IMAGE) -f $(FEM_DOCKERFILE) .
	docker build -t $(CONSOLE_IMAGE) -f $(CONSOLE_DOCKERFILE) .

# Cleanup targets
docker-clean: ## Remove all flame Docker images
	docker rmi $(FSM_IMAGE):$(FSM_TAG) $(FSM_IMAGE):latest 2>/dev/null || true
	docker rmi $(FEM_IMAGE):$(FEM_TAG) $(FEM_IMAGE):latest 2>/dev/null || true
	docker rmi $(CONSOLE_IMAGE):$(CONSOLE_TAG) 2>/dev/null || true

docker-clean-all: ## Remove all Docker images and containers (use with caution)
	docker system prune -a -f

# Development targets
docker-run-fsm: docker-build-fsm ## Run session manager container
	docker run --rm -it $(FSM_IMAGE):latest

docker-run-fem: docker-build-fem ## Run executor manager container
	docker run --rm -it $(FEM_IMAGE):latest

docker-run-console: docker-build-console ## Run console container
	docker run --rm -it $(CONSOLE_IMAGE):latest

# Utility targets
docker-images: ## List all flame Docker images
	docker images | grep $(DOCKER_REGISTRY)/flame

docker-logs: ## Show logs for running flame containers
	docker ps | grep flame | awk '{print $$1}' | xargs -I {} docker logs {}

# Legacy targets for backward compatibility
docker-release-legacy: init ## Legacy release target (original implementation)
	docker build -t $(FSM_IMAGE):$(FSM_TAG) -f $(FSM_DOCKERFILE) .
	docker build -t $(FEM_IMAGE):$(FEM_TAG) -f $(FEM_DOCKERFILE) .

