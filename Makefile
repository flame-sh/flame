FSM_TAG=`cargo get --entry session_manager/ package.version --pretty`
FEM_TAG=`cargo get --entry executor_manager/ package.version --pretty`

PY_RPC_OUT=sdk/python/flame
RPC_DIR=rpc/protos

.PHONY: update_protos

build: update_protos
	cargo build

init:
	cargo install cargo-get --force

docker-release: init
	sudo docker build -t xflops/flame-session-manager:${FSM_TAG} -f docker/Dockerfile.fsm .
	sudo docker build -t xflops/flame-executor-manager:${FEM_TAG} -f docker/Dockerfile.fem .

ci-image:
	sudo docker build -t xflops/flame-session-manager -f docker/Dockerfile.fsm .
	sudo docker build -t xflops/flame-executor-manager -f docker/Dockerfile.fem .
	sudo docker build -t xflops/flame-console -f docker/Dockerfile.console .

update_protos:
	cp rpc/protos/frontend.proto sdk/rust/protos
	cp rpc/protos/types.proto sdk/rust/protos
	cp rpc/protos/shim.proto sdk/rust/protos

generate-grpc:
	for p in types.proto frontend.proto shim.proto ; do \
		python -m grpc_tools.protoc -I${RPC_DIR} --python_out=${PY_RPC_OUT} --pyi_out=${PY_RPC_OUT} --grpc_python_out=${PY_RPC_OUT} ${RPC_DIR}/$$p ; \
	done
