use crate::ffi::raw as ffi;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::Path;
use log::debug;

use crate::{
    node::{HoudiniNode, NodeHandle},
    session::Session,
    errors::Result,
    stringhandle::*,
    ffi::{ParmInfo, AssetInfo},
};

#[derive(Debug, Clone)]
pub struct AssetLibrary {
    lib_id: ffi::HAPI_AssetLibraryId,
    session: Session,
}

impl AssetLibrary {
    pub fn from_file(session: Session, file: impl AsRef<str>) -> Result<AssetLibrary> {
        debug!("Loading library: {}", file.as_ref());
        let cs = CString::new(file.as_ref())?;
        let lib_id = crate::ffi::load_library_from_file(&cs, &session, true)?;
        Ok(AssetLibrary { lib_id, session })
    }

    pub fn get_asset_count(&self) -> Result<i32> {
        crate::ffi::get_asset_count(self.lib_id, &self.session)
    }

    pub fn get_asset_names(&self) -> Result<Vec<String>> {
        let num_assets = self.get_asset_count()?;
        crate::ffi::get_asset_names(self.lib_id, num_assets, &self.session)
    }

    pub fn get_asset_parms(&self, asset_name: impl AsRef<str>) -> Result<Vec<ParmInfo<'_>>> {
        unimplemented!("This is crashing HARS server");
    }
}

impl<'node> AssetInfo<'node> {
    pub fn new(node: &'node HoudiniNode) -> Result<AssetInfo<'_>> {
        Ok(AssetInfo {
            inner: crate::ffi::get_asset_info(node)?,
            session: &node.session,
        })
    }
}