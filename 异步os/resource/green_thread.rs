use std::ptr;
use std::fmt;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: *mut Runtime = std::ptr::NonNull::dangling().as_ptr();

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}
impl std::fmt::Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Runtime {{ current: {} ;\n", self.current);
        for (index, my_struct) in self.threads.iter().enumerate() {
            write!(f, "  {}\n", my_struct)?;
        }
        write!(f, "}}")
    }
}
#[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Available => write!(f, "Available"),
            State::Running => write!(f, "Running"),
            State::Ready => {
                write!(f, "Ready")
            }
        }
    }
}
struct Thread {
    id: usize,
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
    task: Option<Box<dyn Fn()>>,
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
    thread_ptr: u64,
}

impl Thread {
    fn new(id: usize) -> Self {
        Thread {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
            task: None,
        }
    }
}
impl std::fmt::Display for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread {{id: {},  state: {} ,  ctx: {:?}}}", self.id,  self.state, self.ctx)
    }
}
impl Runtime {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let base_thread = Thread {
            id: 0,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
            task: None,
        };

        let mut threads = vec![base_thread];
        threads[0].ctx.thread_ptr = &threads[0] as *const Thread as u64;
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(Thread::new).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            let r_ptr: *mut Runtime = self;
            println!("RUNTIME has been initialized\n");
            RUNTIME = r_ptr;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;

        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            println!("{}", &self);
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        println!(
            "[current pos={pos} old_pos={old_pos} len={}] switch",
            self.threads.len()
        );
        println!(
            "old.ctx={:p} current.cxt={:p}",
            &self.threads[old_pos].ctx, &self.threads[pos].ctx
        );
        unsafe {
            __switch(&mut self.threads[old_pos].ctx, &self.threads[pos].ctx);
        }
        true
    }

    pub fn spawn<F: Fn() + 'static>(f: F) {
        unsafe {
            let rt_ptr = RUNTIME;
            let available = (*rt_ptr)
                .threads
                .iter_mut()
                .find(|t| t.state == State::Available)
                .expect("no available thread.");
            println!("spawn use RUNTIME to find available thread");
            
            let size = available.stack.len();
            let s_ptr = available.stack.as_mut_ptr();
            let s_ptr = s_ptr.add(size);
            let s_ptr = s_ptr as u64 & !0xf;
            let s_ptr = s_ptr as *mut u8;
            
            // 这俩都得对齐
            ptr::write_unaligned(s_ptr.sub(16).cast::<u64>(), guard as usize as u64);
            ptr::write_unaligned(s_ptr.sub(32) as *mut u64, __call as usize as u64);
            available.ctx.rsp = s_ptr.sub(32) as u64;
            println!("available.ctx.rsp = {:x}", available.ctx.rsp);

            available.task = Some(Box::new(f));
            available.ctx.thread_ptr = available as *const Thread as u64;
            available.state = State::Ready;
        }
    }
}

// 因为guard是对齐的，这里ret的时候rsp也得对齐
// 这个用rust好像做不了吧
fn call(thread: u64) {
    let thread = unsafe { &*(thread as *const Thread) };
    if let Some(f) = &thread.task {
        f();
    }
}

std::arch::global_asm!(
    ".globl __call",
    "__call:",
    //  fn call(thread: u64) {
    "   subq    $0x28, %rsp",
    "   movq    %rdi, 0x10(%rsp)",
    //  let thread = unsafe { &*(thread as *const Thread) };
    "   movq    %rdi, 0x18(%rsp)",
    //  if let Some(f) = &thread.task {
    "   movq    %rdi, %rax",
    "   addq    $0x60, %rax",
    "   movq    %rax, 0x8(%rsp)",
    "   movq    0x60(%rdi), %rdx",
    "   movl    $0x1, %eax",
    "   xorl    %ecx, %ecx",
    "   cmpq    $0x0, %rdx",
    "   cmoveq  %rcx, %rax",
    "   cmpq    $0x1, %rax",
    "   jne     0f",
    "   movq    0x8(%rsp), %rdi",
    //     if let Some(f) = &thread.task {
    "   movq    %rdi, 0x20(%rsp)",
    //         f();
    "   movq   8(%rdi), %rdi",
    "   callq   *0x28(%rdi)",
    // }
    "0: addq    $0x28, %rsp",

    // 没错，这里要再加一个8，这样retq使用的就是栈中的那个guard
    "   addq    $8, %rsp",
    "   retq",
    options(att_syntax)
);

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME;
        let rt = &mut *rt_ptr;
        println!("THREAD {} FINISHED.", rt.threads[rt.current].id);
        rt.t_return();
    };
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME;
        (*rt_ptr).t_yield();
    };
}

std::arch::global_asm!(
    r#"
.globl __switch
__switch:
  mov  [rdi+0x00], rsp
  mov  [rdi+0x08], r15
  mov  [rdi+0x10], r14
  mov  [rdi+0x18], r13
  mov  [rdi+0x20], r12
  mov  [rdi+0x28], rbx
  mov  [rdi+0x30], rbp

  mov  rsp, [rsi+0x00]
  mov  r15, [rsi+0x08]
  mov  r14, [rsi+0x10]
  mov  r13, [rsi+0x18]
  mov  r12, [rsi+0x20]
  mov  rbx, [rsi+0x28]
  mov  rbp, [rsi+0x30]
  mov  rdi, [rsi+0x38]
  ret
"#
);

extern "C" {
    fn __switch(old: *mut ThreadContext, new: *const ThreadContext);
    fn __call(thread: u64);
}

#[cfg(not(windows))]
fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    Runtime::spawn(|| {
        println!("I haven't implemented a timer in this example.");
        yield_thread();
        println!("Finally, notice how the tasks are executed concurrently.");
    });
    Runtime::spawn(|| {
        println!("But we can still nest tasks...");
        Runtime::spawn(|| {
            println!("...like this!");
        })
    });
    runtime.run();
    println!("Hello, world!");
}