//! Process management syscalls
use crate::mm::MapPermission;
use crate::timer::get_time_us;
use crate::{
    mm::{translated_byte_buffer, PageTable, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TASK_MANAGER,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

const PROT_READ: usize = 1;
const PROT_WRITE: usize = 2;
const PROT_EXEC: usize = 4;

const READ: usize = 0;
const WRITE: usize = 1;
const SELECT: usize = 2;

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();

    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let src: *const u8 = unsafe { core::mem::transmute(&time_val) };
    let len = core::mem::size_of::<TimeVal>();
    let buffers = translated_byte_buffer(current_user_token(), ts as *const u8, len);
    let mut offset = 0;
    for buffer in buffers {
        buffer
            .copy_from_slice(unsafe { core::slice::from_raw_parts(src.add(offset), buffer.len()) });
        offset += buffer.len();
    }
    0
}

/// TODO: Finish sys_trace to pass testcases start 2025.3.30
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    if trace_request == READ || trace_request == WRITE {
        let addr = VirtAddr::from(id);
        let page_table = PageTable::from_token(current_user_token());
        let pte = page_table.translate(addr.floor());
        if pte.is_none() {
            return -1;
        }
        
        let pte = pte.unwrap();
        if !pte.is_valid() || !pte.is_user(){
            return -1;
        }
        
        if trace_request == READ && pte.readable() {
            let res = pte.ppn().get_bytes_array()[addr.page_offset()];
            res as isize
        } else if trace_request == WRITE && pte.writable() {
            let target_ptr = &mut pte.ppn().get_bytes_array()[addr.page_offset()];
            *target_ptr = data as u8;
            0
        } else {
            -1
        }
    } else if trace_request == SELECT {
        TASK_MANAGER.get_current_syscall(id) as isize
    } else {
        trace!("Unsupported trace request: {}", trace_request);
        -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if (prot & !0x7 )!=0 || (prot & 0x7)==0{
        return -1;
    }
    let mut permission = MapPermission::empty();
    if (prot&PROT_READ)==PROT_READ {
        permission |= MapPermission::R;
    }
    if (prot&PROT_WRITE)==PROT_WRITE {
        permission |= MapPermission::W;
    }
    if (prot&PROT_EXEC)==PROT_WRITE {
        permission |= MapPermission::X;
    }
    

    if start%4096!=0{
        return -1;
    }

    if let Err(_msg) = TASK_MANAGER.mmap(start.into(), (start + len).into(), permission) {
        return -1;
    }
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % 4096 != 0 || len % 4096 != 0 {
        return -1;
    }

    if let Err(_msg) = TASK_MANAGER.unmap(start.into(), (start + len).into()) {
        return -1;
    }
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
