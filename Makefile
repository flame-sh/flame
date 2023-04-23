FLM_TAG=v0.2.0

docker-release:
	sudo docker build -t xflops/flame-session-manager:${FLM_TAG} -f docker/Dockerfile.fsm .
	sudo docker build -t xflops/flame-executor-manager:${FLM_TAG} -f docker/Dockerfile.fem .
