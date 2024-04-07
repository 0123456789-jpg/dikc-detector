use std::{fmt::Display, num::ParseIntError};

use sysctl::{Ctl, Sysctl, SysctlError};

#[derive(Debug)]
pub enum Error {
    NotPosixCompliant,
    BadMacModel,
    SysctlError(SysctlError),
    ParseOsVersionError,
}

impl Display for Error {
    /// Human-readable error descriptions.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPosixCompliant=>write!(f, "Your macOS version is not compliant with POSIX, it is recommended to downgrade your macOS to a version prior to 14.4."),
            Self::BadMacModel=>write!(f, "You have a bad taste, sell your Mac immediately and get a MacBook Pro (13-inch, M1, 2020)."),
            Self::SysctlError(err)=>write!(f, "Sysctl error: {}.", err),
            Self::ParseOsVersionError=>write!(f, "Your macOS version looks weird and can't be parsed."),
        }
    }
}

impl From<SysctlError> for Error {
    fn from(value: SysctlError) -> Self {
        Self::SysctlError(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(_value: ParseIntError) -> Self {
        Self::ParseOsVersionError
    }
}

const HW_MODEL: &str = "hw.model";
const KERN_OSPRODUCTVERSION: &str = "kern.osproductversion";

/// Very bad machine.
const PULP_MACHINE: &str = "MacBookPro16.1";

#[cfg(target_os = "macos")]
pub fn check() -> Result<(), Vec<Error>> {
    let mut err: Vec<Error> = Vec::with_capacity(2);
    let _ = check_posix().is_err_and(|e| {
        err.push(e);
        true
    });
    let _ = check_machine().is_err_and(|e| {
        err.push(e);
        true
    });
    if err.is_empty() {
        Ok(())
    } else {
        Err(err)
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
            match num.parse::<usize>()? {
                ..=13 => return Ok(()),
                14 => is_sonoma = true,
                _ => return Err(Error::NotPosixCompliant),
            }
        } else {
            match num.parse::<usize>()? {
                ..=3 => return Ok(()),
                _ => return Err(Error::NotPosixCompliant),
            }
        }
    }

    // Can't split version string by `.` because the loop doesn't run.
    Err(Error::ParseOsVersionError)
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
        crate::check().unwrap()
    }
}
