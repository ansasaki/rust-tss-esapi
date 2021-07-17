// Copyright 2021 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
use crate::{
    constants::CapabilityType,
    structures::{CapabilityData, PublicParameters},
    tss2_esys::*,
    Context, Error, Result, WrapperErrorKind as ErrorKind,
};
use log::{error, warn};
use mbox::MBox;
use std::convert::TryFrom;
use std::ptr::null_mut;

impl Context {
    /// Get current capability information about the TPM.
    pub fn get_capability(
        &mut self,
        capability: CapabilityType,
        property: u32,
        property_count: u32,
    ) -> Result<(CapabilityData, bool)> {
        let mut outcapabilitydata = null_mut();
        let mut outmoredata: u8 = 0;

        let ret = unsafe {
            Esys_GetCapability(
                self.mut_context(),
                self.optional_session_1(),
                self.optional_session_2(),
                self.optional_session_3(),
                capability.into(),
                property,
                property_count,
                &mut outmoredata,
                &mut outcapabilitydata,
            )
        };
        let moredata = if outmoredata == 0 {
            false
        } else if outmoredata == 1 {
            true
        } else {
            return Err(Error::WrapperError(ErrorKind::WrongValueFromTpm));
        };
        let ret = Error::from_tss_rc(ret);

        if ret.is_success() {
            let capabilitydata = unsafe { MBox::from_raw(outcapabilitydata) };
            let capabilities = CapabilityData::try_from(*capabilitydata)?;
            Ok((capabilities, moredata))
        } else {
            error!("Error when getting capabilities: {}", ret);
            Err(ret)
        }
    }

    /// Test if the given parameters are supported by the TPM.
    ///
    /// # Errors
    /// * if any of the public parameters is not compatible with the TPM,
    /// an `Err` containing the specific unmarshalling error will be returned.
    pub fn test_parms(&mut self, public_parmeters: PublicParameters) -> Result<()> {
        let ret = unsafe {
            Esys_TestParms(
                self.mut_context(),
                self.optional_session_1(),
                self.optional_session_2(),
                self.optional_session_3(),
                &public_parmeters.into(),
            )
        };

        let ret = Error::from_tss_rc(ret);
        if ret.is_success() {
            Ok(())
        } else {
            warn!("Parameters under test could not be unmarshalled: {}", ret);
            Err(ret)
        }
    }
}
