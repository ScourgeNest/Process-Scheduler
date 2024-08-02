# Round Robin Scheduler in Rust

This code implements a basic Round Robin Scheduler in Rust. It manages process execution using a Round Robin algorithm and handles various system call events. Below is an overview of the code structure:

## ProcessInfo Struct
- `ProcessInfo` represents information about a process, encompassing crucial data such as:
  - Process ID (`pid`)
  - Current state
  - Timings
  - Sleep time
  - Priority
  - Additional details
- Implements the `Process` trait to facilitate access to process information like `pid`, `state`, `priority`, `timings`, and extra details.

## RoundRobin Struct
- `RoundRobin` embodies the Round Robin scheduler and holds:
  - A queue of `ProcessInfo` instances (`processes`)
  - Syscall flags (`syscall` and `fork_syscall`)
  - Timeslice values (`timeslice` and `remaining`)
  - Minimum remaining timeslice
- Implements the `Scheduler` trait, offering methods (`next`, `stop`, and `list`) to handle scheduling decisions, process halting, and process listing.

## Scheduler Methods
- `next`: Processes the next scheduling decision, considering process states, syscalls, and timeslices. It manages various process states and syscalls like Fork, Sleep, Exit, Wait, and Signal. Includes checks for deadlocks within the processes.
- `stop`: Halts processes based on different stop reasons such as syscall types (Fork, Sleep, Exit, Wait, Signal) or the expiration of timeslices. Updates process timings and states accordingly.
- `list`: Provides a list of processes that adhere to the `Process` trait.

## Additional Notes
- Utilizes Rust's standard library along with custom implementations for scheduling and process handling.
- Executes a Round Robin scheduling approach, cycling through processes and allocating each a turn to execute based on predefined timeslices. It effectively manages various system call events while ensuring fair process execution.


# Priority Queue Scheduler in Rust

This code implements a priority queue-based scheduler in Rust, emphasizing process management based on priority levels. Below is a detailed overview of the implemented code:

## ProcessInfo Struct
- The `ProcessInfo` structure encapsulates crucial data regarding a process, including:
  - Process ID (`pid`)
  - Current state
  - Timings
  - Sleep time
  - Priority
  - Additional details
- Implements the `Process` trait, enabling methods to access process-related information like `pid`, `state`, `priority`, `timings`, and extra details.

## PriorityQueue Struct
- `PriorityQueue` represents the scheduler relying on a priority queue.
- Maintains a queue of `ProcessInfo` instances (`processes`), manages syscall flags (`syscall` and `fork_syscall`), handles timeslice values (`timeslice` and `remaining`), and stores the minimum remaining timeslice.
- Implements the `Scheduler` trait, offering functionalities like `next`, `stop`, and `list` to manage scheduling decisions, halt processes, and enlist processes.

## Scheduler Methods
- `next`: Manages the next scheduling decision, factoring in process states, syscalls, and priority levels.
- `stop`: Governs the stopping of processes based on various stop reasons such as syscall types (Fork, Sleep) or the exhaustion of timeslices. Updates process timings and states correspondingly.
- `list`: Offers a list of processes that adhere to the Process trait.

## Additional Insights
- Utilizes Rust's standard library in conjunction with custom implementations tailored for managing scheduling and handling processes.
- Operates on a priority queue model, managing processes by assigning precedence based on their specified priority levels. Processes with higher priorities receive precedence in execution.

This priority queue-based scheduler orchestrates processes by their assigned priority levels, ensuring higher priority processes are given precedence in execution while effectively managing various system call events. The code can be adapted or extended to suit specific requirements or serve as a learning resource.
