# Check-only stand-ins

Used by `scripts/check-only-clippy.sh`: a C compiler (`bin/fakecc`) and archiver (`bin/fakear`) that write the files
they are asked for and compile nothing, a `pkg-config` (`bin/fakepkgconfig`) that knows every package, and stub libc
headers (`inc/`) for bindgen. They let clippy type-check Rust code for a target whose C toolchain or system libraries
this machine lacks. Nothing built with them links or runs.
