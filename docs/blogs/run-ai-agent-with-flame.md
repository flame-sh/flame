# Building AI Agents with Flame

The landscape of AI agent development is evolving quickly. Many frameworks promise rapid prototyping and production-ready deployments, yet teams still face familiar challenges: high task latency, suboptimal resource usage, and awkward integration patterns. Because agent workloads are inherently elastic, Flame is a natural fit to address these challenges.

## What Makes Flame Different?

Elastic workloads demand parallelism, efficient data sharing, and fast round-trips. Unlike batch jobs, they don’t rely on heavy inter-task communication. By introducing the concepts of Session, Application, and Executor, Flame provides a distributed computing platform tailored to elastic workloads at scale—such as AI agents.

### 1. Session-Based Isolation

In Flame, a Session is a group of tasks for an elastic workload. Clients can keep creating tasks until the session is closed. Within a session, tasks reuse the same executor to avoid cold starts and to share session-scoped data.

With session-based isolation:
- Data is not shared across sessions
- Executors are not reused across sessions, preventing data leakage

This makes it straightforward for agent frameworks like LangChain and CrewAI to support multi-tenancy and data isolation using Flame.

The following code demonstrates creating a session and submitting a task to it. When creating a session via `flame.create_session`, the client identifies the application (agent) to use and provides common data shared by all tasks in the session. We’ll introduce how to build and deploy an application in Flame in the following section.

```python
# Each session is completely isolated
session = await flame.create_session("openai-agent", b"You are a weather forecaster.")

# First task of the session
task = await session.invoke(b"Who are you?")
print(task.output)
```

### 2. Zero Cold Starts

Thanks to sessions, the executor stays warm for subsequent tasks in the same session, avoiding cold start latencies. If a session becomes idle, the executor remains available for a short period to absorb bursts; it’s released when the session closes or when the delayed-release timeout expires.

In addition to mapping one session to a single executor instance, Flame can scale out to multiple executors per session to increase parallelism when needed.

### 3. Elegantly Simple Python API

Flame’s Python SDK is designed with developer experience in mind. Building an AI agent typically boils down to three methods that map to session lifecycle events:

```python
class MyAgent(flame.FlameService):
    async def on_session_enter(self, context: flame.SessionContext):
        # Initialize your agent (runs once per session)
        pass
    
    async def on_task_invoke(self, context: flame.TaskContext) -> flame.TaskOutput:
        # Process individual tasks
        pass
    
    async def on_session_leave(self):
        # Clean up resources
        pass
```

API overview:
- **`on_session_enter`**: Ideal for expensive, once-per-session initialization
- **`on_task_invoke`**: Handles individual requests with full session context (session ID, task ID, credentials/delegations)
- **`on_session_leave`**: Cleans up resources when a session ends

Client-side APIs are equally simple. In addition to the synchronous example above, Flame supports asynchronous APIs. With a callback-based informer, the client receives state change notifications (e.g., pending, running).

```python
class MyAgentTaskInformer(flame.TaskInformer):
    def on_update(self, task: flame.Task):
        pass
    
    def on_error(self):
        pass

# Each session is completely isolated
session = await flame.create_session("openai-agent", b"You are a weather forecaster.")

informer = MyAgentTaskInformer()

await session.invoke(b"Who are you?", informer)
```

As a distributed system, Flame schedules tasks onto executors according to the scheduler’s algorithm. Client calls like `flame.create_session` and `session.invoke` enqueue work and don’t synchronously trigger server-side execution. Executors pick up work when scheduled.

### 4. Universal Integration: Framework-Agnostic

Flame’s general-purpose APIs integrate seamlessly with existing AI frameworks and tools. Whether you’re using LangChain, CrewAI, AutoGen, or others, Flame provides the execution layer:

```python
# LangChain Integration Example
from langchain.agents import create_openai_functions_agent, AgentExecutor
from langchain.chat_models import ChatOpenAI

class LangChainAgent(FlameService):
    async def on_session_enter(self, context: SessionContext):
        llm = ChatOpenAI(temperature=0)
        self.agent = create_openai_functions_agent(llm, tools, prompt)
        self.agent_executor = AgentExecutor(agent=self.agent, tools=tools)
    
    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        result = await self.agent_executor.ainvoke({
            "input": context.input.decode("utf-8")
        })
        return TaskOutput(data=result["output"].encode("utf-8"))
```

This flexibility means:
- **No vendor lock-in**: Use your preferred AI libraries and models
- **Gradual migration**: Adopt Flame incrementally within existing projects
- **Best of both worlds**: Combine Flame’s infrastructure benefits with your favorite AI tools

## Real-World Example: OpenAI Agent

Let’s walk through a complete example that demonstrates Flame’s capabilities.

### The Agent Implementation

This example uses the OpenAI Python SDK to chat with DeepSeek:
- In `on_session_enter`, it reads the API key and creates a client for DeepSeek; the session’s common data acts as the system prompt
- In `on_task_invoke`, it combines the system prompt with the task input (user prompt), calls DeepSeek, and returns the response as the task output
- In `on_session_leave`, no cleanup is required for this example

Flame guarantees that `on_task_invoke` runs only after a successful `on_session_enter`. If `on_session_enter` fails (after retries), the session fails. Similarly, a task fails if `on_task_invoke` raises an error. Clients receive task status notifications, and a failed task does not affect other tasks in the session.

```python
import os
import asyncio
import flame
from flame import FlameService, SessionContext, TaskContext, TaskOutput
from openai import OpenAI

class OpenAIAgent(FlameService):
    def __init__(self):
        self.client = None
        self.system_prompt = None

    async def on_session_enter(self, context: SessionContext):
        # Initialize OpenAI client once per session
        self.client = OpenAI(
            api_key=os.getenv("DEEPSEEK_API_KEY"),
            base_url="https://api.deepseek.com"
        )
        self.system_prompt = context.common_data.decode("utf-8")

    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        response = self.client.chat.completions.create(
            model="deepseek-chat",
            messages=[
                {"role": "system", "content": self.system_prompt},
                {"role": "user", "content": context.input.decode("utf-8")}
            ]
        )
        return TaskOutput(data=response.choices[0].message.content.encode("utf-8"))

    async def on_session_leave(self):
        # Clean up if needed
        pass

# Run the agent
if __name__ == "__main__":
    asyncio.run(flame.run(OpenAIAgent()))
```

### Deploy the Agent

After building the agent, deploy it to Flame. The deployment configuration assigns a name to the agent so clients can create sessions by name. It also specifies the agent’s startup command (arguments, environment variables, etc.).

In this example, the agent is named `openai-agent` and uses `uv` to launch, simplifying dependency management. For simplicity, the example is mounted directly into the `flame-executor-manager` container. We’ll discuss more advanced deployments (e.g., microVM) in future posts.

```yaml
# openai-agent.yaml
metadata:
  name: openai-agent
spec:
  command: /usr/bin/uv
  arguments:
    - run
    - /opt/examples/agents/openai/main.py
  environments:
    DEEPSEEK_API_KEY: sk-xxxxxxxxxxxxxxxxx
```

Another benefit of `uv` is streamlined Python dependency management. `uv` supports declaring dependencies directly in script comments. This example uses the following to declare dependencies:

```python
# /// script
# dependencies = [
#   "openai",
#   "flame",
# ]
# [tool.uv.sources]
# flame = { path = "/usr/local/flame/sdk/python" }
# ///

# ... your script code ...
```

After preparing the deployment, use `flmctl` to register the agent:

```shell
$ flmctl register -f openai-agent.yaml
$ flmctl list -a
Name                Shim      State       Created        Command                       
flmping             Grpc      Enabled     03:36:51       /usr/local/flame/bin/flmping-service
flmexec             Grpc      Enabled     03:36:51       /usr/local/flame/bin/flmexec-service
flmtest             Log       Enabled     03:36:51       -                             
openai-agent        Grpc      Enabled     03:36:56       /usr/bin/uv                   
```

### The Client Usage

With the agent deployed, build a simple client to verify it. In this client, we create a session that asks the agent to act as a weather forecaster, then send a prompt asking the agent to introduce itself.

```python
import flame
import asyncio

async def main():
    # Create a session with initial context
    session = await flame.create_session(
        "openai-agent",
        b"You are a weather forecaster."
    )
    
    # Send a task to the same session
    task1 = await session.invoke(b"Who are you?")
    print(task1.output.decode("utf-8"))

if __name__ == "__main__":
    asyncio.run(main())
```

Run this client in a virtual environment on your desktop. You should see a response similar to the following:

```shell
(agent_example) $ python3 ./examples/agents/client.py 
I’m your friendly weather forecaster assistant! 🌦️ I can help you check current weather conditions, forecasts, or answer any weather-related questions you have—whether it’s about rain, snow, storms, or just deciding if you need a jacket today.  
 
Want a forecast for your location or somewhere else? Just let me know! (Note: For real-time data, I may need you to enable location access or specify a place.)  
 
How can I brighten your weather knowledge today? ☀️🌧️
```

## Next / Roadmap

This post introduced how to run an AI agent with Flame at a high level. Upcoming posts will cover topics including, but not limited to:

1. Running generated scripts via Flame
2. Resource management in Flame
3. Updating common data within a session
4. Security considerations
5. Observability and evaluation
6. Performance best practices
