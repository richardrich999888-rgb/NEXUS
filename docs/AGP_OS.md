# AGP OS — Description

**AGP OS** (Agent Governance Platform Operating System) is the Python operating-system layer inside `agp-core`. It treats **agents as processes**: each agent has a process (a Process Control Block, or PCB), and the kernel manages when and whether that process is allowed to run. No agent gets CPU time without going through the kernel and, when enforcement is on, through the TELOS commitment membrane.

---

## What it is

AGP OS is a **micro-kernel for AI agents**. It provides:

1. **Process lifecycle** — Spawn a process for an agent, register it with TELOS so it can be authorized to execute, persist process state to a SQLite database, and kill or terminate processes.
2. **Scheduling** — Decide which runnable process runs next. Priority is driven by the agent’s **endocrine state** (e.g. dopamine, norepinephrine, cortisol): higher “drive” and lower stress tend to raise priority; high stress can throttle or deprioritize.
3. **Resource accounting** — Track token usage, CPU cycles, memory pages, and disk bytes per process, with quotas and throttling so no single agent can monopolize resources.

So in practice: you have many agents; each is a process. The kernel keeps a process table, recovers it from the DB on boot, runs a scheduler loop that picks the next process to run, and hands control to it only after a successful **TELOS crossing** (entropy, authority, and trust checks). That handoff is **execution** in AGP OS terms: the moment a process goes to RUNNING and gets its time slice.

---

## Main pieces

| Piece | What it does |
|-------|----------------|
| **BioKernel** (`agp-core/src/os/kernel.py`) | The kernel: process table, spawn/kill, scheduler loop, context switch. Boots, recovers state from DB, spawns System_Init if needed. |
| **ProcessControlBlock (PCB)** (`agp-core/src/os/process.py`) | One per agent: pid, agent_id, name, state (CREATED, READY, RUNNING, WAITING, SLEEPING, TERMINATED, ZOMBIE), priority, usage (tokens, cycles, pages, bytes), quota. Priority is computed from the agent’s endocrine state. |
| **context_switch()** (in kernel.py) | The only place the kernel sets a process to RUNNING. Before that, it calls TELOS `request_crossing(..., required_scope="execute:*")`. If the membrane says no, it raises `ExecutionBlocked` and does not set RUNNING. |
| **TELOS membrane** (`agp-core/src/telos/membrane.py`) | Checks entropy budget, authority scope, and (for high-consequence actions) trust. Used by `context_switch()` so that execution handoff is gated. |
| **Persistence** (`agp-core/src/os/persistence/database.py`) | SQLite-backed save/load of the process table so the kernel can recover after restart. |
| **IPC** (`agp-core/src/os/ipc/`) | Message queue, shared memory, and signals (e.g. SIGTERM, SIGSTOP, SIGCONT) between processes. |
| **FS** (`agp-core/src/os/fs/`) | Virtual filesystem: /proc (process info), /home, shared storage. |
| **Scheduler** (`agp-core/src/os/scheduler.py`) | Standalone “AdvancedScheduler” for selection and deadlock detection. The kernel does **not** use it for execution handoff; the kernel has its own `schedule()` loop that calls `context_switch()`. |

---

## How execution works

1. The kernel’s **scheduler loop** (`BioKernel.schedule()`) runs while the kernel is running.
2. It builds the list of runnable processes (READY or RUNNING), sorts by priority (endocrine-based), and takes the top one.
3. It calls **`context_switch(next_process)`**.
4. Inside `context_switch()`: build a `Decision`, call **`telos_membrane.request_crossing(decision, required_scope="execute:*")`**.
5. If the result is **not allowed**: raise **`ExecutionBlocked`** and return. The process does **not** go to RUNNING; `last_scheduled_at` is not updated.
6. If the result is **allowed**: set `pcb.state = ProcessState.RUNNING` and `pcb.last_scheduled_at = time.time()`. Execution handoff is done.

So **execution** in AGP OS is: the kernel chose this process, TELOS allowed the crossing, and the kernel marked it RUNNING and updated its last-scheduled time. There is no other path in the kernel that sets a process to RUNNING; the only path is through `context_switch()`, and that path always goes through TELOS first.

---

## Where it lives

- **Kernel and process:** `agp-core/src/os/kernel.py`, `agp-core/src/os/process.py`
- **TELOS gate:** `context_switch()` in `kernel.py` (lines ~179–199); `request_crossing()` in `agp-core/src/telos/membrane.py`
- **Rest of OS:** `agp-core/src/os/` — persistence, IPC, FS, scheduler, shell, syscalls, recovery, resilience, etc.

For enforcement and audit, see **EXECUTION_LAW.md**, **EXECUTION_ENFORCEMENT_AUDIT_REPORT.md** (§4), and **REGULATOR_GRADE_EXECUTION_TESTS.md**.
