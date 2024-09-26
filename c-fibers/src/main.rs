#![feature(naked_functions)]
use std::arch::asm;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;

/// Runtime 对象的地址
static mut RUNTIME: usize = 0;

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Avaliable,
    Running,
    Ready,
}

struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

impl Thread {
    fn new() -> Self {
        Self {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Avaliable,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        // to run Runtime itself
        let base_thread = Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        let mut threads = vec![base_thread];
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();

        threads.append(&mut available_threads);

        Self {
            threads,
            current: 0,
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        // 还有工作,持续调度
        while self.t_yield() {}
        std::process::exit(0);
    }

    /// 工作线程执行完毕会调用这个函数
    fn t_return(&mut self) {
        // Runtime线程不会
        if self.current != 0 {
            self.threads[self.current].state = State::Avaliable;
            self.t_yield();
        }
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;

        // 找一个就绪状态的线程
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        // 若当前线程有任务，就设置为就绪态
        if self.threads[self.current].state != State::Avaliable {
            self.threads[self.current].state = State::Ready;
        }
        // 调度新线程开始执行
        self.threads[pos].state == State::Running;
        let old_pos = self.current;
        self.current = pos;

        // 切换线程上下文
        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *const ThreadContext = &self.threads[pos].ctx;
            asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C"));
        }
        self.threads.len() > 0
    }

    pub fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Avaliable)
            .expect("no available thread.");

        let size = available.stack.len();
        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            // 找到16字节对齐的栈底
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            // 执行完毕需要执行的函数
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            // 16字节对齐
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            // 执行地址
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }
        available.state = State::Ready;
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    }
}

#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn))
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    }
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch() {
    asm!(
        // 保存旧线程的上下文
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        // 加载新线程的上下文
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    );
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("Thread 1 Starting");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            // 放弃执行权
            yield_thread();
        }
        println!("Thread 1 Finished");
    });

    runtime.spawn(|| {
        println!("Thread 2 Starting");
        let id = 2;
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            // 放弃执行权
            yield_thread();
        }
        println!("Thread 2 Finished");
    });

    runtime.run();
}
