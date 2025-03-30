//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;
/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task syscall count
    pub task_sys_count : [u32;MAX_SYSCALL_NUM]
}

impl TaskControlBlock {
    ///  inc syscall_id for current task 
    pub fn syscall_id(&mut self,syscall_id: usize){
        self.task_sys_count[syscall_id]+=1;
    }
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
