use std::ffi::CString;

use log::debug;

use crate::ffi::raw as ffi;
use crate::{
    errors::{Result, HapiError, Kind},
    ffi::{AssetInfo, ParmInfo},
    node::HoudiniNode,
    session::Session,
};
use std::ops::Deref;
use crate::ffi::raw::ParmType;

struct AssetParmValues {
    int: Vec<i32>,
    float: Vec<f32>,
    string: Vec<String>,
}

pub struct AssetParameters {
    infos: Vec<ParmInfo>,
    values: AssetParmValues,
}

impl<'a> IntoIterator for &'a AssetParameters {
    type Item = AssetParm<'a>;
    type IntoIter = AssetParmIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AssetParmIter {
            iter: self.infos.iter(),
            values: &self.values,
        }
    }
}

pub struct AssetParmIter<'a> {
    iter: std::slice::Iter<'a, ParmInfo>,
    values: &'a AssetParmValues,
}

impl<'a> Iterator for AssetParmIter<'a> {
    type Item = AssetParm<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|info| AssetParm {
            info,
            values: &self.values,
        })
    }
}

pub struct AssetParm<'a> {
    info: &'a ParmInfo,
    values: &'a AssetParmValues,
}

impl<'a> std::ops::Deref for AssetParm<'a> {
    type Target = ParmInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

#[derive(Debug)]
pub enum ParmValue<'a> {
    Int(&'a [i32]),
    Float(&'a [f32]),
    String(&'a [String]),
    Other(String)
}

impl<'a> AssetParm<'a> {
    pub fn default_values(&self) -> ParmValue<'a> {
        let size = self.info.size() as usize;
       match self.info.parm_type() {
           ParmType::Int => {
               let start = self.info.int_values_index() as usize;
               ParmValue::Int(&self.values.int[start..start + size])
           }
           ParmType::Float => {
               let start = self.info.float_values_index() as usize;
               ParmValue::Float(&self.values.float[start..start + size])
}
           ParmType::String | ParmType::PathFileGeo => {
               let start = self.info.string_values_index() as usize;
               ParmValue::String(&self.values.string[start..start + size])
}
           _ => ParmValue::Other(format!("TODO: {:?}", self.info.parm_type()))
       }
    }
}

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
            .map(|a| a.into_iter().collect())
    }

    /// Try to create the first available asset in the library
    pub fn try_create_first(&self) -> Result<HoudiniNode> {
        use crate::errors::{HapiError, Kind};
        match self.get_asset_names()?.first() {
            Some(name) => self.session.create_node_blocking(name, None, None),
            None => Err(HapiError::new(
                Kind::Other("Empty AssetLibrary".to_string()),
                None,
                None,
            )),
        }
    }


    pub fn get_asset_parms(&self, asset: impl AsRef<str>) -> Result<AssetParameters> {
        let asset = CString::new(asset.as_ref())?;
        let count = crate::ffi::get_asset_def_parm_count(self.lib_id, &asset, &self.session)?;
        let infos = crate::ffi::get_asset_def_parm_info(self.lib_id, &asset, count.parm_count, &self.session)?
            .into_iter()
            .map(|info| ParmInfo {
                inner: info,
                session: self.session.clone(),
            });
        let values = crate::ffi::get_asset_def_parm_values(self.lib_id, &asset, &self.session, &count)?;
        let values = AssetParmValues {
            int: values.0,
            float: values.1,
            string: values.2
        };
        Ok(AssetParameters{ infos: infos.collect(), values })
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
