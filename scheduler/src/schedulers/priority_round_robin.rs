use std::collections::VecDeque;
use crate::{Pid, Process, ProcessState, Scheduler, SchedulingDecision, StopReason, SyscallResult,};
use std::num::NonZeroUsize;

#[derive(Clone)]
pub struct ProcessInfo {
    pid: Pid,
    state: ProcessState,
    timings: (usize, usize, usize),
    sleep_time: usize,
    has_sleep_time: bool,
    priority: i8,
    extra: String,
}

impl Process for ProcessInfo {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn state(&self) -> ProcessState {
        self.state
    }

    fn priority(&self) -> i8 {
        self.priority
    }

    fn timings(&self) -> (usize, usize, usize) {
        self.timings
    }

    fn extra(&self) -> String {
        self.extra.clone()
    }
}
pub struct PriorityRoundRobin {
    pub processes: VecDeque<ProcessInfo>,
    pub syscall: bool,
    pub fork_syscall: bool,
    pub timeslice: NonZeroUsize,
    pub remaining: NonZeroUsize,
    pub minimum_remaining_timeslice: usize,
}

impl PriorityRoundRobin {
    fn sleep(&mut self, duration: usize, remaining:usize) -> SyscallResult {
        let time = self.timeslice.get() - remaining;
        // If there is at least one process in the queue, update its timings
        if let Some(first_process) = self.processes.front_mut() {
            first_process.timings.0 += time;
            first_process.timings.2 += time;

            // For the remaining processes, add the given time to the total time
            for process in self.processes.iter_mut().skip(1) {
                process.timings.0 += time;
            }
        }
        if let Some(mut process) = self.processes.pop_front() {
            process.state = ProcessState::Waiting{ event: None };
            process.timings.2 -= 1;
            process.timings.0 -= 1;
            self.syscall = true;
            self.fork_syscall = false;
            process.sleep_time = duration;
            self.processes.push_back(process.clone());
            SyscallResult::Success
        } else {
            SyscallResult::NoRunningProcess
        }
    }

    fn fork(&mut self, priority: i8, remaining: usize) -> SyscallResult {
        let pid = Pid::new(self.processes.len() + 1);
        let state = if self.processes.is_empty() {
            ProcessState::Running
        } else {
            ProcessState::Ready
        };
        let process = ProcessInfo {
            pid,
            state,
            timings: (0, 0, 0),
            sleep_time: 0,
            has_sleep_time: false,
            priority,
            extra: String::new(),
        };

        if !self.processes.is_empty() {
            self.syscall = true;
            self.fork_syscall = true;
        }
        if remaining != 0 {
            self.remaining = NonZeroUsize::new(remaining).unwrap();
        }
        self.processes.push_back(process.clone());
        SyscallResult::Pid(process.pid())
    }

    fn wait(&mut self, pid_to_wait: usize) -> SyscallResult {
    
        for process in &mut self.processes {
            if process.pid() == pid_to_wait {
                process.state = ProcessState::Waiting { event: Some(pid_to_wait) };
            }
        }
        SyscallResult::Success
    }

    fn signal(&mut self, processid:usize) -> SyscallResult { 
        for process in &mut self.processes {
            if process.pid().as_usize() == processid {
                match process.state {
                    ProcessState::Waiting { event: Some(waiting_pid) } if waiting_pid == processid => {
                        // Procesul este în stare de așteptare după semnal și PID-ul corespunde
                        process.state = ProcessState::Ready;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        SyscallResult::Success
    }
}


impl Scheduler for PriorityRoundRobin {
    /// Processes the next process scheduling decision.
    fn next(&mut self) -> SchedulingDecision {
        // Checks if process 1 is active and there are processes in the queue.
        if !self.processes.iter().any(|process| process.pid() == Pid::new(1)) && !self.processes.is_empty() {
            return SchedulingDecision::Panic; // Returns Panic if process 1 is not active and there are processes.
        }

        // If there's an active syscall, increments the syscall timing.
        if self.syscall {
            // Get the length of the processes queue
            let len = self.processes.len();

            if self.fork_syscall {
                // Iterate through each process in the queue along with its index
                for (index, process) in self.processes.iter_mut().enumerate() {
                    // Check if the current process is not the last one in the queue
                    if index < len - 1 {
                        // Increment the total time for the current process
                        process.timings.0 += 1;
                    }
                }
            } else {
                // Iterate through each process in the queue along with its index
                for (_index, process) in self.processes.iter_mut().enumerate() {
                    process.timings.0 += 1;
                }
            }
        }

        // Tries to process the next process in the queue.
        if let Some(mut process) = self.processes.pop_front() {
            print!("Exec_time: {}", process.timings.2);
            // If there's an active syscall for the process, increments the syscall timing and deactivates the syscall.
            if self.syscall {
                process.timings.1 += 1;
                self.syscall = false;
                self.fork_syscall = false;
            }

            if process.has_sleep_time {
                println!("Sleep time: {}", process.sleep_time);
                for process2 in &mut self.processes {
                    process2.timings.0 += process2.sleep_time;
                }
                process.timings.0 += process.sleep_time;
                process.has_sleep_time = false;
                process.state = ProcessState::Ready;
            }

            if(process.state == ProcessState::Waiting { event: None }) {
                process.has_sleep_time = true;
                self.processes.push_front(process.clone());
                return SchedulingDecision::Sleep(NonZeroUsize::new(process.sleep_time).unwrap());
            }

            // Checks if there's minimum wait time to allow the process to execute.
            if self.remaining.get() >= self.minimum_remaining_timeslice {
                process.state = ProcessState::Running;
                self.processes.push_front(process.clone());
                return SchedulingDecision::Run {
                    pid: process.pid(),
                    timeslice: self.remaining,
                }; // Returns Run for the current process.
            }

            // Updates the remaining time to the timeslice and sets the process state to Ready.
            self.remaining = self.timeslice;
            process.state = ProcessState::Ready;

            // Checks if there's a second process in the queue to execute.
            if let Some(mut second_process) = self.processes.pop_front() {
                second_process.state = ProcessState::Running;
                self.processes.push_back(process.clone());
                self.processes.push_front(second_process.clone());
                return SchedulingDecision::Run {
                    pid: second_process.pid(),
                    timeslice: self.timeslice,
                }; // Returns Run for the second process.
            } else {
                // If there's no second process, runs the current process and returns the Run decision.
                process.state = ProcessState::Running;
                self.processes.push_front(process.clone());
                return SchedulingDecision::Run {
                    pid: process.pid(),
                    timeslice: self.remaining,
                };
            }
        }

        // If there are no more processes to process, returns Done.
        SchedulingDecision::Done
    }
    
    fn stop(&mut self, reason: StopReason) -> SyscallResult {
        match reason {
            StopReason::Syscall { syscall, remaining } =>
            match syscall {
                crate::Syscall::Fork(priority) => {
                    self.fork(priority, remaining)
                }

                crate::Syscall::Sleep(duration) => {
                    self.syscall = true;
                    self.sleep(duration, remaining)
                    
                }

                crate::Syscall::Wait(pid_to_wait) => {
                    self.syscall = true;
                    self.wait(pid_to_wait)
                    
                }

                crate::Syscall::Signal(pid) => {
                    self.syscall = true;
                    self.signal(pid)
                }

                crate::Syscall::Exit => {
                    let time = self.remaining.get() - remaining;
                    // If there is at least one process in the queue, update its timings
                    if let Some(first_process) = self.processes.front_mut() {
                        first_process.timings.0 += time;
                        first_process.timings.2 += time;

                        // For the remaining processes, add the given time to the total time
                        for process in self.processes.iter_mut().skip(1) {
                            process.timings.0 += time;
                        }
                    }
                    self.processes.pop_front();
                    SyscallResult::Success
                }
            },
            StopReason::Expired => {
                let time = self.remaining.get();
                    // If there is at least one process in the queue, update its timings
                    if let Some(first_process) = self.processes.front_mut() {
                        first_process.timings.0 += time;
                        first_process.timings.2 += time;

                        // For the remaining processes, add the given time to the total time
                        for process in self.processes.iter_mut().skip(1) {
                            process.timings.0 += time;
                        }
                    }
                if let Some(mut process) = self.processes.pop_front() {
                    process.state = ProcessState::Ready;
                    self.remaining = self.timeslice;
                    self.processes.push_back(process);
                }
                SyscallResult::Success
            }
        }
    }

    fn list(&mut self) -> Vec<&dyn Process> {
        //println!("intra in list");
        self.processes.iter().map(|x| x as &dyn Process).collect()
    }
}