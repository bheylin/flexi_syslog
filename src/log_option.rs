use core::fmt;
use core::ops;

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct LogOption {
    pub bits: libc::c_int,
}

impl LogOption {
    /// Write directly to system console if there is an error while sending to system logger.
    pub const LOG_CONS: Self = Self {
        bits: libc::LOG_CONS,
    };

    /// Open the connection immediately (normally, the connection is opened when the first message is logged).
    pub const LOG_NDELAY: Self = Self {
        bits: libc::LOG_NDELAY,
    };

    /// Don't wait for child processes that may have been created while logging the message.
    /// The GNU C library does not create a child process, so this option has no effect on Linux.
    pub const LOG_NOWAIT: Self = Self {
        bits: libc::LOG_NOWAIT,
    };

    /// The converse of LOG_NDELAY; opening of the connection is delayed until syslog() is called.
    /// This is the default, and need not be specified.
    pub const LOG_ODELAY: Self = Self {
        bits: libc::LOG_ODELAY,
    };

    /// Print to stderr as well. (Not in POSIX.1-2001 or POSIX.1-2008.)
    pub const LOG_PERROR: Self = Self {
        bits: libc::LOG_PERROR,
    };

    /// Include PID with each message.
    pub const LOG_PID: Self = Self {
        bits: libc::LOG_PID,
    };

    #[inline]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns `true` if no flags are currently stored.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Returns `true` if any flags are currently stored.
    #[inline]
    pub const fn is_set(&self) -> bool {
        self.bits != 0
    }

    #[inline]
    pub const fn all() -> Self {
        Self {
            bits: Self::LOG_CONS.bits
                | Self::LOG_NDELAY.bits
                | Self::LOG_NOWAIT.bits
                | Self::LOG_ODELAY.bits
                | Self::LOG_PERROR.bits
                | Self::LOG_PID.bits,
        }
    }
}

impl fmt::Display for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut opts = vec![];

        if self.bits & Self::LOG_CONS.bits == Self::LOG_CONS.bits {
            opts.push("LOG_CONS");
        }
        if self.bits & Self::LOG_NDELAY.bits == Self::LOG_NDELAY.bits {
            opts.push("LOG_NDELAY");
        }
        if self.bits & Self::LOG_NOWAIT.bits == Self::LOG_NOWAIT.bits {
            opts.push("LOG_NOWAIT");
        }
        if self.bits & Self::LOG_ODELAY.bits == Self::LOG_ODELAY.bits {
            opts.push("LOG_ODELAY");
        }
        if self.bits & Self::LOG_PERROR.bits == Self::LOG_PERROR.bits {
            opts.push("LOG_PERROR");
        }
        if self.bits & Self::LOG_PID.bits == Self::LOG_PID.bits {
            opts.push("LOG_PID");
        }

        let s = opts.join(" | ");
        f.write_str(&s)
    }
}

impl fmt::Debug for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.bits)
    }
}

impl fmt::Binary for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.bits, f)
    }
}

impl fmt::Octal for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.bits, f)
    }
}

impl fmt::LowerHex for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.bits, f)
    }
}

impl fmt::UpperHex for LogOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.bits, f)
    }
}

impl ops::Deref for LogOption {
    type Target = libc::c_int;

    fn deref(&self) -> &Self::Target {
        &self.bits
    }
}

impl ops::BitOr for LogOption {
    type Output = Self;
    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, other: LogOption) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

impl ops::BitOrAssign for LogOption {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}

impl ops::BitXor for LogOption {
    type Output = Self;
    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }
}

impl ops::BitXorAssign for LogOption {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        self.bits ^= other.bits;
    }
}

impl ops::BitAnd for LogOption {
    type Output = Self;
    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }
}

impl ops::BitAndAssign for LogOption {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        self.bits &= other.bits;
    }
}

impl ops::Sub for LogOption {
    type Output = Self;
    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }
}

impl ops::SubAssign for LogOption {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.bits &= !other.bits;
    }
}

impl ops::Not for LogOption {
    type Output = Self;
    /// Returns the complement of this set of flags.
    #[inline]
    fn not(self) -> Self {
        Self { bits: !self.bits } & Self::all()
    }
}

#[cfg(test)]
mod test {
    use super::LogOption;

    #[test]
    fn should_display() {
        assert_eq!(&format!("{}", LogOption::LOG_CONS), "LOG_CONS");

        assert_eq!(
            &format!("{}", LogOption::LOG_CONS | LogOption::LOG_PID),
            "LOG_CONS | LOG_PID"
        );
    }
}
