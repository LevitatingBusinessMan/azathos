/// Don't completely trust these.
#[repr(i32)]
#[derive(Debug)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGSTKFLT,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPWR,
    SIGSYS,
}

impl TryFrom<i32> for Signal {
    type Error = ();

    fn try_from(sig: i32) -> Result<Self, Self::Error> {
        if sig >= Signal::SIGHUP as i32 && sig <= Signal::SIGSYS as i32 {
            Ok(unsafe { std::mem::transmute(sig) })
        } else {
            Err(())
        }
    }
}
