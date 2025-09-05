.PHONY: help install test clean build-protos format lint

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install the package in development mode
	pip install -e .

test: ## Run tests
	uv run -n test/test_flame.py

clean: ## Clean up generated files
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
	find . -name "*.egg-info" -type d -exec rm -rf {} + 2>/dev/null || true

build-protos: ## Generate protobuf Python files
	python build_protos.py

format: ## Format code with black and isort
	black . test/test_flame.py
	isort . test/test_flame.py

lint: ## Run linting checks
	black --check . test/test_flame.py
	isort --check-only . test/test_flame.py
	mypy .

check: format lint test ## Run all checks (format, lint, test)

dist: clean ## Build distribution package
	python setup.py sdist bdist_wheel
