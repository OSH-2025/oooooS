# Rust-thread

## 项目结构

(略)

## 项目实现

(略)

## 运行

执行 `cargo build` 构建项目。

执行 `qemu-system-arm   -cpu cortex-m4   -machine netduinoplus2   -nographic   -semihosting-config enable=on,target=native   -kernel target/thumbv7em-none-eabihf/debug/RusT-thread` 在QEMU上运行项目。

（注：需要安装QEMU，并安装arm-none-eabi-gcc）

或者执行 `cargo run` 在QEMU上运行项目。

## 测试

并没有进行测试。



