#![allow(unused)]
use super::array::{DataArray, StringMultiArray};
pub use super::bindings::AttribAccess;
use crate::errors::Result;
pub use crate::ffi::enums::StorageType;
pub use crate::ffi::AttributeInfo;
use crate::node::HoudiniNode;
use crate::stringhandle::StringArray;
use duplicate::duplicate;
use std::any::Any;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

pub(crate) struct _NumericAttrData<T: AttribAccess> {
    pub(crate) info: AttributeInfo,
    pub(crate) name: CString,
    pub(crate) node: HoudiniNode,
    pub(crate) _m: std::marker::PhantomData<T>,
}

pub(crate) struct _StringAttrData {
    pub(crate) info: AttributeInfo,
    pub(crate) name: CString,
    pub(crate) node: HoudiniNode,
}

pub struct NumericAttr<T: AttribAccess>(pub(crate) _NumericAttrData<T>);

pub struct NumericArrayAttr<T: AttribAccess>(pub(crate) _NumericAttrData<T>);

pub struct StringAttr(pub(crate) _StringAttrData);

pub struct StringArrayAttr(pub(crate) _StringAttrData);

impl<T: AttribAccess> NumericArrayAttr<T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub(crate) fn new(
        name: CString,
        info: AttributeInfo,
        node: HoudiniNode,
    ) -> NumericArrayAttr<T> {
        NumericArrayAttr(_NumericAttrData {
            info,
            name,
            node,
            _m: Default::default(),
        })
    }
    pub fn get(&self, part_id: i32) -> Result<DataArray<T>> {
        T::get_array(&self.0.name, &self.0.node, &self.0.info, part_id)
    }
    pub fn set(&self, part_id: i32, values: &DataArray<T>) -> Result<()> {
        T::set_array(
            &self.0.name,
            &self.0.node,
            &self.0.info,
            part_id,
            values.data(),
            values.sizes(),
        )
    }
}

impl<T: AttribAccess> NumericAttr<T> {
    pub(crate) fn new(name: CString, info: AttributeInfo, node: HoudiniNode) -> NumericAttr<T> {
        NumericAttr(_NumericAttrData {
            info,
            name,
            node,
            _m: Default::default(),
        })
    }
    pub fn get(&self, part_id: i32) -> Result<Vec<T>> {
        T::get(
            &self.0.name,
            &self.0.node,
            &self.0.info,
            part_id,
            -1,
            0,
            self.0.info.count(),
        )
    }
    pub fn set(&self, part_id: i32, values: &[T]) -> Result<()> {
        T::set(
            &self.0.name,
            &self.0.node,
            &self.0.info,
            part_id,
            values,
            0,
            self.0.info.count(),
        )
    }
}

impl StringAttr {
    pub fn new(name: CString, info: AttributeInfo, node: HoudiniNode) -> StringAttr {
        StringAttr(_StringAttrData { info, name, node })
    }
    fn get(&self, part_id: i32) -> Result<StringArray> {
        debug_assert!(self.0.node.is_valid()?);
        super::bindings::get_attribute_string_data(
            &self.0.node,
            part_id,
            self.0.name.as_c_str(),
            &self.0.info.inner,
        )
    }
    pub fn set(&self, part_id: i32, values: &[&str]) -> Result<()> {
        debug_assert!(self.0.node.is_valid()?);
        let cstr: std::result::Result<Vec<CString>, std::ffi::NulError> =
            values.iter().map(|s| CString::new(*s)).collect();
        let cstr = cstr?;
        let mut ptrs: Vec<&CStr> = cstr.iter().map(|cs| cs.as_c_str()).collect();
        super::bindings::set_attribute_string_data(
            &self.0.node,
            part_id,
            self.0.name.as_c_str(),
            &self.0.info.inner,
            ptrs.as_ref(),
        )
    }
}

impl StringArrayAttr {
    pub fn new(name: CString, info: AttributeInfo, node: HoudiniNode) -> StringArrayAttr {
        StringArrayAttr(_StringAttrData { info, name, node })
    }
    pub fn get(&self, part_id: i32) -> Result<StringMultiArray> {
        debug_assert!(self.0.node.is_valid()?);
        super::bindings::get_attribute_string_array_data(
            &self.0.node,
            self.0.name.as_c_str(),
            &self.0.info.inner,
        )
    }
    pub fn set(&self, part_id: i32, values: &[&str], sizes: &[i32]) -> Result<()> {
        debug_assert!(self.0.node.is_valid()?);
        let cstr: std::result::Result<Vec<CString>, std::ffi::NulError> =
            values.iter().map(|s| CString::new(*s)).collect();
        let cstr = cstr?;
        let mut ptrs: Vec<&CStr> = cstr.iter().map(|cs| cs.as_c_str()).collect();
        super::bindings::set_attribute_string_array_data(
            &self.0.node,
            self.0.name.as_c_str(),
            &self.0.info.inner,
            &ptrs,
            sizes,
        )
    }
}

pub trait AsAttribute {
    fn info(&self) -> &AttributeInfo;
    fn storage(&self) -> StorageType;
    fn boxed(self) -> Box<dyn AnyAttribWrapper>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
    fn name(&self) -> Cow<str>;
}

impl<T: AttribAccess> AsAttribute for NumericAttr<T> {
    fn info(&self) -> &AttributeInfo {
        &self.0.info
    }
    fn storage(&self) -> StorageType {
        T::storage()
    }

    fn name(&self) -> Cow<str> {
        self.0.name.to_string_lossy()
    }
}

impl<T: AttribAccess> AsAttribute for NumericArrayAttr<T> {
    fn info(&self) -> &AttributeInfo {
        &self.0.info
    }
    fn storage(&self) -> StorageType {
        T::storage()
    }
    fn name(&self) -> Cow<str> {
        self.0.name.to_string_lossy()
    }
}

impl AsAttribute for StringAttr {
    fn info(&self) -> &AttributeInfo {
        &self.0.info
    }

    fn storage(&self) -> StorageType {
        StorageType::String
    }

    fn name(&self) -> Cow<str> {
        self.0.name.to_string_lossy()
    }
}

impl AsAttribute for StringArrayAttr {
    fn info(&self) -> &AttributeInfo {
        &self.0.info
    }

    fn storage(&self) -> StorageType {
        StorageType::StringArray
    }

    fn name(&self) -> Cow<str> {
        self.0.name.to_string_lossy()
    }
}

// object safe trait
pub trait AnyAttribWrapper: Any + AsAttribute {
    fn as_any(&self) -> &dyn Any;
}

impl<T: AsAttribute + 'static> AnyAttribWrapper for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Attribute(Box<dyn AnyAttribWrapper>);

impl Attribute {
    pub(crate) fn new(attr_obj: Box<dyn AnyAttribWrapper>) -> Self {
        Attribute(attr_obj)
    }
    pub fn downcast<T: AnyAttribWrapper>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }
    pub fn name(&self) -> Cow<str> {
        self.0.name()
    }
    pub fn storage(&self) -> StorageType {
        self.0.storage()
    }
    pub fn info(&self) -> &AttributeInfo {
        self.0.info()
    }
}
