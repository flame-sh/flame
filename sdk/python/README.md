# Flame Python SDK

Python SDK for the Flame distributed computing framework.

## Installation

```bash
pip install flame-sdk
```

## Quick Start

```python
import asyncio
from flame import Connection, SessionAttributes, Shim

async def main():
    # Connect to Flame service
    conn = await Connection.connect("http://localhost:8080")
    # Create a session
    session = await conn.create_session(SessionAttributes(
        application="flmlog",
        slots=2,
        common_data=b"shared data"
    ))
    
    # Create and run a task
    task = await session.create_task(b"task input data")
    
    # Watch task progress
    async for update in session.watch_task(task.id):
        print(f"Task {task.id}: {update.state}")
        if update.is_completed():
            break
    
    # Close session
    await session.close()
    await conn.close()

if __name__ == "__main__":
    asyncio.run(main())
```

## API Reference

### Connection

The main entry point for connecting to Flame services.

```python
from flame import Connection

# Connect to a Flame service
conn = await Connection.connect("http://localhost:8080")
```

### Session

Represents a computing session with distributed resources.

```python
# Create a session
session = await conn.create_session(SessionAttributes(
    application="my-app",
    slots=2
))

# List sessions
sessions = await conn.list_sessions()

# Close a session
await session.close()
```

### Task

Represents individual computing tasks within a session.

```python
# Create a task
task = await session.create_task(b"input data")

# Get task status
task = await session.get_task(task.id)

# Watch task progress
async for update in session.watch_task(task.id):
    print(f"Task state: {update.state}")
    if update.is_completed():
        break
```

### Application

Manage distributed applications.

```python
# Register an application
await conn.register_application("my-app", {
    "shim": Shim.SHELL,
    "command": "python",
    "arguments": ["script.py"]
})

# List applications
apps = await conn.list_applications()
```

## Error Handling

The SDK provides custom exception types for different error scenarios:

```python
from flame import FlameError, FlameErrorCode

try:
    session = await conn.create_session(attrs)
except FlameError as e:
    if e.code == FlameErrorCode.INVALID_CONFIG:
        print("Configuration error:", e.message)
    elif e.code == FlameErrorCode.INVALID_STATE:
        print("State error:", e.message)
```

## Development

To set up the development environment:

```bash
# Clone the repository
git clone https://github.com/flame-sh/flame.git
cd flame/sdk/python

# Install in development mode
pip install -e .[dev]

# Run tests
pytest

# Format code
black flame/
isort flame/

# Type checking
mypy flame/
``` 