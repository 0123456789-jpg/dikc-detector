//! Crate for finding bad Mac users.

#![warn(missing_docs)]
#![cfg(target_os = "macos")]

use std::fmt::Display;

use sysctl::{Ctl, Sysctl, SysctlError};

/// Errors which will occur when checking Mac quality.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The macOS version is not compliant with POSIX.
    NotPosix,
    /// The Mac model is bad.
    BadMacModel,
    /// Errors from [`sysctl`].
    Sysctl(SysctlError),
    /// Error when parsing macOS version.
    ParseOsVersion,
    /// Error variant that contains multiple errors.
    Many(Vec<Self>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotPosix => write!(f, "your macOS version is not compliant with POSIX, it is recommended to downgrade your macOS to a version prior to 14.4"),
            Error::BadMacModel => write!(f, "you have a bad taste, sell your Mac immediately and get a MacBook Pro (13-inch, M1, 2020)"),
            Error::Sysctl(err) => write!(f, "sysctl error: {}", err),
            Error::ParseOsVersion => write!(f, "your macOS version looks weird and can't be parsed"),
            Error::Many(errs) => {
                write!(f, "multiple errors: ")?;
                for err in errs {
                    write!(f, "{}, ", err)?;
                }
                Ok(())
            },
        }
    }
}

impl From<SysctlError> for Error {
    #[inline]
    fn from(value: SysctlError) -> Self {
        Self::Sysctl(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Sysctl(err) => Some(err),
            _ => None,
        }
    }
}

const HW_MODEL: &str = "hw.model";
const KERN_OSPRODUCTVERSION: &str = "kern.osproductversion";

/// Very bad machine.
const PULP_MACHINE: &str = "MacBookPro16,1";

/// Checks whether this Mac is bad.
///
/// # Errors
///
/// - Errors if macOS version is equal to or newer than __`14.4`__, which is not POSIX-compliant.
/// - Errors if the Mac model is `MacBookPro16,1`.
pub fn check() -> Result<(), Error> {
    let mut errs: Vec<Error> = Vec::with_capacity(2);
    if let Err(err) = check_posix() {
        errs.push(err);
    }
    if let Err(err) = check_machine() {
        errs.push(err);
    }
    if errs.is_empty() {
        Ok(())
    } else if let Some(err) = errs.pop().filter(|_| errs.is_empty()) {
        Err(err)
    } else {
        Err(Error::Many(errs))
    }
}

/// Checks whether macOS version is equal to or newer than __`14.4`__, which is not POSIX-compliant.
fn check_posix() -> Result<(), Error> {
    let ctl = Ctl::new(KERN_OSPRODUCTVERSION)?;
    let ver_str = ctl.value_string()?;
    let ver_split = ver_str.split('.');
    let mut is_sonoma = false;
    for num in ver_split {
        if !is_sonoma {
            match num.parse::<usize>().map_err(|_| Error::ParseOsVersion)? {
                ..=13 => return Ok(()),
                14 => is_sonoma = true,
                _ => return Err(Error::NotPosix),
            }
        } else if let ..=3 = num.parse::<usize>().map_err(|_| Error::ParseOsVersion)? {
            return Ok(());
        } else {
            return Err(Error::NotPosix);
        }
    }

    // Can't split version string by `.` because the loop doesn't run.
    Err(Error::ParseOsVersion)
}

fn check_machine() -> Result<(), Error> {
    let ctl = Ctl::new(HW_MODEL)?;
    if ctl.value_string()? == PULP_MACHINE {
        Err(Error::BadMacModel)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_decency() {
        assert!(crate::check().is_ok())
    }
}
