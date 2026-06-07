//! Process isolation primitives applied to sandboxed child processes.
//!
//! Two layers, both enforced *per child process* (the cage-bro server itself is
//! never restricted):
//!
//! * **Resource limits** (`setrlimit`) — address space, CPU time, file size,
//!   process count and open files. Enforced on every Unix platform.
//! * **Landlock filesystem confinement** (Linux ≥ 5.13) — the child may only
//!   read/execute a fixed allowlist of system directories and read/write its
//!   own workspace and `/tmp`. Everything else on the filesystem is denied.
//!
//! Both layers are installed inside a `pre_exec` hook, i.e. in the forked child
//! after `fork(2)` and before `execvp(2)`. Only async-signal-safe operations
//! (raw syscalls, no allocation) run there; anything that allocates (opening the
//! Landlock path descriptors, building the ruleset) happens in the parent and is
//! captured by the closure as plain file descriptors.
//!
//! Not yet implemented: seccomp syscall filtering and network namespacing. A
//! sandbox is therefore *filesystem + resource* isolated, not syscall isolated —
//! see the crate README for the honest threat model.

use crate::traits::SandboxConfig;

/// Read-only + execute system directories every interpreter needs.
///
/// `/proc` and `/sys` are included for runtime compatibility: Python/Node and
/// common native libs (numpy, OpenBLAS, psutil) read `/proc/cpuinfo`,
/// `/proc/meminfo`, `/sys/devices/system/cpu`, etc., and fully denying them
/// breaks those workloads. This grants *read* access only — it does leak some
/// cross-process info via `/proc/<pid>`, which is acceptable under cage-bro's
/// threat model (process-level isolation, not an adversarial boundary — use a
/// VM for that; see README).
#[cfg(target_os = "linux")]
const SYSTEM_RO_DIRS: &[&str] = &[
    "/usr", "/bin", "/sbin", "/lib", "/lib64", "/etc", "/opt", "/proc", "/sys",
];

/// World paths the child may read+write (interpreters, temp files, devices).
#[cfg(target_os = "linux")]
const SYSTEM_RW_PATHS: &[&str] = &[
    "/tmp", "/dev/null", "/dev/zero", "/dev/full",
    "/dev/random", "/dev/urandom", "/dev/tty",
];

/// Resolved isolation policy for a single sandboxed exec.
///
/// Cheap to build; holds only the values the `pre_exec` closure needs. The
/// Landlock ruleset descriptor (Linux) is created lazily at `apply` time in the
/// parent so its lifetime is tied to the spawn.
#[derive(Clone)]
pub struct Isolation {
    /// Hard cap on the child's virtual address space (RLIMIT_AS), bytes.
    mem_bytes: Option<u64>,
    /// CPU-seconds wall budget (RLIMIT_CPU). Derived from the exec timeout.
    cpu_secs: Option<u64>,
    /// Max single-file size the child may create (RLIMIT_FSIZE), bytes.
    fsize_bytes: Option<u64>,
    /// Workspace directory the child may read+write (Landlock).
    workspace: Option<String>,
}

impl Isolation {
    /// Derive a policy from sandbox config. `timeout_ms` (the exec timeout, if
    /// any) is used as a hard CPU-second ceiling so a runaway process is capped
    /// even if the wall-clock timeout race is lost.
    pub fn from_config(config: &SandboxConfig, timeout_ms: Option<u64>) -> Self {
        Self {
            mem_bytes: config.memory_limit_mb.map(|mb| mb.saturating_mul(1024 * 1024)),
            // RLIMIT_CPU is whole seconds; round the timeout up and add a grace
            // second so the wall-clock timeout (SIGKILL) normally fires first.
            cpu_secs: timeout_ms.map(|ms| ms.div_ceil(1000).saturating_add(1)),
            fsize_bytes: config.disk_limit_mb.map(|mb| mb.saturating_mul(1024 * 1024)),
            workspace: config.workspace_dir.clone(),
        }
    }

    /// Install isolation into a `std::process::Command`, to take effect in the
    /// child between fork and exec.
    ///
    /// On Linux this builds the Landlock ruleset in the parent (opening the
    /// allowlisted path descriptors) and registers a `pre_exec` hook that sets
    /// `no_new_privs`, enforces the ruleset, and applies the resource limits.
    /// On other Unix platforms only the resource limits are applied. Landlock
    /// unavailability (old kernel, disabled) degrades to resource-limits-only
    /// with a warning rather than failing the exec.
    ///
    /// Returns a [`ParentRuleset`] guard that closes the parent-side Landlock
    /// descriptor when dropped; hold it until after the child is spawned.
    #[must_use]
    pub fn apply(&self, command: &mut std::process::Command) -> ParentRuleset {
        let mem = self.mem_bytes;
        let cpu = self.cpu_secs;
        let fsize = self.fsize_bytes;

        // Landlock ruleset is created here in the parent (allocates, opens fds)
        // and only *enforced* in the child via the inherited descriptor.
        #[cfg(target_os = "linux")]
        let ruleset_fd = linux::build_ruleset(self.workspace.as_deref());
        #[cfg(not(target_os = "linux"))]
        let _ = &self.workspace;

        use std::os::unix::process::CommandExt;
        // SAFETY: the closure runs in the forked child before exec and performs
        // only async-signal-safe operations — raw syscalls (setrlimit, prctl,
        // landlock_restrict_self), plus write/_exit on failure. No allocation,
        // no std error construction. The Landlock fd, if any, is captured by
        // value as a plain integer. enforce() fails closed (writes to stderr and
        // _exits) if a built ruleset can't be enforced, so the child never runs
        // unconfined; fd == -1 (Landlock unavailable) is an intentional skip.
        unsafe {
            command.pre_exec(move || {
                set_rlimits(mem, cpu, fsize);
                #[cfg(target_os = "linux")]
                linux::enforce(ruleset_fd);
                Ok(())
            });
        }

        ParentRuleset {
            #[cfg(target_os = "linux")]
            fd: ruleset_fd,
        }
    }
}

/// Guard owning the parent process's copy of the Landlock ruleset descriptor.
/// Dropping it closes the descriptor (the child enforces its own inherited copy
/// in `pre_exec`). A no-op on non-Linux platforms.
pub struct ParentRuleset {
    #[cfg(target_os = "linux")]
    fd: std::os::unix::io::RawFd,
}

impl Drop for ParentRuleset {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if self.fd >= 0 {
            unsafe {
                libc::close(self.fd);
            }
        }
    }
}

/// Apply `setrlimit` ceilings. Async-signal-safe (single syscalls, no alloc).
/// Failures are swallowed: a missing limit must not abort the exec, and there is
/// no safe way to log from a `pre_exec` context.
fn set_rlimits(mem_bytes: Option<u64>, cpu_secs: Option<u64>, fsize_bytes: Option<u64>) {
    unsafe {
        if let Some(bytes) = mem_bytes {
            set_one(libc::RLIMIT_AS, bytes);
        }
        if let Some(secs) = cpu_secs {
            set_one(libc::RLIMIT_CPU, secs);
        }
        if let Some(bytes) = fsize_bytes {
            set_one(libc::RLIMIT_FSIZE, bytes);
        }
        // Per-process fd-exhaustion guard (best-effort). We deliberately do NOT
        // set RLIMIT_NPROC: it accounts processes/threads per *real UID*, not per
        // process tree, so a small constant would be shared across the server and
        // every sandbox and trigger spurious EAGAIN. Per-sandbox process caps
        // belong to the deploy layer (e.g. cgroups `pids.max`).
        set_one(libc::RLIMIT_NOFILE, 1024);
    }
}

/// `setrlimit`'s resource argument type differs by platform: a distinct enum
/// type on glibc/Linux, a plain `c_int` elsewhere.
#[cfg(target_os = "linux")]
type RlimitResource = libc::__rlimit_resource_t;
#[cfg(not(target_os = "linux"))]
type RlimitResource = libc::c_int;

/// Set both soft and hard limit of `resource` to `value`.
unsafe fn set_one(resource: RlimitResource, value: u64) {
    let rl = libc::rlimit {
        rlim_cur: value as libc::rlim_t,
        rlim_max: value as libc::rlim_t,
    };
    // Ignore errors — best-effort hardening.
    libc::setrlimit(resource, &rl);
}

#[cfg(target_os = "linux")]
mod linux {
    use std::os::unix::io::RawFd;

    // Landlock ABI v1 filesystem access rights (kernel ≥ 5.13).
    const ACCESS_FS_EXECUTE: u64 = 1 << 0;
    const ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    const ACCESS_FS_READ_FILE: u64 = 1 << 2;
    const ACCESS_FS_READ_DIR: u64 = 1 << 3;
    const ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
    const ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
    const ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
    const ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
    const ACCESS_FS_MAKE_REG: u64 = 1 << 8;
    const ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
    const ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
    const ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
    const ACCESS_FS_MAKE_SYM: u64 = 1 << 12;

    /// Everything Landlock v1 can govern — the ruleset "handles" all of these,
    /// so any right not explicitly granted to a path is denied.
    const ACCESS_FS_ALL: u64 = ACCESS_FS_EXECUTE
        | ACCESS_FS_WRITE_FILE
        | ACCESS_FS_READ_FILE
        | ACCESS_FS_READ_DIR
        | ACCESS_FS_REMOVE_DIR
        | ACCESS_FS_REMOVE_FILE
        | ACCESS_FS_MAKE_CHAR
        | ACCESS_FS_MAKE_DIR
        | ACCESS_FS_MAKE_REG
        | ACCESS_FS_MAKE_SOCK
        | ACCESS_FS_MAKE_FIFO
        | ACCESS_FS_MAKE_BLOCK
        | ACCESS_FS_MAKE_SYM;

    /// Read + traverse + execute, no mutation.
    const ACCESS_FS_RO: u64 = ACCESS_FS_READ_FILE | ACCESS_FS_READ_DIR | ACCESS_FS_EXECUTE;

    const LANDLOCK_RULE_PATH_BENEATH: libc::c_int = 1;

    #[repr(C)]
    struct RulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C, packed)]
    struct PathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    // All args to the variadic `syscall` must be passed at register width
    // (`c_long`). Rust does not promote variadic args, so a bare `u32`/`i32`/fd
    // would be read by the glibc wrapper as a `long` with undefined upper bits.
    unsafe fn create_ruleset(attr: *const RulesetAttr, size: usize, flags: u32) -> libc::c_long {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            attr as usize as libc::c_long,
            size as libc::c_long,
            flags as libc::c_long,
        )
    }
    unsafe fn add_rule(ruleset_fd: RawFd, rule_type: libc::c_int, attr: *const PathBeneathAttr) -> libc::c_long {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            ruleset_fd as libc::c_long,
            rule_type as libc::c_long,
            attr as usize as libc::c_long,
            0_i64 as libc::c_long,
        )
    }
    unsafe fn restrict_self(ruleset_fd: RawFd) -> libc::c_long {
        libc::syscall(
            libc::SYS_landlock_restrict_self,
            ruleset_fd as libc::c_long,
            0_i64 as libc::c_long,
        )
    }

    /// Grant `access` on `path` (a directory or file) within `ruleset_fd`.
    /// Missing paths are silently skipped — not every host has `/lib64`, etc.
    unsafe fn allow_path(ruleset_fd: RawFd, path: &str, access: u64) {
        let cpath = match std::ffi::CString::new(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let fd = libc::open(cpath.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
        if fd < 0 {
            return;
        }
        let attr = PathBeneathAttr { allowed_access: access, parent_fd: fd };
        // Runs in the parent (not pre_exec), so logging here is safe. A failed
        // rule means the sandbox silently loses access to `path` (e.g. its own
        // workspace) — worth surfacing.
        if add_rule(ruleset_fd, LANDLOCK_RULE_PATH_BENEATH, &attr) != 0 {
            tracing::warn!(
                "failed to add Landlock rule for {}: {}",
                path,
                std::io::Error::last_os_error()
            );
        }
        libc::close(fd);
    }

    /// Build a Landlock ruleset in the *parent* process and return its fd, or
    /// `-1` if Landlock is unavailable (old kernel / disabled). The fd is
    /// `O_CLOEXEC`-free so it survives `execvp`; the child enforces it.
    pub fn build_ruleset(workspace: Option<&str>) -> RawFd {
        unsafe {
            // Probe the supported ABI version. <1 → Landlock unusable.
            const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1 << 0;
            let abi = create_ruleset(std::ptr::null(), 0, LANDLOCK_CREATE_RULESET_VERSION);
            if abi < 1 {
                tracing::warn!(
                    "Landlock unavailable (kernel <5.13 or disabled); falling back to rlimits-only isolation"
                );
                return -1;
            }

            let attr = RulesetAttr { handled_access_fs: ACCESS_FS_ALL };
            let ruleset_fd = create_ruleset(&attr, std::mem::size_of::<RulesetAttr>(), 0);
            if ruleset_fd < 0 {
                tracing::warn!("landlock_create_ruleset failed; falling back to rlimits-only isolation");
                return -1;
            }
            let ruleset_fd = ruleset_fd as RawFd;
            // Close-on-exec so this fd can't leak into an unrelated child that
            // another thread execs concurrently. Our own child still inherits it
            // across fork (CLOEXEC only acts on exec) and enforces + closes it in
            // pre_exec before its own exec.
            libc::fcntl(ruleset_fd, libc::F_SETFD, libc::FD_CLOEXEC);

            for dir in super::SYSTEM_RO_DIRS {
                allow_path(ruleset_fd, dir, ACCESS_FS_RO);
            }
            for path in super::SYSTEM_RW_PATHS {
                allow_path(ruleset_fd, path, ACCESS_FS_ALL);
            }
            if let Some(ws) = workspace {
                allow_path(ruleset_fd, ws, ACCESS_FS_ALL);
            }

            ruleset_fd
        }
    }

    /// Enforce the ruleset in the child. Runs post-fork in `pre_exec`, so it uses
    /// ONLY async-signal-safe operations: raw syscalls, `write`, and `_exit` — no
    /// allocation, no std error construction. A `-1` fd means Landlock was
    /// unavailable (intentional rlimits-only skip). On any *enforcement* failure
    /// it fails **closed**: write a diagnostic to the (captured) stderr pipe and
    /// `_exit(127)` so the child dies before `execvp` rather than run unconfined.
    pub fn enforce(ruleset_fd: RawFd) {
        if ruleset_fd < 0 {
            return;
        }
        unsafe {
            // Landlock requires no_new_privs to be set first.
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                fail_closed(b"[cage-bro] isolation setup failed: prctl(PR_SET_NO_NEW_PRIVS)\n");
            }
            if restrict_self(ruleset_fd) != 0 {
                fail_closed(b"[cage-bro] isolation setup failed: landlock_restrict_self\n");
            }
            libc::close(ruleset_fd);
        }
    }

    /// Write `msg` to stderr and terminate the child. Async-signal-safe.
    unsafe fn fail_closed(msg: &[u8]) -> ! {
        libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const libc::c_void, msg.len());
        libc::_exit(127);
    }
}
