.PHONY: help install test clean build-protos format lint

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install the package in development mode
	pip install -e .[dev]

test: ## Run tests
	pytest test_flame.py -v

clean: ## Clean up generated files
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
	find . -name "*.egg-info" -type d -exec rm -rf {} + 2>/dev/null || true

build-protos: ## Generate protobuf Python files
	python build_protos.py

format: ## Format code with black and isort
	black flame/ test_flame.py example.py
	isort flame/ test_flame.py example.py

lint: ## Run linting checks
	black --check flame/ test_flame.py example.py
	isort --check-only flame/ test_flame.py example.py
	mypy flame/

check: format lint test ## Run all checks (format, lint, test)

dist: clean ## Build distribution package
	python setup.py sdist bdist_wheel

install-deps: ## Install development dependencies
	pip install -r requirements-dev.txt || pip install pytest black isort mypy grpcio grpcio-tools protobuf 