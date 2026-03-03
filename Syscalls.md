# m5rOS System Call Reference

## Overview

System calls provide the interface between user programs and the kernel. They allow user programs to request services from the kernel such as file I/O, process management, and hardware access.

**Status**: Design phase - Implementation pending

## Calling Convention

### x86_64 System Call ABI

m5rOS uses the `syscall` instruction (x86_64 fast system call).

**Registers:**
- `RAX` - System call number (input) and return value (output)
- `RDI` - Argument 1
- `RSI` - Argument 2
- `RDX` - Argument 3
- `R10` - Argument 4 (note: not RCX, which is used by syscall instruction)
- `R8`  - Argument 5
- `R9`  - Argument 6

**Return Values:**
- Success: Non-negative value (often 0 or count of bytes/items)
- Error: Negative value encoding error code

**Preserved Registers:**
- `RBX`, `RBP`, `R12`, `R13`, `R14`, `R15` are callee-saved

**Clobbered Registers:**
- `RCX` (saved by syscall instruction, contains return address)
- `R11` (saved by syscall instruction, contains RFLAGS)

## System Call Table

### File Operations

#### read - Read from File Descriptor
```c
ssize_t read(int fd, void *buf, size_t count);
```

**Number**: 0

**Arguments:**
- `fd` - File descriptor to read from
- `buf` - Buffer to read data into
- `count` - Maximum number of bytes to read

**Returns:**
- Number of bytes read (0 = EOF)
- `-EBADF` - Invalid file descriptor
- `-EFAULT` - buf points to invalid memory
- `-EINVAL` - Invalid count
- `-EIO` - I/O error

**Description**: Reads up to `count` bytes from file descriptor `fd` into buffer `buf`.

---

#### write - Write to File Descriptor
```c
ssize_t write(int fd, const void *buf, size_t count);
```

**Number**: 1

**Arguments:**
- `fd` - File descriptor to write to
- `buf` - Buffer containing data to write
- `count` - Number of bytes to write

**Returns:**
- Number of bytes written
- `-EBADF` - Invalid file descriptor
- `-EFAULT` - buf points to invalid memory
- `-EINVAL` - Invalid count
- `-EIO` - I/O error
- `-ENOSPC` - No space left on device

**Description**: Writes up to `count` bytes from buffer `buf` to file descriptor `fd`.

---

#### open - Open File
```c
int open(const char *pathname, int flags, mode_t mode);
```

**Number**: 2

**Arguments:**
- `pathname` - Path to file to open
- `flags` - Open flags (O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_TRUNC, O_APPEND)
- `mode` - Permission mode if creating file

**Returns:**
- File descriptor (non-negative)
- `-EACCES` - Permission denied
- `-EEXIST` - File exists and O_CREAT | O_EXCL was used
- `-EFAULT` - pathname points to invalid memory
- `-EINVAL` - Invalid flags
- `-EMFILE` - Process file descriptor limit reached
- `-ENFILE` - System file table full
- `-ENOENT` - File does not exist
- `-ENOMEM` - Insufficient memory

**Description**: Opens the file specified by `pathname` and returns a file descriptor.

---

#### close - Close File Descriptor
```c
int close(int fd);
```

**Number**: 3

**Arguments:**
- `fd` - File descriptor to close

**Returns:**
- 0 on success
- `-EBADF` - Invalid file descriptor

**Description**: Closes the file descriptor `fd`.

---

### Process Management

#### fork - Create Child Process
```c
pid_t fork(void);
```

**Number**: 10

**Arguments**: None

**Returns:**
- 0 in child process
- PID of child in parent process
- `-ENOMEM` - Insufficient memory
- `-EAGAIN` - Cannot fork (process limit reached)

**Description**: Creates a new process by duplicating the calling process. The child is an exact copy except for the return value.

---

#### exec - Execute Program
```c
int exec(const char *pathname, char *const argv[], char *const envp[]);
```

**Number**: 11

**Arguments:**
- `pathname` - Path to executable
- `argv` - Argument vector (NULL-terminated)
- `envp` - Environment vector (NULL-terminated)

**Returns:**
- Does not return on success
- `-EACCES` - Permission denied or not executable
- `-EFAULT` - Invalid pointer
- `-EINVAL` - Invalid ELF file
- `-ENOENT` - File does not exist
- `-ENOMEM` - Insufficient memory
- `-ENOTDIR` - Component of path is not directory

**Description**: Replaces current process image with new program loaded from `pathname`.

---

#### exit - Terminate Process
```c
void exit(int status);
```

**Number**: 12

**Arguments:**
- `status` - Exit status code (0-255)

**Returns**: Never returns

**Description**: Terminates the calling process and returns `status` to parent.

---

#### wait - Wait for Child Process
```c
pid_t wait(int *status);
```

**Number**: 13

**Arguments:**
- `status` - Pointer to store child exit status

**Returns:**
- PID of terminated child
- `-ECHILD` - No child processes
- `-EFAULT` - status points to invalid memory

**Description**: Waits for a child process to terminate and retrieves its exit status.

---

#### getpid - Get Process ID
```c
pid_t getpid(void);
```

**Number**: 14

**Arguments**: None

**Returns**: Process ID of calling process (always succeeds)

**Description**: Returns the process ID (PID) of the calling process.

---

#### getppid - Get Parent Process ID
```c
pid_t getppid(void);
```

**Number**: 15

**Arguments**: None

**Returns**: Parent process ID (always succeeds)

**Description**: Returns the process ID of the parent of the calling process.

---

### Signals

#### kill - Send Signal to Process
```c
int kill(pid_t pid, int sig);
```

**Number**: 20

**Arguments:**
- `pid` - Process ID to send signal to
- `sig` - Signal number

**Returns:**
- 0 on success
- `-EINVAL` - Invalid signal
- `-EPERM` - Permission denied
- `-ESRCH` - No such process

**Description**: Sends signal `sig` to process `pid`.

---

### Memory Management

#### brk - Change Data Segment Size
```c
int brk(void *addr);
```

**Number**: 30

**Arguments:**
- `addr` - New program break address

**Returns:**
- 0 on success
- `-ENOMEM` - Insufficient memory

**Description**: Sets the program break to `addr`, effectively changing heap size.

---

#### mmap - Map Memory
```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
```

**Number**: 31

**Arguments:**
- `addr` - Preferred address (0 = kernel chooses)
- `length` - Size of mapping
- `prot` - Protection flags (PROT_READ, PROT_WRITE, PROT_EXEC)
- `flags` - Mapping flags (MAP_SHARED, MAP_PRIVATE, MAP_ANONYMOUS)
- `fd` - File descriptor (for file-backed mapping)
- `offset` - Offset in file

**Returns:**
- Address of mapping on success
- `-EACCES` - Permission denied
- `-EBADF` - Invalid file descriptor
- `-EINVAL` - Invalid arguments
- `-ENOMEM` - Insufficient memory

**Description**: Creates a memory mapping at the specified address.

---

#### munmap - Unmap Memory
```c
int munmap(void *addr, size_t length);
```

**Number**: 32

**Arguments:**
- `addr` - Start of mapping
- `length` - Size of mapping

**Returns:**
- 0 on success
- `-EINVAL` - Invalid address or length

**Description**: Removes the mapping at the specified address.

---

### Directory Operations

#### chdir - Change Directory
```c
int chdir(const char *path);
```

**Number**: 40

**Arguments:**
- `path` - Path to new directory

**Returns:**
- 0 on success
- `-EACCES` - Permission denied
- `-EFAULT` - path points to invalid memory
- `-ENOENT` - Directory does not exist
- `-ENOTDIR` - Component is not a directory

**Description**: Changes the current working directory to `path`.

---

#### getcwd - Get Current Directory
```c
char *getcwd(char *buf, size_t size);
```

**Number**: 41

**Arguments:**
- `buf` - Buffer to store path
- `size` - Size of buffer

**Returns:**
- Pointer to buf on success
- `-EFAULT` - buf points to invalid memory
- `-EINVAL` - size is zero
- `-ERANGE` - buf is too small

**Description**: Stores absolute path of current directory in `buf`.

---

#### mkdir - Create Directory
```c
int mkdir(const char *pathname, mode_t mode);
```

**Number**: 42

**Arguments:**
- `pathname` - Path of directory to create
- `mode` - Permission mode

**Returns:**
- 0 on success
- `-EACCES` - Permission denied
- `-EEXIST` - Directory already exists
- `-EFAULT` - pathname points to invalid memory
- `-ENOENT` - Parent directory does not exist
- `-ENOSPC` - No space left on device

**Description**: Creates a new directory at `pathname`.

---

#### rmdir - Remove Directory
```c
int rmdir(const char *pathname);
```

**Number**: 43

**Arguments:**
- `pathname` - Path of directory to remove

**Returns:**
- 0 on success
- `-EACCES` - Permission denied
- `-EBUSY` - Directory is in use
- `-EFAULT` - pathname points to invalid memory
- `-ENOENT` - Directory does not exist
- `-ENOTDIR` - Component is not a directory
- `-ENOTEMPTY` - Directory is not empty

**Description**: Removes the directory at `pathname` (must be empty).

---

### Time

#### time - Get Current Time
```c
time_t time(time_t *tloc);
```

**Number**: 50

**Arguments:**
- `tloc` - Optional pointer to store time

**Returns:**
- Current time in seconds since epoch
- `-EFAULT` - tloc points to invalid memory

**Description**: Returns current time. If `tloc` is not NULL, also stores time there.

---

#### nanosleep - Sleep for Specified Time
```c
int nanosleep(const struct timespec *req, struct timespec *rem);
```

**Number**: 51

**Arguments:**
- `req` - Requested sleep duration
- `rem` - Remaining time if interrupted (optional)

**Returns:**
- 0 on success
- `-EFAULT` - Invalid pointer
- `-EINTR` - Interrupted by signal
- `-EINVAL` - Invalid time value

**Description**: Suspends execution for the time specified in `req`.

---

## Error Codes

All system calls return negative error codes on failure:

| Code | Name | Description |
|------|------|-------------|
| -1 | EPERM | Operation not permitted |
| -2 | ENOENT | No such file or directory |
| -3 | ESRCH | No such process |
| -4 | EINTR | Interrupted system call |
| -5 | EIO | I/O error |
| -6 | ENXIO | No such device or address |
| -7 | E2BIG | Argument list too long |
| -8 | ENOEXEC | Exec format error |
| -9 | EBADF | Bad file descriptor |
| -10 | ECHILD | No child processes |
| -11 | EAGAIN | Try again |
| -12 | ENOMEM | Out of memory |
| -13 | EACCES | Permission denied |
| -14 | EFAULT | Bad address |
| -15 | ENOTBLK | Block device required |
| -16 | EBUSY | Device or resource busy |
| -17 | EEXIST | File exists |
| -18 | EXDEV | Cross-device link |
| -19 | ENODEV | No such device |
| -20 | ENOTDIR | Not a directory |
| -21 | EISDIR | Is a directory |
| -22 | EINVAL | Invalid argument |
| -23 | ENFILE | File table overflow |
| -24 | EMFILE | Too many open files |
| -25 | ENOTTY | Not a typewriter |
| -26 | ETXTBSY | Text file busy |
| -27 | EFBIG | File too large |
| -28 | ENOSPC | No space left on device |
| -29 | ESPIPE | Illegal seek |
| -30 | EROFS | Read-only file system |
| -31 | EMLINK | Too many links |
| -32 | EPIPE | Broken pipe |
| -33 | EDOM | Math argument out of domain |
| -34 | ERANGE | Math result not representable |

## Usage from C

### Syscall Wrapper

```c
// In libc/syscall.c
long syscall(long number, ...) {
    long ret;
    register long rax asm("rax") = number;
    register long rdi asm("rdi") = arg1;
    register long rsi asm("rsi") = arg2;
    register long rdx asm("rdx") = arg3;
    register long r10 asm("r10") = arg4;
    register long r8 asm("r8") = arg5;
    register long r9 asm("r9") = arg6;

    asm volatile (
        "syscall"
        : "=a"(ret)
        : "0"(rax), "r"(rdi), "r"(rsi), "r"(rdx), "r"(r10), "r"(r8), "r"(r9)
        : "rcx", "r11", "memory"
    );

    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}
```

### Example Usage

```c
// Open file
int fd = open("/home/user/file.txt", O_RDONLY, 0);
if (fd < 0) {
    perror("open");
    return -1;
}

// Read data
char buf[256];
ssize_t n = read(fd, buf, sizeof(buf));
if (n < 0) {
    perror("read");
    close(fd);
    return -1;
}

// Close file
close(fd);
```

## Kernel Implementation

### System Call Dispatcher

```rust
// In kernel/src/syscall/mod.rs
pub fn syscall_handler(
    number: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    match number {
        0 => sys_read(arg1 as i32, arg2 as *mut u8, arg3),
        1 => sys_write(arg1 as i32, arg2 as *const u8, arg3),
        2 => sys_open(arg1 as *const u8, arg2 as i32, arg3 as u32),
        // ... more syscalls
        _ => -ENOSYS, // Function not implemented
    }
}
```

### Argument Validation

All arguments from userspace must be validated:

```rust
fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
    // Validate file descriptor
    let file = match current_process().get_file(fd) {
        Some(f) => f,
        None => return -EBADF,
    };

    // Validate buffer is in userspace and writable
    if !is_user_writable(buf, count) {
        return -EFAULT;
    }

    // Perform the read
    file.read(buf, count)
}
```

## Security Considerations

1. **Argument Validation**: All pointers and values from userspace must be validated
2. **Buffer Checks**: Ensure buffers are within user address space
3. **Integer Overflow**: Check for overflow in size calculations
4. **Race Conditions**: Handle TOCTOU (time-of-check-time-of-use) issues
5. **Resource Limits**: Enforce per-process limits (open files, memory, etc.)
6. **Capability Checks**: Verify process has permission for operation

## Future System Calls

Additional syscalls planned for future phases:

- `socket`, `bind`, `listen`, `accept`, `connect` - Networking
- `sendmsg`, `recvmsg` - Network I/O
- `select`, `poll`, `epoll` - I/O multiplexing
- `ioctl` - Device control
- `mount`, `umount` - Filesystem mounting
- `setuid`, `setgid` - Change user/group ID
- `sigaction`, `sigprocmask` - Signal handling
- `clone` - Advanced process creation
- `futex` - Fast userspace mutex

## References

- Linux System Call Reference: https://man7.org/linux/man-pages/
- Intel® 64 and IA-32 Architectures Software Developer's Manual
- "UNIX Systems for Modern Architectures" by Curt Schimmel
- "The Linux Programming Interface" by Michael Kerrisk
