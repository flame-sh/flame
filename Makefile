FLM_TAG=v0.1.0

docker-release:
	sudo docker build -t flame-session-manager:${FLM_TAG} -f docker/Dockerfile.fsm .
	sudo docker build -t flame-executor-manager:${FLM_TAG} -f docker/Dockerfile.fem .
