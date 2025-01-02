FSM_TAG=`cargo get --entry session_manager/ package.version --pretty`
FEM_TAG=`cargo get --entry executor_manager/ package.version --pretty`

all: docker-release

init:
	cargo install cargo-get

docker-release: init
	sudo docker build -t xflops/flame-session-manager:${FSM_TAG} -f docker/Dockerfile.fsm .
	sudo docker build -t xflops/flame-executor-manager:${FEM_TAG} -f docker/Dockerfile.fem .
