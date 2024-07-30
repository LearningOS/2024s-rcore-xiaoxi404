use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec::Vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    //println!("sys_mutex_create");
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_available_vec[id] = Some(1);
        process_inner
            .mutex_allocation_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap()[id] = Some(0));
        process_inner
            .mutex_request_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap()[id] = Some(0));
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_available_vec.push(Some(1));
        process_inner
            .mutex_allocation_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap().push(Some(0)));
        process_inner
            .mutex_request_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap().push(Some(0)));
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    //println!("sys_mutex_lock 1");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    *process_inner.mutex_request_matrix[tid].as_mut().unwrap()[mutex_id]
        .as_mut()
        .unwrap() += 1;
    if process_inner.is_enable_deadlock_detect {
        /* if 1 > 1 - process_inner.mutex_allocation_matrix[tid].as_mut().unwrap()[mutex_id].unwrap() {
            return -0xDEAD;
        }
        if 1 > process_inner.mutex_available_vec[mutex_id].unwrap() {
            return -0xDEAD;
        } */

        /*  *process_inner.mutex_available_vec[mutex_id]
            .as_mut()
            .unwrap() -= 1;
        *process_inner.mutex_allocation_matrix[tid].as_mut().unwrap()[mutex_id]
            .as_mut()
            .unwrap() += 1; */

        let mut work = process_inner.mutex_available_vec.clone();
        let mut finish: Vec<_> = process_inner
            .tasks
            .iter()
            .map(|x| match x {
                Some(_) => false,
                None => true,
            })
            .collect();

        let mut i: usize = 0;
        'out: while i < finish.len() {
            if !finish[i] {
                for j in 0..work.len() {
                    if let Some(v) = work[j] {
                        if v < process_inner.mutex_request_matrix[i].as_ref().unwrap()[j].unwrap() {
                            i += 1;
                            continue 'out;
                        }
                    }
                }
                for j in 0..work.len() {
                    if let Some(v) = work[j].as_mut() {
                        *v +=
                            process_inner.mutex_allocation_matrix[i].as_ref().unwrap()[j].unwrap();
                    }
                }
                i = 0;
                finish[i] = true;
            }
            i += 1;
        }
        if finish.iter().any(|&x| !x) {
            *process_inner.mutex_request_matrix[tid].as_mut().unwrap()[mutex_id]
                .as_mut()
                .unwrap() -= 1;
            return -0xDEAD;
        }
    }

    drop(process_inner);
    mutex.lock();

    let mut process_inner = process.inner_exclusive_access();
    *process_inner.mutex_request_matrix[tid].as_mut().unwrap()[mutex_id]
        .as_mut()
        .unwrap() -= 1;
    *process_inner.mutex_available_vec[mutex_id]
        .as_mut()
        .unwrap() -= 1;
    *process_inner.mutex_allocation_matrix[tid].as_mut().unwrap()[mutex_id]
        .as_mut()
        .unwrap() += 1;

    //println!("sys_mutex_lock 2");
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    *process_inner.mutex_available_vec[mutex_id]
        .as_mut()
        .unwrap() += 1;
    *process_inner.mutex_allocation_matrix[tid].as_mut().unwrap()[mutex_id]
        .as_mut()
        .unwrap() -= 1;
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    //println!("sys_semaphore_create res_count: {} tid: {}", res_count, tid);
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_available_vec[id] = Some(res_count);
        process_inner
            .semaphore_allocation_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap()[id] = Some(0));
        process_inner
            .semaphore_request_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap()[id] = Some(0));
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner.semaphore_available_vec.push(Some(res_count));
        process_inner
            .semaphore_allocation_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap().push(Some(0)));
        process_inner
            .semaphore_request_matrix
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| x.as_mut().unwrap().push(Some(0)));
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    *process_inner.semaphore_available_vec[sem_id]
        .as_mut()
        .unwrap() += 1;
    *process_inner.semaphore_allocation_matrix[tid]
        .as_mut()
        .unwrap()[sem_id]
        .as_mut()
        .unwrap() -= 1;
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    //println!("sys_semaphore_down sem_id: {} tid: {}", sem_id, tid);
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    *process_inner.semaphore_request_matrix[tid]
        .as_mut()
        .unwrap()[sem_id]
        .as_mut()
        .unwrap() += 1;
    if process_inner.is_enable_deadlock_detect {
        //println!("tid {} deadlock detect start sem_id {}", tid, sem_id);
        /* if 1 >
            - process_inner.semaphore_allocation_matrix[tid]
                .as_mut()
                .unwrap()[sem_id]
                .unwrap()
        {
            return -0xDEAD;
        }
        if 1 > process_inner.semaphore_available_vec[sem_id].unwrap() {
            return -0xDEAD;
        } */

        let mut work = process_inner.semaphore_available_vec.clone();
        let mut finish: Vec<_> = process_inner
            .tasks
            .iter()
            .map(|x| match x {
                Some(_) => false,
                None => true,
            })
            .collect();

        let mut i = 0;
        'out: while i < finish.len() {
            if !finish[i] {
                for j in 0..work.len() {
                    if let Some(v) = work[j] {
                        if v < process_inner.semaphore_request_matrix[i].as_ref().unwrap()[j]
                            .unwrap()
                        {
                            //println!("i {} j {} v {}", i, j, v);
                            i += 1;
                            continue 'out;
                        }
                    }
                }
                for j in 0..work.len() {
                    if let Some(v) = work[j].as_mut() {
                        *v += process_inner.semaphore_allocation_matrix[i]
                            .as_ref()
                            .unwrap()[j]
                            .unwrap();
                    }
                }
                finish[i] = true;
                //println!("tid {} finish", i);
                i = 0;
                continue 'out;
            }
            i += 1;
        }
        //println!("tid {} deadlock detect end sem_id {}", tid, sem_id);

        if finish.iter().any(|&x| !x) {
            *process_inner.semaphore_request_matrix[tid]
                .as_mut()
                .unwrap()[sem_id]
                .as_mut()
                .unwrap() -= 1;
            //println!("tid {} sem_id {} deadlock", tid, sem_id);
            return -0xDEAD;
        }
    }
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    let mut process_inner = process.inner_exclusive_access();
    *process_inner.semaphore_request_matrix[tid]
        .as_mut()
        .unwrap()[sem_id]
        .as_mut()
        .unwrap() -= 1;
    *process_inner.semaphore_available_vec[sem_id]
        .as_mut()
        .unwrap() -= 1;
    *process_inner.semaphore_allocation_matrix[tid]
        .as_mut()
        .unwrap()[sem_id]
        .as_mut()
        .unwrap() += 1;

    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    if enabled == 1 {
        process_inner.is_enable_deadlock_detect = true;
        return 0;
    }
    if enabled == 0 {
        process_inner.is_enable_deadlock_detect = false;
        return 0;
    }
    -1
}
