// nvm-core/src/isa/register.rs
//
//! # NVM registers
//!
//! This module defines the register type of the virtual machine.
//!
//! A register is an identifier of one of the 255 general-purpose
//! registers. The register itself **does not store a value** —
//! it only indicates which register to access.
//!
//! Register values are part of the virtual machine state
//! and are stored separately.
use std::fmt::{self, Display, Formatter};

/// The register identifier.
///
/// NVM provides 255 registers, so one byte is enough
/// to store their number.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Register(pub u8);

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "R{}", self.0)
    }
}
