#![no_std]
#![deny(unsafe_code)]

//! Bare-metal-compatible RTOS scheduling primitives for governed agents.
//!
//! This crate intentionally avoids allocation, threads, async runtimes, and OS
//! syscalls. Platform ports can call `advance_ticks` from a hardware timer ISR
//! and `pop_next` from the cooperative or preemptive dispatch loop.

#[cfg(test)]
extern crate std;

/// Lower numeric value means higher scheduling priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum RtPriority {
    /// Emergency stops, motor safety, and fail-safe actions.
    Critical = 0,
    /// Sensor polling and actuator commands.
    High = 1,
    /// Standard governed agent operations.
    Normal = 2,
    /// Background analytics and governance maintenance.
    Low = 3,
    /// Cleanup and non-essential telemetry.
    Idle = 4,
}

/// A fixed-size real-time task descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RtTask {
    /// Platform-local task identifier.
    pub id: u32,
    /// Static scheduling priority.
    pub priority: RtPriority,
    /// Absolute deadline in scheduler ticks.
    pub deadline_ticks: u64,
    /// Worst-case execution time estimate in ticks.
    pub wcet_ticks: u32,
}

impl RtTask {
    /// Create a task descriptor.
    pub const fn new(id: u32, priority: RtPriority, deadline_ticks: u64, wcet_ticks: u32) -> Self {
        Self {
            id,
            priority,
            deadline_ticks,
            wcet_ticks,
        }
    }
}

/// Scheduler errors that can be handled without heap allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerError {
    /// No free static task slot exists.
    Full,
    /// A task with the same id is already queued.
    DuplicateTaskId,
}

/// Fixed-capacity scheduler suitable for bare-metal ports.
pub struct StaticScheduler<const N: usize> {
    tasks: [Option<RtTask>; N],
    queued: usize,
    tick: u64,
    missed_deadlines: u32,
}

impl<const N: usize> StaticScheduler<N> {
    /// Create an empty scheduler.
    pub const fn new() -> Self {
        Self {
            tasks: [None; N],
            queued: 0,
            tick: 0,
            missed_deadlines: 0,
        }
    }

    /// Queue a task.
    pub fn submit(&mut self, task: RtTask) -> Result<(), SchedulerError> {
        if self
            .tasks
            .iter()
            .flatten()
            .any(|queued| queued.id == task.id)
        {
            return Err(SchedulerError::DuplicateTaskId);
        }

        for slot in &mut self.tasks {
            if slot.is_none() {
                *slot = Some(task);
                self.queued += 1;
                return Ok(());
            }
        }

        Err(SchedulerError::Full)
    }

    /// Return the next task without removing it.
    pub fn peek_next(&self) -> Option<RtTask> {
        self.best_slot().and_then(|index| self.tasks[index])
    }

    /// Pop the next task by priority, then earliest deadline, then stable id.
    pub fn pop_next(&mut self) -> Option<RtTask> {
        let index = self.best_slot()?;
        let task = self.tasks[index].take()?;
        self.queued -= 1;
        if self.tick > task.deadline_ticks {
            self.missed_deadlines = self.missed_deadlines.saturating_add(1);
        }
        Some(task)
    }

    /// Advance scheduler time in platform ticks.
    pub fn advance_ticks(&mut self, ticks: u64) {
        self.tick = self.tick.saturating_add(ticks);
    }

    /// Current scheduler tick.
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Number of queued tasks.
    pub const fn queued(&self) -> usize {
        self.queued
    }

    /// Number of tasks popped after their deadline.
    pub const fn missed_deadlines(&self) -> u32 {
        self.missed_deadlines
    }

    fn best_slot(&self) -> Option<usize> {
        let mut best: Option<(usize, RtTask)> = None;

        for (index, task) in self.tasks.iter().enumerate() {
            let Some(task) = task else {
                continue;
            };

            match best {
                None => best = Some((index, *task)),
                Some((_, current)) if is_better(*task, current) => best = Some((index, *task)),
                Some(_) => {}
            }
        }

        best.map(|(index, _)| index)
    }
}

impl<const N: usize> Default for StaticScheduler<N> {
    fn default() -> Self {
        Self::new()
    }
}

const fn is_better(candidate: RtTask, current: RtTask) -> bool {
    let candidate_priority = candidate.priority as u8;
    let current_priority = current.priority as u8;

    if candidate_priority != current_priority {
        return candidate_priority < current_priority;
    }

    if candidate.deadline_ticks != current.deadline_ticks {
        return candidate.deadline_ticks < current.deadline_ticks;
    }

    candidate.id < current.id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_task_runs_before_lower_priority_tasks() {
        let mut scheduler: StaticScheduler<4> = StaticScheduler::new();
        scheduler
            .submit(RtTask::new(1, RtPriority::Low, 10, 1))
            .unwrap();
        scheduler
            .submit(RtTask::new(2, RtPriority::Critical, 20, 1))
            .unwrap();
        scheduler
            .submit(RtTask::new(3, RtPriority::Normal, 5, 1))
            .unwrap();

        assert_eq!(scheduler.pop_next().unwrap().id, 2);
        assert_eq!(scheduler.pop_next().unwrap().id, 3);
        assert_eq!(scheduler.pop_next().unwrap().id, 1);
    }

    #[test]
    fn earliest_deadline_wins_within_same_priority() {
        let mut scheduler: StaticScheduler<3> = StaticScheduler::new();
        scheduler
            .submit(RtTask::new(1, RtPriority::High, 30, 1))
            .unwrap();
        scheduler
            .submit(RtTask::new(2, RtPriority::High, 10, 1))
            .unwrap();

        assert_eq!(scheduler.pop_next().unwrap().id, 2);
        assert_eq!(scheduler.pop_next().unwrap().id, 1);
    }

    #[test]
    fn capacity_and_duplicate_ids_are_rejected() {
        let mut scheduler: StaticScheduler<1> = StaticScheduler::new();
        let task = RtTask::new(7, RtPriority::Normal, 10, 1);

        assert_eq!(scheduler.submit(task), Ok(()));
        assert_eq!(scheduler.submit(task), Err(SchedulerError::DuplicateTaskId));
        assert_eq!(
            scheduler.submit(RtTask::new(8, RtPriority::Normal, 10, 1)),
            Err(SchedulerError::Full)
        );
    }

    #[test]
    fn missed_deadline_is_counted_when_task_is_popped_late() {
        let mut scheduler: StaticScheduler<1> = StaticScheduler::new();
        scheduler
            .submit(RtTask::new(1, RtPriority::Critical, 5, 1))
            .unwrap();
        scheduler.advance_ticks(6);

        assert_eq!(scheduler.pop_next().unwrap().id, 1);
        assert_eq!(scheduler.missed_deadlines(), 1);
    }
}
