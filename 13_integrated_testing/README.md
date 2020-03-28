# Tutorial 13 - Integrated Testing

## tl;dr

- We implement our own test framework using `Rust`'s [custom_test_frameworks] feature by enabling
  `Unit Tests` and `Integration Tests` using `QEMU`.
- It is also possible to have test automation for the kernel's `console` (provided over `UART` in
  our case): Sending strings/characters to the console and expecting specific answers in return.

<img src="../doc/13_demo.gif" widht="880">

## Table of Contents

- [Introduction](#introduction)
- [Challenges](#challenges)
  * [Acknowledgements](#acknowledgements)
- [Implementation](#implementation)
  * [Test Organization](#test-organization)
  * [Enabling `custom_test_frameworks` for Unit Tests](#enabling-custom_test_frameworks-for-unit-tests)
    + [The Unit Test Runner](#the-unit-test-runner)
    + [Calling the Test `main()` Function](#calling-the-test-main-function)
  * [Quitting QEMU with user-defined Exit Codes](#quitting-qemu-with-user-defined-exit-codes)
    + [Exiting Unit Tests](#exiting-unit-tests)
  * [Controlling Test Kernel Execution](#controlling-test-kernel-execution)
    + [Wrapping QEMU Test Execution](#wrapping-qemu-test-execution)
  * [Writing Unit Tests](#writing-unit-tests)
  * [Integration Tests](#integration-tests)
    + [Test Harness](#test-harness)
    + [No Test Harness](#no-test-harness)
    + [Overriding Panic Behavior](#overriding-panic-behavior)
  * [Console Tests](#console-tests)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

Through the course of the previous tutorials, we silently started to adopt a kind of anti-pattern:
Using the kernel's main function to not only boot the target, but also test or showcase
functionality. For example:
  - Stalling execution during boot to test the kernel's timekeeping code by spinning for 1 second.
  - Willingly causing exceptions to see the exception handler running.

The feature set of the kernel is now rich enough so that it makes sense to introduce proper testing
modeled after Rust's [native testing framework]. This tutorial extends our kernel with three basic
testing facilities:
  - Classic `Unit Tests`.
  - [Integration Tests] (self-contained tests stored in the `$CRATE/tests/` directory).
  - `Console Tests`. These are integration tests acting on external stimuli - aka `console` input.
    Sending strings/characters to the console and expecting specific answers in return.

[native testing framework]: https://doc.rust-lang.org/book/ch11-00-testing.html

## Challenges

Testing Rust `#![no_std]` code like our kernel is, at the point of writing this tutorial, not an
easy endeavor. The short version is: We cannot use Rust's [native testing framework] straight away.
Utilizing the `#[test]` attribute macro and running `cargo test` (`xtest` in our case) would throw
compilation errors, because there are dependencies on the standard library.

[native testing framework]: https://doc.rust-lang.org/book/ch11-00-testing.html

We have to fall back to Rust's unstable [custom_test_frameworks] feature. It relieves us from
dependencies on the standard library, but comes at the cost of having a reduced feature set. Instead
of annotating functions with `#[test]`, the `#[test_case]` attribute must be used. Additionally, we
need to write a `test_runner` function, which is supposed to execute all the functions annotated
with `#[test_case]`. This is barely enough to get `Unit Tests` running, though. There will be some
more challenges that need solving for getting `Integration Tests` running as well.

Please note that for automation purposes, all testing will be done in `QEMU` and not on real
hardware.

[custom_test_frameworks]: https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html
[Integration Tests]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests

### Acknowledgements

On this occasion, kudos to [@phil-opp] for his x86-based [testing] article. It helped a lot in
putting together this tutorial. Please go ahead and read it for a different perspective and
additional insights.

[testing]: https://os.phil-opp.com/testing

## Implementation

We introduce a new `Makefile` target:

```shell
make test
```

In essence, `make test` will execute `cargo xtest` instead of `cargo xrustc`. The details will be
explained in due course. The rest of the tutorial will explain as chronologically as possible what
happens when `make test` aka `cargo xtest` runs.

### Test Organization

Until now, our kernel was a so-called `binary crate`. As [explained in the official Rust book], this
crate type disallows having `integration tests`. Quoting the book:

[explained in the official Rust book]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates

> If our project is a binary crate that only contains a _src/main.rs_ file and doesn’t have a
> _src/lib.rs_ file, we can’t create integration tests in the _tests_ directory and bring functions
> defined in the _src/main.rs_ file into scope with a `use` statement. Only library crates expose
> functions that other crates can use; binary crates are meant to be run on their own.

> This is one of the reasons Rust projects that provide a binary have a straightforward
> _src/main.rs_ file that calls logic that lives in the _src/lib.rs_ file. Using that structure,
> integration tests _can_ test the library crate with `use` to make the important functionality
> available. If the important functionality works, the small amount of code in the _src/main.rs_
> file will work as well, and that small amount of code doesn’t need to be tested.

So let's do that first: We add a `lib.rs` to our crate that aggregates and exports the lion's share
of the kernel code. The `main.rs` file is stripped down to the minimum. It only keeps the
`kernel_init() -> !` and `kernel_main() -> !` functions, everything else is brought into scope with
`use` statements.

Since it is not possible to use `kernel` as the name for both the library and the binary part of the
crate, new entries in `Cargo.toml` are needed to differentiate the names. What's more, `cargo xtest`
would try to compile and run `unit tests` for both. In our case, it will be sufficient to have all
the unit test code in `lib.rs`, so test generation for `main.rs` can be disabled in `Cargo.toml` as
well through the `test` flag:

```toml
[lib]
name = "libkernel"
test = true

[[bin]]
name = "kernel"
test = false
```

### Enabling `custom_test_frameworks` for Unit Tests

In `lib.rs`, we add the following headers to get started with `custom_test_frameworks`:

```rust
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]
```

Since this is a library now, we do not keep the `#![no_main]` inner attribute that `main.rs` has,
because a library has no `main()` entry function, so the attribute does not apply. When compiling
for testing, though, it is still needed. The reason is that `cargo xtest` basically turns `lib.rs`
into a binary again by inserting a generated `main()` function (which is then calling a function
that runs all the unit tests, but more about that in a second...).

However, since  our kernel code [overrides the compiler-inserted `main` shim] by way of using
`#![no_main]`, we need the same when `cargo xtest` is producing its test kernel binary. After all,
what we want is a minimal kernel that boots on the target and runs its own unit tests. Therefore, we
conditionally set this attribute (`#![cfg_attr(test, no_main)]`) when the `test` flag is set, which
it is when `cargo xtest` runs.

[overrides the compiler-inserted `main` shim]: https://doc.rust-lang.org/unstable-book/language-features/lang-items.html?highlight=no_main#writing-an-executable-without-stdlib

#### The Unit Test Runner

The `#![test_runner(crate::test_runner)]` attribute declares the path of the test runner function
that we are supposed to provide. This is the one that will be called by the `cargo xtest` generated
`main()` function. Here is the implementation in `lib.rs`:

```rust
/// The default runner for unit tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    println!("Running {} tests", tests.len());
    println!("-------------------------------------------------------------------\n");
    for (i, test) in tests.iter().enumerate() {
        print!("{:>3}. {:.<58}", i + 1, test.name);

        // Run the actual test.
        (test.test_func)();

        // Failed tests call panic!(). Execution reaches here only if the test has passed.
        println!("[ok]")
    }
}
```

The function signature shows that `test_runner` takes one argument: A slice of
`test_types::UnitTest` references. This type definition lives in an external crate stored at
`$ROOT/test_types`. It is external because the type is also needed for a self-made [procedural
macro] that we'll use to write unit tests, and procedural macros _have_ to live in their own crate.
So to avoid a circular dependency between kernel and proc-macro, this split was needed. Anyways,
here is the type definition:

[procedural macro]: https://doc.rust-lang.org/reference/procedural-macros.html

```rust
/// Unit test container.
pub struct UnitTest {
    /// Name of the test.
    pub name: &'static str,

    /// Function pointer to the test.
    pub test_func: fn(),
}
```

A `UnitTest` provides a name and a classic function pointer to the unit test function. The
`test_runner` just iterates over the slice, prints the respective test's name and calls the test
function.

The convetion is that as long as the test function does not `panic!`, the test was successful.

#### Calling the Test `main()` Function

The last of the attributes we added is `#![reexport_test_harness_main = "test_main"]`. Remember that
our kernel uses the `no_main` attribute, and that we also set it for the test compilation. We did
that because we wrote our own `_start()` function (in `aarch64.rs`), which kicks off the following
call chain during kernel boot:

| | Function  | File |
| - | - | - |
| 1. | `_start()` | `lib.rs` |
| 2. | (some more arch64 code) | `lib.rs` |
| 3. | `runtime_init()` | `lib.rs` |
| 4. | `kernel_init()` | `main.rs` |
| 5. | `kernel_main()` | `main.rs` |

A function named `main` is never called. Hence, the `main()` function generated by `cargo xtest`
would be silently dropped, and therefore the tests would never be executed. As you can see,
`runtime_init()` is the last function residing in our carved-out `lib.rs`, and it calls into
`kernel_init()`. So in order to get the tests to execute, we add a test-environment version of
`kernel_init()` to `lib.rs` as well (conditional compilation ensures it is only present when the
test flag is set), and call the `cargo xtest` generated `main()` function from there.

This is where `#![reexport_test_harness_main = "test_main"]` finally comes into picture. It declares
the name of the generated main function so that we can manually call it. Here is the final
implementation in `lib.rs`:

```rust
/// The `kernel_init()` for unit tests. Called from `runtime_init()`.
#[cfg(test)]
#[no_mangle]
unsafe fn kernel_init() -> ! {
    bsp::console::qemu_bring_up_console();

    test_main();

    cpu::qemu_exit_success()
}
```

Note that we first call `bsp::console::qemu_bring_up_console()`. Since we are running all our tests
inside `QEMU`, we need to ensure that whatever peripheral implements the kernel's `console`
interface is initialized, so that we can print from our tests. If you recall [tutorial 03], bringing
up peripherals in `QEMU` might not need the full initialization as is needed on real hardware
(setting clocks, config registers, etc...) due to the abstractions in `QEMU`'s emulation code. So
this is an opportunity to cut down on setup code.

[tutorial 03]: ../03_hacky_hello_world

As a matter of fact, for the `Raspberrys`, nothing needs to be done and the function is empy. But
this might be different for other hardware emulated by QEMU, so it makes sense to introduce the
function now to make it easier in case new `BSPs` are  added to the kernel in the future.

Next, the reexported `test_main()` is called, which will call our `test_runner()` which finally
prints the unit test names and executes them.

### Quitting QEMU with user-defined Exit Codes

Let's recap where we are right now:

We've enabled `custom_test_frameworks` in `lib.rs` to a point where, when using `make test`, the
code gets compiled to a test kernel binary that eventually executes all the (yet-to-be-defined)
`UnitTest` instances by executing all the way from `_start()` to our `test_runner()` function.

Through mechanisms that are explained later, `cargo` will now instantiate a `QEMU` process that
exectues this test kernel. The question now is: How is test success/failure communicated to `cargo`?
Answer: `cargo` inspects `QEMU`'s [exit status]:

  - `0` translates to testing was successful.
  - `non-0` means failure.

Hence, we need a clever trick now so that our Rust kernel code can get `QEMU` to exit itself with an
exit status that the kernel code supplies. In [@phil-opp]'s testing article, you [learned how to do
this] for `x86 QEMU` systems by using a special `ISA` debug-exit device. Unfortunately, we can't
have that one for our `aarch64` system because it is not compatible.

In our case, we can leverage the ARM [semihosting] emulation of `QEMU` and do a `SYS_EXIT`
semihosting call with an additional parameter for the exit code. I've written a separate crate,
[qemu-exit], to do this, so let us import it. Specifically, the following two functions:

```rust
qemu_exit::aarch64::exit_success() // QEMU binary executes `exit(0)`.
qemu_exit::aarch64::exit_failure() // QEMU binary executes `exit(1)`.
```

[Click here] in case you are interested in the implementation. Note that for the functions to work,
the `-semihosting` flag must be added to the `QEMU` invocation.

[exit status]: https://en.wikipedia.org/wiki/Exit_status
[@phil-opp]: https://github.com/phil-opp
[learned how to do this]: https://os.phil-opp.com/testing/#exiting-qemu
[semihosting]: https://static.docs.arm.com/100863/0200/semihosting.pdf
[qemu-exit]: https://github.com/andre-richter/qemu-exit
[Click here]: https://github.com/andre-richter/qemu-exit/blob/master/src/aarch64.rs

#### Exiting Unit Tests

Unit test failure shall be triggered by the `panic!` macro, either directly or by way of using
`assert!` macros. Until now, our `panic!` implementation finally called `cpu::wait_forever()` to
safely park the panicked CPU core in a busy loop. This can't be used for the unit tests, because
`cargo` would wait forever for `QEMU` to exit and stall the whole test run. Again, conditional
compilation is used to differentiate between a release and testing version of how a `panic!`
concludes. Here is the new testing version:

```rust
/// The point of exit when the library is compiled for testing.
#[cfg(test)]
#[no_mangle]
fn _panic_exit() -> ! {
    cpu::qemu_exit_failure()
}
```

In case none of the unit tests panicked, `lib.rs`'s  `kernel_init()` calls
`cpu::qemu_exit_success()` to successfully conclude the unit test run.

### Controlling Test Kernel Execution

Now is a good time to catch up on how the test kernel binary is actually being executed. Normally,
`cargo test` would try to execute the compiled binary as a normal child process. This would fail
horribly because we build a kernel, and not a userspace process. Also, chances are very high that
you sit in front of an `x86` machine, whereas the RPi kernel is `AArch64`.

Therefore, we need to install some hooks that make sure the test kernel gets executed inside `QEMU`,
quite like it is done for the existing `make qemu` target that is in place since tutorial 1. The
first step is to add a new file to the project, `.cargo/config`:

```toml
[target.'cfg(target_os = "none")']
runner = "target/kernel_test_runner.sh"
```

Instead of executing a compilation result directly, the `runner` flag will instruct `cargo` to
delegate the execution. Using the setting depicted above, `target/kernel_test_runner.sh` will be
executed and given the full path to the compiled test kernel as the first command line argument.

The file `kernel_test_runner.sh` does not exist by default. We generate it on demand throguh the
`make test` target:

```Makefile
define KERNEL_TEST_RUNNER
	#!/usr/bin/env bash

	$(OBJCOPY_CMD) $$1 $$1.img
	TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')
	$(DOCKER_CMD_TEST) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
		ruby tests/runner.rb $(DOCKER_EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
endef

export KERNEL_TEST_RUNNER
test: $(SOURCES)
	@mkdir -p target
	@echo "$$KERNEL_TEST_RUNNER" > target/kernel_test_runner.sh
	@chmod +x target/kernel_test_runner.sh
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(XTEST_CMD) $(TEST_ARG)
```

It first does the standard `objcopy` step to strip the `ELF` down to a raw binary. Just like in all
the other Makefile targets. Next, the script generates a relative path from the absolute path
provided to it by `cargo`, and finally compiles a `docker` command to execute the test kernel. For
reference, here it is fully resolved for an `RPi3 BSP`:

```bash
docker run -it --rm -v /opt/rust-raspi3-OS-tutorials/13_integrated_testing:/work -w /work rustembedded/osdev-utils ruby tests/runner.rb qemu-system-aarch64 -M raspi3 -serial stdio -display none -semihosting -kernel $TEST_BINARY
```

We're still not done with all the redirections. Spotted the `ruby tests/runner.rb` part that gets
excuted inside Docker?

#### Wrapping QEMU Test Execution

`runner.rb` is a [Ruby] wrapper script around `QEMU` that, for unit tests, catches the case that a
test gets stuck, e.g. in an unintentional busy loop or a crash. If `runner.rb` does not observe any
output of the test kernel for `5 seconds`, it cancels the execution and reports a failure back to
`cargo`. If `QEMU` exited itself by means of `aarch64::exit_success() / aarch64::exit_failure()`,
the respective exit status code is passed through. The essential part happens here in `class
RawTest`:

```ruby
def exec
    error = 'Timed out waiting for test'
    io = IO.popen(@qemu_cmd)

    while IO.select([io], nil, nil, MAX_WAIT_SECS)
        begin
            @output << io.read_nonblock(1024)
        rescue EOFError
            io.close
            error = $CHILD_STATUS.to_i != 0
            break
        end
    end
```

[Ruby]: https://www.ruby-lang.org/

### Writing Unit Tests

Alright, that's a wrap for the whole chain from `make test` all the way to reporting the test exit
status back to `cargo xtest`. It is a lot to digest already, but we haven't even learned to write
`Unit Tests` yet.

In essence, it is almost like in `std` environments, with the difference that `#[test]` can't be
used, because it is part of the standard library. The `no_std` replacement attribute provided by
`custom_test_frameworks` is `#[test_case]`. You can put `#[test_case]` before functions, constants
or statics (you have to decide for one and stick with it). Each attributed item is added to the
"list" that is then passed to the `test_runner` function.

As you learned earlier, we decided that our tests shall be instances of `test_types::UnitTest`. Here
is the type definition again:

```rust
/// Unit test container.
pub struct UnitTest {
    /// Name of the test.
    pub name: &'static str,

    /// Function pointer to the test.
    pub test_func: fn(),
}
```

So what we could do now is write something like:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    const TEST1: test_types::UnitTest = test_types::UnitTest {
            name: "test_runner_executes_in_kernel_mode",
            test_func: || {
                let (level, _) = current_privilege_level();

                assert!(level == PrivilegeLevel::Kernel)
            },
        };
}
```

Since this is a bit boiler-platy with the const and name definition, let's write a [procedural
macro] named `#[kernel_test]` to simplify this. It should work this way:

  1. Must be put before functions that take no arguments and return nothing.
  2. Automatically constructs a `const UnitTest` from attributed functions like shown above by:
      1. Converting the function name to the `name` member of the `UnitTest` struct.
      2. Populating the `test_func` member with a closure that executes the body of the attributed
         function.

For the sake of brevity, we're not going to discuss the macro implementation. [The source is in the
test-macros crate] if you're interested in it. Using the macro, the example shown before now boils
down to this (this is now an actual example from [exception.rs]:

[procedural macro]: https://doc.rust-lang.org/reference/procedural-macros.html
[The source is in the test-macros crate]: test-macros/src/lib.rs
[exception.rs]: src/exception.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Libkernel unit tests must execute in kernel mode.
    #[kernel_test]
    fn test_runner_executes_in_kernel_mode() {
        let (level, _) = current_privilege_level();

        assert!(level == PrivilegeLevel::Kernel)
    }
}
```

Note that since proc macros need to live in their own crates, we need to create a new one at
`$ROOT/test-macros` and save it there.

Aaaaaand that's how you write unit tests. We're finished with that part for good now :raised_hands:.

### Integration Tests

We are still not done with the tutorial, though :scream:.

Integration tests need some special attention here and there too. As you already learned, they live
in `$CRATE/tests/`. Each `.rs` file in there gets compiled into its own test kernel binary and
executed separately by `cargo xtest`. The code in the integration tests includes the library part of
our kernel (`libkernel`) through `use` statements.

Also note that the entry point for each `integration test` must be the `kernel_init()` function
again, just like in the `unit test` case.

#### Test Harness

By default, `cargo xtest` will pull in the test harness (that's the official name for the generated
`main()` function) into integration tests as well. This gives you a further means of partitioning
your test code into individual chunks. For example, take a look at
`tests/01_interface_sanity_timer.rs`:

```rust
//! Timer sanity tests.

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::test_runner)]

mod panic_exit_failure;

use core::time::Duration;
use libkernel::{bsp, cpu, time, time::interface::TimeManager};
use test_macros::kernel_test;

#[no_mangle]
unsafe fn kernel_init() -> ! {
    bsp::console::qemu_bring_up_console();

    // Depending on CPU arch, some timer bring-up code could go here. Not needed for the RPi.

    test_main();

    cpu::qemu_exit_success()
}

/// Simple check that the timer is running.
#[kernel_test]
fn timer_is_counting() {
    assert!(time::time_manager().uptime().as_nanos() > 0)
}

/// Timer resolution must be sufficient.
#[kernel_test]
fn timer_resolution_is_sufficient() {
    assert!(time::time_manager().resolution().as_nanos() < 100)
}
```

Note how the `test_runner` from `libkernel` is pulled in through
`#![test_runner(libkernel::test_runner)]`.

#### No Test Harness

For some tests, however, it is not needed to have the harness, because there is no need or
possibility to partition the test into individual pieces. In this case, all the test code can live
in `kernel_init()`, and harness generation can be turned off through `Cargo.toml`. This tutorial
introduces two tests that don't need a harness. Here is how harness generation is turned off for
them:

```toml
# List of tests without harness.
[[test]]
name = "00_interface_sanity_console"
harness = false

[[test]]
name = "02_arch_exception_handling_sync_page_fault"
harness = false
```

#### Overriding Panic Behavior

It is also important to understand that the `libkernel` made available to the integration tests is
the _release_ version. Therefore, it won't contain any code attributed with `#[cfg(test)]`!

One of the implications of this is that the `panic handler` provided by `libkernel` will be the
version from the release kernel that spins forever, and not the test version that exits `QEMU`.

One way to navigate around this is to declare the _release version of the panic exit function_ in
`lib.rs` as a [weak symbol]:

```rust
#[cfg(not(test))]
#[linkage = "weak"]
#[no_mangle]
fn _panic_exit() -> ! {
    cpu::wait_forever()
}
```

[weak symbol]: https://en.wikipedia.org/wiki/Weak_symbol

Integration tests in `$CRATE/tests/` can now override it according to their needs, because depending
on the kind of test, a `panic!` could mean success or failure. For example,
`tests/02_arch_exception_handling_sync_page_fault.rs` is intentionally causing a page fault, so the
wanted outcome is a `panic!`. Here is the whole test (minus some inline comments):

```rust
//! Page faults must result in synchronous exceptions.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod panic_exit_success;

use libkernel::{bsp, cpu, exception, memory, println};

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use memory::mmu::interface::MMU;

    bsp::console::qemu_bring_up_console();

    println!("Testing synchronous exception handling by causing a page fault");
    println!("-------------------------------------------------------------------\n");

    exception::handling_init();

    if let Err(string) = memory::mmu::mmu().init() {
        println!("MMU: {}", string);
        cpu::qemu_exit_failure()
    }

    println!("Writing beyond mapped area to address 9 GiB...");
    let big_addr: u64 = 9 * 1024 * 1024 * 1024;
    core::ptr::read_volatile(big_addr as *mut u64);

    // If execution reaches here, the memory access above did not cause a page fault exception.
    cpu::qemu_exit_failure()
}
```

The `_panic_exit()` version that makes `QEMU` return `0` (indicating test success) is pulled in by
`mod panic_exit_success;`. The counterpart would be `mod panic_exit_failure;`. We provide both in
the `tests` folder, so each integration test can import the one that it needs.

### Console Tests

As the kernel or OS grows, it will be more and more interesting to test user/kernel interaction
through the serial console. That is, sending strings/characters to the console and expecting
specific answers in return. The `runner.rb` wrapper script provides infrastructure to do this with
little overhead. It basically works like this:

  1. For each integration test, check if a companion file to the `.rs` test file exists.
      - A companion file has the same name, but ends in `.rb`.
      - The companion file contains one or more console subtests.
  2. If it exists, load the file to dynamically import the console subtests.
  3. Spawn `QEMU` and attach to the serial console.
  4. Run the console subtests.

Here is an excerpt from `00_interface_sanity_console.rb` showing a subtest that does a handshake
with the kernel over the console:

```ruby
TIMEOUT_SECS = 3

# Verify sending and receiving works as expected.
class TxRxHandshake
    def name
        'Transmit and Receive handshake'
    end

    def run(qemu_out, qemu_in)
        qemu_in.write_nonblock('ABC')
        raise('TX/RX test failed') if qemu_out.expect('OK1234', TIMEOUT_SECS).nil?
    end
end
```

The subtest first sends `"ABC"` over the console to the kernel, and then expects to receive
`"OK1234"` back. On the kernel side, it looks like this in `00_interface_sanity_console.rs`:

```rust
#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod panic_exit_failure;

use libkernel::{bsp, console, print};

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use bsp::console::{console, qemu_bring_up_console};
    use console::interface::*;

    qemu_bring_up_console();

    // Handshake
    assert_eq!(console().read_char(), 'A');
    assert_eq!(console().read_char(), 'B');
    assert_eq!(console().read_char(), 'C');
    print!("OK1234");
```

## Test it

Believe it or not, that is all. There are three ways you can run tests:

  1. `make test` will run all tests back-to-back.
  2. `TEST=unit make test` will run `libkernel`'s unit tests.
  3. `TEST=TEST_NAME make test` will run a specficic integration test.
      - For example, `TEST=01_interface_sanity_timer make test`

```console
$ make test
[...]
RUSTFLAGS="-C link-arg=-Tsrc/bsp/rpi/link.ld -C target-cpu=cortex-a53 -D warnings -D missing_docs" cargo xtest --target=aarch64-unknown-none-softfloat --features bsp_rpi3 --release
    Finished release [optimized] target(s) in 0.01s
     Running target/aarch64-unknown-none-softfloat/release/deps/libkernel-e34f3f4734d1b219
         -------------------------------------------------------------------
         🦀 Running 5 tests
         -------------------------------------------------------------------

           1. test_runner_executes_in_kernel_mode.......................[ok]
           2. bss_section_is_sane.......................................[ok]
           3. virt_mem_layout_sections_are_64KiB_aligned................[ok]
           4. virt_mem_layout_has_no_overlaps...........................[ok]
           5. zero_volatile_works.......................................[ok]

         -------------------------------------------------------------------
         ✅ Success: libkernel
         -------------------------------------------------------------------


     Running target/aarch64-unknown-none-softfloat/release/deps/00_interface_sanity_console-fd36bc6543537769
         -------------------------------------------------------------------
         🦀 Running 3 console-based tests
         -------------------------------------------------------------------

           1. Transmit and Receive handshake............................[ok]
           2. Transmit statistics.......................................[ok]
           3. Receive statistics........................................[ok]

         -------------------------------------------------------------------
         ✅ Success: 00_interface_sanity_console
         -------------------------------------------------------------------


     Running target/aarch64-unknown-none-softfloat/release/deps/01_interface_sanity_timer-9ddd4857e51af91d
         -------------------------------------------------------------------
         🦀 Running 3 tests
         -------------------------------------------------------------------

           1. timer_is_counting.........................................[ok]
           2. timer_resolution_is_sufficient............................[ok]
           3. spin_accuracy_check_1_second..............................[ok]

         -------------------------------------------------------------------
         ✅ Success: 01_interface_sanity_timer
         -------------------------------------------------------------------


     Running target/aarch64-unknown-none-softfloat/release/deps/02_arch_exception_handling_sync_page_fault-8e8e460dd9041f11
         -------------------------------------------------------------------
         🦀 Testing synchronous exception handling by causing a page fault
         -------------------------------------------------------------------

         Writing beyond mapped area to address 9 GiB...

         Kernel panic:

         CPU Exception!
         FAR_EL1: 0x0000000240000000
         ESR_EL1: 0x96000004
         [...]

         -------------------------------------------------------------------
         ✅ Success: 02_arch_exception_handling_sync_page_fault
         -------------------------------------------------------------------
```

## Diff to previous
