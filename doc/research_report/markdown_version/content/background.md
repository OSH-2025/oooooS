# 项目背景——赵于洋

## RT-thread介绍

RT-Thread 是一个面向嵌入式操作系统和物联网设备的开源实时操作系统[^Wikipedia_2023b]。由中国 RT-Thread 开发团队创建，该系统旨在填补中国微控制器领域在开源实时操作系统方面的空白。RT-Thread 采用轻量级内核，支持抢占式多任务调度、动态和静态内存管理、设备驱动框架、文件系统、网络协议栈及图形用户界面（GUI）等核心功能，可满足从资源受限的低功耗设备到复杂嵌入式应用的广泛需求。

其核心设计目标包括高性能、低功耗和可扩展性，使其能够适用于多种嵌入式应用场景。截至 2020 年 8 月，RT-Thread 在全球贡献者数量最多的 RTOS 列表中排名第三，仅次于 Zephyr 和 Mbed。

RT-Thread 采用 C 语言编写，模块化设计良好，支持单核和多核架构，并兼容多种处理器架构（如 ARM Cortex-M、RISC-V）。该系统不仅提供了精简高效的 RTOS 内核，还具备组件化扩展框架，使开发者可以根据需求选择文件系统、TCP/IP 网络协议栈、GUI 界面、AI 计算库等功能，以适应更复杂的应用环境。

## RT-Thread的局限性

尽管 RT-Thread 具备强大的功能和广泛的应用场景，但由于其使用 C 语言开发，使得系统面临诸多内存安全问题。例如，C 语言缺乏内置的内存安全机制，容易引发以下常见漏洞[^2023rust]：

- **空指针解引用**：访问已被释放或未初始化的指针可能导致系统崩溃。
- **缓冲区溢出**：超出数组或缓冲区边界的访问可能导致数据损坏或安全漏洞。
- **Use-After-Free**：释放后仍然访问指针可能导致未定义行为，甚至被攻击者利用进行权限提升或远程代码执行。

虽然 RT-Thread 提供了一定的调试工具和运行时检查机制，但这些手段仅能在特定情况下发现问题，无法根本杜绝 C 语言固有的内存安全漏洞。此外，C 语言的低级特性意味着开发者需要手动管理内存、同步机制和错误处理，导致操作系统代码通常较为复杂，维护成本较高。例如，RT-Thread 采用的动态内存管理机制要求开发者手动申请和释放内存，稍有不慎就可能导致内存泄漏或碎片化，影响系统的长期稳定运行。

这些问题不仅限制了 RT-Thread 在高安全性和高可靠性场景下的应用，也增加了代码维护的难度。因此，探索更安全、高效的编程语言来优化 RT-Thread 内核，提高其安全性、稳定性和开发效率，成为值得研究的重要方向。

## Rust语言的优越性

Rust 是一门专为高性能、类型安全和并发编程设计的通用编程语言，尤其适用于系统级开发[^Wikipedia_2025]。与传统的 C 和 C++ 不同，Rust 在不依赖垃圾回收的情况下实现了内存安全，有效防止空指针解引用、缓冲区溢出和数据竞争等常见问题。

Rust 之所以能够提供内存安全保障，主要得益于其独特的 所有权（Ownership）、借用（Borrowing） 和 生命周期（Lifetime） 三大核心机制。Rust 编译器在编译期执行严格的借用检查，确保所有引用始终指向有效的内存，同时自动管理资源释放，避免手动内存管理带来的泄漏和未定义行为。此外，Rust 的零成本抽象允许开发者编写既安全又高效的代码，而不会引入额外的运行时开销。

在并发编程方面，Rust 通过 线程安全机制（`Send` 和 `Sync`），在编译期保证多线程程序的安全性：

- **数据竞争消除** → Rust 强制确保并发访问的数据完整性。
- **无锁并发支持** → Rust 采用 Actor 模型和无锁数据结构，提升多线程效率。
- **高效的线程同步** → 提供 安全的 `Mutex`、`RwLock` 机制，避免死锁。

这些特性使 Rust 成为 高安全性、高性能系统软件开发（如 操作系统、浏览器引擎、云计算）的理想选择

正因为 Rust 在性能、内存安全和并发方面的独特优势，它成为了操作系统内核、嵌入式系统、WebAssembly、高性能计算等领域的热门选择[^sharma2023rust][^sharma2024rust]，并广泛应用于安全性要求极高的开发场景，如浏览器引擎（Firefox 的 Servo）、区块链（Solana）、云计算（AWS Firecracker）[^Zhang]等。越来越多的公司和企业选择 Rust 语言来进行开发[^Lib.rs]。

![Rust下载量和公司使用情况](img/lib-rs-stats-rust-downloads-users.jpg){#fig1}

所以我们小组计划使用 Rust 语言对 Rt-Thread 系统的部分内核进行重构，以提升系统的安全性和性能，从而更好地满足嵌入式应用的需求。

## 当前Rust RTOS生态空缺

当前 Rust RTOS 生态中，已有多个项目正在开发和完善[^AreWeRTOSYet]：

![Rust RTOS生态](img/Rust_RTOS.png){#fig2}

可以看出，Rust 原生实时操作系统的开发已取得一定进展，然而，与 RT-Thread、Zephyr 等成熟 RTOS 相比，Rust RTOS仍然存在生态空缺：

- 驱动程序支持有限：Rust 生态尚未建立完备的外设驱动库。
- 实时性优化不足：Rust 原生 RTOS 仍需优化调度机制和实时性。
- 开发者社区规模较小：Rust RTOS 仍处于早期发展阶段，缺乏大规模应用案例。

因此，将 Rust 引入 RT-Thread 这样的成熟嵌入式操作系统，不仅能够提升系统的安全性和性能，还能为 Rust RTOS 生态的发展提供重要的参考和借鉴，进一步推动其成熟与完善。

### Rust改写Rt-thread内核的意义

+ **内存安全性提升**：Rust 的所有权模型和借用检查机制可以从根本上消除空指针解引用、缓冲区溢出等常见内存安全漏洞，提高系统的稳定性和可靠性。
+ **并发性能优化**：Rust 提供线程安全的并发模型，利用 `Send` 和 `Sync` 机制在编译期确保数据在多线程环境中的安全性，避免数据竞争问题，从而提升系统的并发性能。
+ **代码质量提升**：Rust 的现代化语言特性和强类型系统使代码更加清晰易读，结合其强大的包管理器 Cargo 及丰富的生态系统，可以提高代码的可维护性和可扩展性，降低长期维护成本。
+ **生态贡献**：Rust 在嵌入式领域的应用仍处于发展阶段，引入 Rust 到 RT-Thread 的过程中，可以丰富嵌入式开发生态，推动 Rust 在该领域的进一步普及。

综上所述，我们计划使用 Rust 语言对 RT-Thread 内核模块进行改写，以提升系统的安全性和性能，为嵌入式开发者提供更安全、高效的开发工具。

[^RTThread]: [RT-Thread Documentation](https://www.rt-thread.org/document/site/#/) (Accessed: 2025-03-18)
[^Wikipedia_2025]: [Rust (programming language)](https://en.wikipedia.org/wiki/Rust_(programming_language)) (Accessed: 2025-03)
[^Wikipedia_2023b]: [RT-Thread](https://en.wikipedia.org/wiki/RT-Thread) (Accessed: 2023-12)
[^2023rust]: 胡霜, 华保健, 欧阳婉容, 樊淇梁. Rust 语言安全研究综述[J]. 信息安全学报, 2023, 8(6): 64-83.
[^sharma2023rust]: Sharma, Ayushi and Sharma, Shashank and Torres-Arias, Santiago and Machiry, Aravind. Rust for embedded systems: current state, challenges and open problems[J]. arXiv preprint arXiv:2311.05063, 2023.
[^sharma2024rust]: Sharma, Ayushi and Sharma, Shashank and Tanksalkar, Sai Ritvik and Torres-Arias, Santiago and Machiry, Aravind. Rust for Embedded Systems: Current State and Open Problems[C]. Proceedings of the 2024 on ACM SIGSAC Conference on Computer and Communications Security, 2024.
[^Zhang]: [Rust 2022 全球商业化应用盘点](https://rustmagazine.org/issue-1/2022-review-the-adoption-of-rust-in-business-zh/) (Accessed: 2025-03-22)
[^Lib.rs]: [Lib.rs Statistics](https://lib.rs/stats) (Accessed: 2025-03-22)
[^AreWeRTOSYet]: [Are We RTOS Yet?](https://arewertosyet.com/) (Accessed: 2025-03-22)