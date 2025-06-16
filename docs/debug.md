```shell
export RUSTFLAGS="-C opt-level=0 -g -Zmacro-backtrace"
lldb target/deps/path-to-binary
(lldb) br set -n malloc_error_break
(lldb) process launch
(lldb) up
```
