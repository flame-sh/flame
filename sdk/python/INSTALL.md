# Flame Python SDK Installation Guide

## Prerequisites

- Python 3.8 or higher
- pip (Python package installer)

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/flame-sh/flame.git
cd flame/sdk/python
```

2. Install in development mode:
```bash
pip install -e .
```

### Install Dependencies

Install development dependencies:
```bash
pip install -r requirements-dev.txt
```

Or install manually:
```bash
pip install grpcio grpcio-tools protobuf pytest black isort mypy
```

## Development Setup

### 1. Install in Development Mode

```bash
pip install -e .[dev]
```

### 2. Generate Protobuf Files

```bash
python build_protos.py
```

### 3. Run Tests

```bash
pytest test_flame.py -v
```

### 4. Format Code

```bash
black flame/ test_flame.py example.py
isort flame/ test_flame.py example.py
```

### 5. Run Linting

```bash
black --check flame/ test_flame.py example.py
isort --check-only flame/ test_flame.py example.py
mypy flame/
```

## Using Makefile

The SDK includes a Makefile for common development tasks:

```bash
# Show available commands
make help

# Install in development mode
make install

# Run tests
make test

# Format code
make format

# Run linting
make lint

# Run all checks
make check

# Clean up generated files
make clean

# Build distribution package
make dist
```

## Quick Start

1. Install the SDK:
```bash
pip install -e .
```

2. Run the example:
```bash
python example.py
```

3. Run tests:
```bash
pytest test_flame.py -v
```

## Project Structure

```
sdk/python/
├── flame/                    # Main SDK package
│   ├── __init__.py          # Public API
│   ├── types.py             # Type definitions and enums
│   ├── client.py            # Main client implementation
│   └── protos/              # Protobuf definitions
│       ├── __init__.py
│       ├── types.proto      # Type definitions
│       ├── frontend.proto   # Frontend service
│       ├── backend.proto    # Backend service
│       └── placeholder.py   # Placeholder for development
├── setup.py                 # Package setup
├── README.md                # SDK documentation
├── example.py               # Usage example
├── test_flame.py            # Tests
├── build_protos.py          # Protobuf generation script
├── Makefile                 # Development tasks
├── requirements-dev.txt     # Development dependencies
├── pyproject.toml          # Development configuration
└── docs/                   # Documentation
    └── API.md              # API reference
```

## Troubleshooting

### Common Issues

1. **Import errors**: Make sure you've installed the package in development mode:
   ```bash
   pip install -e .
   ```

2. **Protobuf generation errors**: Install grpcio-tools:
   ```bash
   pip install grpcio-tools
   ```

3. **Type checking errors**: Install mypy:
   ```bash
   pip install mypy
   ```

4. **Formatting errors**: Install black and isort:
   ```bash
   pip install black isort
   ```

### Development Tips

1. **Use virtual environment**: Create a virtual environment before installing:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   pip install -e .[dev]
   ```

2. **Pre-commit hooks**: Consider setting up pre-commit hooks for automatic formatting and linting.

3. **IDE support**: The SDK includes type hints for better IDE support. Make sure your IDE is configured to use the correct Python interpreter.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Format code: `make format`
6. Run linting: `make lint`
7. Submit a pull request

## License

This project is licensed under the Apache License, Version 2.0. 