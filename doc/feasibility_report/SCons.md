## SCons
### **SCons介绍**
SCons 是一套由 Python 语言编写的开源构建系统，用于替代传统的 Makefile。它通过 `SConstruct` 和 `SConscript` 文件来描述构建规则，支持跨平台构建，并能够自动管理文件依赖。SCons 的主要特点包括：
1. **跨平台支持**：能够根据目标平台自动调整构建过程，支持多种工具链（如 ARM GCC、Keil MDK、IAR 等）。
2. **自动依赖管理**：无需手动维护依赖关系，减少因依赖错误导致的构建问题。
3. **灵活的配置**：支持通过 Python 脚本扩展功能，适合复杂的编译需求。
4. **易读性**：基于 Python 的语法清晰易懂，比 Makefile 更易于维护。

#### **SCons构建Rust和C交叉编译的优势**
1. **跨平台支持**：SCons 能够自动识别目标平台并调整构建规则，适合 Rust 和 C 的交叉编译需求。
2. **依赖管理自动化**：SCons 自动管理文件依赖，减少手动维护的复杂性，尤其在混合编译 Rust 和 C 代码时，能够避免因依赖错误导致的构建问题。
3. **灵活的工具链配置**：SCons 支持多种工具链，可以轻松配置 Rust 和 C 的交叉编译环境。
4. **扩展性强**：基于 Python 的脚本支持，可以轻松扩展功能，例如处理 Rust 和 C 的混合编译逻辑。

>值得注意的是，RT-Thread本身采用Scons构建，因此我们可以直接采用官方的构建思路来构建改写后的操作系统，大大降低了对文件依赖关系的处理

#### **SCons构建RT-Thread的优势**
RT-Thread 本身采用 SCons 作为构建工具，其优势包括：
1. **统一的构建系统**：RT-Thread 的 SCons 构建系统已经广泛应用于嵌入式开发，能够无缝支持 Rust 和 C 的交叉编译。
2. **多平台兼容性**：RT-Thread 的 SCons 支持多种工具链（如 ARM GCC、Keil MDK 等），降低了跨平台开发的复杂性。


#### **劣势与挑战**
1. **调试复杂性**：SCons 内部逻辑较为复杂，可能导致调试大型项目时耗时较长。
2. **IDE支持有限**：SCons 对 IDE 的支持有限，可能需要手动配置工程文件。
3. **生态支持不足**：SCons 的社区规模较小，扩展库较少，可能在某些场景下不如 CMake 等工具。

>**总结**
&emsp;&emsp;SCons 是一个强大的构建工具，尤其适合 RT-Thread 和 Rust/C 的交叉编译需求。其跨平台支持、自动依赖管理和扩展性是其主要优势，但调试复杂性和 IDE 支持不足是需要克服的挑战。

### 使用Scons生成C的静态库
Rust调用C代码需要在编译时链接C代码生成的静态库（.a文件）,以下是使用SCons生成C的静态库的方法：
>如何使用Cargo来调用生成的静态库，请看[使用Cargo在Rust中调用C](#)
1. **文件结构：**
    ```
    .
    ├── Cargo.toml          # Cargo的配置文件
    ├── SConstruct          # scons构建使用的脚本，scons根据此脚本来构建项目
    ├── build.rs            # Cargo的编译脚本
    └── src
        ├── clib
        │   ├── ctools.c    # 要用来生成静态库的文件夹
        │   └── ctools.h
        └── main.rs         # Rust源代码，程序入口

    ```

2. **C代码准备：**
    ```C
    //ctools.h
    #ifndef CTOOLS
    #define CTOOLS

    int add(int a, int b);

    #endif

    //ctools.c
    #include "ctools.h"
    int add(int a, int b) {
        return a + b;
    }
    ```
    **Rust代码**
    ```Rust
    extern crate libc;
    use libc::c_int;

    extern "C" {
        fn add(a: c_int, b: c_int) -> c_int;
    }
    fn main() {
        println!("Hello, world!");
        let result:i32 = unsafe {
            add(100,34)
        };
        println!("{}",result);
    }
    ```

3. **Sconstruct编写**
    ```python
    import os   # python的os库
    
    # Environment()函数创建一个构建环境，这个环境对象用于定义构建规则和选项。
    env = Environment(  
        ENV = {
            'PATH': os.environ['PATH'],
            # 这条语句讲解看下方正文
        }
    )

    # 使用replace函数可以修改一些配置的默认值，如更改编译器编译选项
    env.Replace(
        CFLAGS = '-fPIC -I./src/clib',
    )

    # 构建C静态库
    c_sources = Glob('src/clib/*.c')    # 指定要编译的C文件  
    c_objs = env.StaticObject(c_sources)    # 将C的源文件编译成.o文件
    c_lib = env.StaticLibrary(target='libctools', source=c_objs)    # 将.o文件打包成静态库（.a文件）

    # 自定义命令行命令，这里使用Cargo来编译Rust
    rust_build = env.Command(
        target = 'build',
        source = None,
        action = 'cargo build'  # 命令行的命令
    )

    # 显式指定依赖关系，执行rust_build前要编译出c_lib
    env.Depends(rust_build, c_lib)

    # 清理配置
    Clean('.', [
        'libctools.a',          # 生成的静态库
        '.sconsign.dblite',     # scons的辅助文件，可以不管 
        'Cargo.lock',           # Cargo生成的辅助文件   
        'target'                # Cargo build命令生成的的目标文件夹
    ])


    # 构建帮助信息
    Help("""
    构建命令：
    scons          # 构建项目
    scons -Q       # 常规构建
    scons -c       # 清理构建产物
    scons --debug=explain  # 显示详细构建信息
    """)
    ```
    **代码说明：**
    1. **ENV的`PATH`变量**
    - os.environ['PATH'] 获取系统的PATH环境变量。
    - ENV 参数用于设置构建环境中的环境变量。
    - 通过将系统的PATH传递给构建环境，确保构建过程中能够找到系统中的可执行文件（通常是你要用到的一些编译工具，比如Cargo）。
    2. **Replace函数**
    可以在此更改编译器，例如
    ```toml
    CC = 'arm-linux-gnueabihf-gcc'  # 更改默认的C编译器为'arm-linux-gnueabihf-gcc'
    ```
4. **构建信息输出**
    ```bash
    $ scons 
    scons: Reading SConscript files ...
    scons: done reading SConscript files.
    scons: Building targets ...
    gcc -o src/clib/ctools.o -c -fPIC -I./src/clib src/clib/ctools.c    #编译C文件
    ar rc libctools.a src/clib/ctools.o     #打包成库
    ranlib libctools.a
    cargo build     #使用Cargo来编译Rust
        Updating crates.io index
        Locking 3 packages to latest compatible versionsmplete; 0 pending                                                                                                                   
    Compiling shlex v1.3.0
    Compiling libc v0.2.171
    Compiling cc v1.2.17                    ] 0/8: libc(build.rs), shlex                                                                                                                  
    Compiling main v0.1.0 (/home/Asukirina/RC)3/8: libc, cc                                                                                                                               
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s                                                                                                                  
    scons: done building targets.
    ```

5. **运行结果**
    ```bash
    $ cargo run
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
        Running `target/debug/main`
    Hello, world!
    134
    ```

### Scons下在C中调用rust生成的静态库
C代码中调用Rust函数同样需要将Rust代码编译成静态库，再将静态库链接到C代码中
>此部分只讲解如何使用SCons将Rust的静态库链接到C中，关于如何使用Cargo将Rust代码编译成静态库，请看[使用Cargo编译Rust的静态库](#)

1. **代码框架**
    ```
    .
    ├── Cargo.toml  # Cargo的配置文件
    ├── Sconstruct  # Scons的编译脚本
    └── src 
        ├── lib.rs  # 用来生成静态库的rust代码
        └── main.c  # 调用Rust静态库的C程序
    ```

2. **代码准备**
    **Rust代码**
    ```rust
    //lib.rs
    extern crate libc;
    use libc::c_int;
    #[no_mangle]    
    pub extern "C" fn fibonacci(n: c_int) -> c_int {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }
    ```
    >使用`#[no_mangle]`和`extern "C"`来确保函数名不会被Rust编译器修改，并且可以被C代码识别。

    **C代码**
    ```C
    //main.c
    #include <stdio.h>
    extern unsigned int fibonacci(unsigned int n);

    int main() {
        int n = 10;
        unsigned int result = fibonacci(n);
        printf("Fibonacci(%u) = %u\n", n, result);
        return 0;
    }
    ```

    **Cargo.toml**
    ```toml
    [package]
    name = "rustlink"
    version = "0.1.0"
    edition = "2021"

    [lib]                       # Cargo中没指定库的原码会默认为src/librs
    crate-type = ["staticlib"]  # 编译成静态库

    [dependencies]
    libc = "0.2"

    [build-dependencies]
    cc = "1.0"
    ```

    **Sconstruct**
    ```python
    import os

    # 设置环境变量
    env = Environment(
        ENV = {
            'PATH': os.environ['PATH'],
        }
    )

    # 调用Cargo编译lib.rs为静态库librustlink.a,生成的静态库位于target/release中
    Rustlink = env.Command(
        target = 'librustlink.a',
        source = None,
        action = 'cargo build --release'
    )

    # 定义C程序的构建规则
    mian = env.Program('main', 'src/main.c', LIBS=['rustlink'], LIBPATH=['./target/release'])

    # 显示指定依赖关系，先构建Rustlink，再构建main
    env.Depends(mian, Rustlink)

    # 创建别名
    env.Alias('build', mian)

    # 确保Rustlink作为显式目标被构建
    env.Alias('rustlink', Rustlink)

    Clean('.', [
        '.sconsign.dblite',
        'target',
        'Cargo.lock',
    ])
    ```

3. **构建结果**
    *命令行输出*
    ```bash
    $ scons build
    scons: Reading SConscript files ...
    scons: done reading SConscript files.
    scons: Building targets ...
    gcc -o src/main.o -c src/main.c
    cargo build --release
        Updating crates.io index
        Locking 3 packages to latest compatible versionsmplete; 0 pending                                                                                                                   
    Compiling libc v0.2.171
    Compiling rustlink v0.1.0 (/home/Asukirina/Rust_to_C)                                                                                                                                 
        Finished `release` profile [optimized] target(s) in 2.02s                                                                                                                            
    gcc -o main src/main.o -Ltarget/release -lrustlink
    scons: done building targets.
    ```
    **构建完成后的文件结构**
    ```
    .
    ├── Cargo.lock
    ├── Cargo.toml
    ├── Sconstruct
    ├── main        # 生成的可执行文件
    ├── src
    │   ├── lib.rs
    │   ├── main.c
    │   └── main.o
    └── target
        ├── CACHEDIR.TAG
        └── release
            ├── build
            │   ├── libc-6cfc524b8a508b22
            │   │   ├── invoked.timestamp
            │   │   ├── out
            │   │   ├── output
            │   │   ├── root-output
            │   │   └── stderr
            │   └── libc-9192a796e54f7544
            │       ├── build-script-build
            │       ├── build_script_build-9192a796e54f7544
            │       └── build_script_build-9192a796e54f7544.d
            ├── deps
            │   ├── libc-d12fd92582940488.d
            │   ├── liblibc-d12fd92582940488.rlib
            │   ├── liblibc-d12fd92582940488.rmeta
            │   ├── librustlink-5a3057bc13fd054d.a
            │   └── rustlink-5a3057bc13fd054d.d
            ├── examples
            ├── incremental
            ├── librustlink.a   # 生成的静态库
            └── librustlink.d
    ```
4. **执行结果**
    ```bash
    $ ./main
    Fibonacci(10) = 55
    ```
