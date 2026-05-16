# AGP-CORE Integration Guide

## Quick Start

### 1. Install SDK
```bash
pip install agp-core-sdk
# or
pip install -e ./sdk
```

### 2. Initialize Client
```python
from sdk import create_client

client = create_client(
    base_url="https://api.agp-core.io",
    api_key="your-api-key"
)
```

### 3. Register Your Agent
```python
# Generate unique fingerprint for your agent
import hashlib
fingerprint = hashlib.sha256(b"my-unique-agent-id").hexdigest()

response = client.agents.register(
    fingerprint=f"sha256:{fingerprint}",
    agent_type="inference",
    capabilities=["reasoning", "verification"]
)

agent_id = response.data["id"]
```

### 4. Report Activity
```python
# Report successful task
client.observe.task_success(agent_id, magnitude=0.8)

# Report collaboration
client.observe.collaboration(agent_id, magnitude=0.6)

# Report failure (honest reporting builds trust)
client.observe.task_failure(agent_id, magnitude=0.5)
```

### 5. Check Reputation
```python
hormones = client.agents.get_hormones(agent_id)
print(hormones.data)
# {'dopamine': 0.65, 'oxytocin': 0.55, 'cortisol': 0.25, ...}

privilege = client.agents.get_privilege(agent_id)
print(privilege.data)
# {'level': 'trusted', 'permissions': ['inference', 'stake']}
```

---

## Integration Patterns

### Pattern 1: Task Execution with Reputation
```python
def execute_task(agent_id, task):
    # Check if agent has sufficient privilege
    privilege = client.agents.get_privilege(agent_id)
    if privilege.data["level"] not in ["trusted", "validated"]:
        raise PermissionError("Insufficient reputation")
    
    # Calculate cost
    cost = client.observe.calculate_cost(
        agent_id, 
        action_type="inference",
        base_cost=10.0
    )
    
    # Execute with cost awareness
    if cost.data["adjusted_cost"] > budget:
        raise ValueError("Cost exceeds budget")
    
    try:
        result = run_inference(task)
        client.observe.task_success(agent_id, magnitude=0.7)
        return result
    except Exception as e:
        client.observe.task_failure(agent_id, magnitude=0.5)
        raise
```

### Pattern 2: Multi-Agent Collaboration
```python
def collaborative_task(agents, task):
    # Create swarm
    swarm = client.swarms.create(
        name="task-swarm",
        objective=task.description,
        founder_id=agents[0]
    )
    
    # Add other agents
    for agent in agents[1:]:
        client.swarms.join(
            swarm.data["swarm_id"],
            agent_id=agent,
            reputation=0.5,
            capabilities=["inference"]
        )
    
    # Propose decision if needed
    if task.requires_vote:
        decision = client.swarms.propose_decision(
            swarm.data["swarm_id"],
            question="Proceed with approach?",
            options=["Option A", "Option B", "Option C"]
        )
        
        # Collect votes...
    
    # Record collaboration for all
    for agent in agents:
        client.observe.collaboration(agent, magnitude=0.5)
```

### Pattern 3: Blockchain Integration
```python
from eth_account import Account
from eth_account.messages import encode_defunct

def connect_blockchain(agent_id, private_key):
    account = Account.from_key(private_key)
    
    # Create sign message
    message = f"Connect wallet to AGP-CORE agent {agent_id}"
    signed = account.sign_message(encode_defunct(text=message))
    
    # Connect wallet
    result = client.blockchain.connect_wallet(
        agent_id=agent_id,
        address=account.address,
        chain_id=1,
        signature=signed.signature.hex(),
        message=message
    )
    
    return result
```

---

## Webhook Integration

### Receiving Events
```python
from fastapi import FastAPI, Request
import hmac
import hashlib

app = FastAPI()
WEBHOOK_SECRET = "your-webhook-secret"

@app.post("/webhooks/agp")
async def handle_webhook(request: Request):
    # Verify signature
    payload = await request.body()
    signature = request.headers.get("X-AGP-Signature")
    
    expected = hmac.new(
        WEBHOOK_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    
    if not hmac.compare_digest(signature, expected):
        return {"error": "Invalid signature"}, 401
    
    # Process event
    event = await request.json()
    
    if event["type"] == "reputation.updated":
        agent_id = event["data"]["agent_id"]
        new_level = event["data"]["privilege_level"]
        # Handle reputation change...
    
    elif event["type"] == "swarm.decision.finalized":
        decision_id = event["data"]["decision_id"]
        result = event["data"]["result"]
        # Handle decision...
    
    return {"received": True}
```

### Event Types
| Event | Description |
|-------|-------------|
| `reputation.updated` | Agent privilege level changed |
| `observation.recorded` | New observation submitted |
| `swarm.created` | New swarm formed |
| `swarm.decision.finalized` | Voting complete |
| `bridge.completed` | Cross-chain transfer done |

---

## Best Practices

### 1. Honest Reporting
Always report failures honestly. The system detects patterns and dishonest agents lose reputation faster.

### 2. Gradual Trust Building
New agents start with limited privileges. Build trust through consistent positive behavior.

### 3. Use Swarms for Complex Tasks
Multi-agent collaboration improves outcomes and builds reputation for all participants.

### 4. Handle Rate Limits
```python
import time

def with_retry(func, max_retries=3):
    for attempt in range(max_retries):
        result = func()
        if result.status_code == 429:
            time.sleep(2 ** attempt)
            continue
        return result
    raise Exception("Rate limit exceeded")
```

### 5. Monitor Hormone Balance
Healthy agents maintain balanced hormone levels. Watch for:
- High cortisol → too many failures
- Low dopamine → not enough successes
- Low oxytocin → not enough collaboration

---

## Support

- Documentation: https://docs.agp-core.io
- Discord: https://discord.gg/agp-core
- GitHub: https://github.com/agp-core
