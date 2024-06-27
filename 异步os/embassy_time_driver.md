# embassy_time_driver::Driver

## 在 embassy 中，计时器的核心抽象是 Driver 接口：

```rust
pub trait Driver: Send + Sync + 'static {
    fn now(&self) -> u64;

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle>;

    fn set_alarm_callback(
        &self,
        alarm: AlarmHandle,
        callback: fn(_: *mut ()),
        ctx: *mut ()
    );

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool;
}
```

下面来解释一下 `Driver trait` 的各个方法：

- `now`：返回当前时间戳（以 `tick` 为单位）。对 `now()` 的调用将始终返回大于或等于早期调用的值。时间不能“倒退”。
- `allocate_alarm`：尝试分配一个警报句柄 `alarm`。如果没有剩余警报，则返回 None。 最初警报没有设置回调，并且`ctx`指针为空。
- `set_alarm_callback`：设置警报 `alarm` 触发时要调用的回调函数 `__pender()` 。回调可以从任何上下文（中断或线程模式）调用。
- `set_alarm`：在给定的时间戳 `timestamp` 设置警报 `alarm`。当当前 `timestamp` 达到 `alarm` 时时间戳，将调用提供的回调函数。


## 示例

```rust
impl Driver for MyDriver {
    fn now(&self) -> u64 {
        riscv::register::time::read64()
    }
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        static ALARM: AtomicU8 = AtomicU8::new(0);
        Some(AlarmHandle::new(ALARM.fetch_add(1, Ordering::Relaxed)))
    }
    fn set_alarm_callback(&self, alarm: AlarmHandle, _callback: fn(*mut ()), _ctx: *mut ()) {
        debug!("now={} alarm_id={} set_alarm_callback", self.now(), alarm.id(),);
        // unsafe { riscv::register::sstatus::set_sie() };
        
    }
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        let now = self.now();
        // debug!("now={now} timestamp={timestamp} alarm_id={} set_alarm", alarm.id());
        let set = now < timestamp;
        if set {
            info!("now={now} set_timer for timestamp={timestamp}");
            // set timer interrupt to wake up CPU from wfi
            set_timer(timestamp as usize);
            unsafe { riscv::register::sstatus::set_sie() };
        }
        set
        // false
    }
}

#[no_mangle]
fn __pender(_ctx: *mut ()) {
    info!("call __pender_");
}

```
    
## 参考

- [embassy_time_driver](https://github.com/embassy-rs/embassy/tree/main/embassy-time-driver)
- [zjp的博客-[Rust] embassy_time_driver](https://zjp-cn.github.io/os-notes/embassy-timer.html#embassy_time_driverdriver)
`