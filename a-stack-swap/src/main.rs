use std::arch::asm;

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    /// stack pointer
    rsp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    asm!(
        // 将第0个参数加上偏移0x00 的到的地址中存的值，放进rsp
        "mov rsp, [{0} + 0x00]",
        // 从栈pop一个地址即hello的地址，并跳转到那里
        "ret",
        // 让编译器用一个通用寄存器存储new的值
        in(reg) new,
    );
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];

    /*

    +------------------------+  <-- 栈底（`stack.as_mut_ptr()` + SSIZE）
    |       stack_bottom     |
    |        ...             |
    |                      16|  <-- 偏移 -16，存储 `hello` 函数的地址
    +                        +
    |           |
    |        (unused)        |
    |         ...            |
    +------------------------+  <-- 结束位置（`stack.as_mut_ptr()`)

         */
    unsafe {
        // 栈底，从高地址开始，向低地址扩展
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
        // 将栈底地址对齐到16字节
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
        // 底部16字节存函数hello的地址
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);
        // 栈指针
        ctx.rsp = sb_aligned.offset(-16) as u64;

        for i in 0..SSIZE {
            println!(
                "mem: {}, val: {}",
                sb_aligned.offset(-i as isize) as usize,
                *sb_aligned.offset(-i as isize)
            )
        }

        gt_switch(&mut ctx);
    }
}
