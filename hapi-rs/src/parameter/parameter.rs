use crate::{
    errors::Result,
    ffi,
    ffi::{
        ChoiceListType, HAPI_GetParmInfo, HAPI_NodeId, HAPI_ParmId, HAPI_ParmInfo,
        HAPI_ParmInfo_Create, NodeFlags, NodeType, ParmType, Permissions, PrmScriptType, RampType,
    },
    node::{HoudiniNode, NodeHandle, NodeInfo},
    session::Session,
};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::Formatter;
use std::mem::MaybeUninit;
use std::rc::Rc;

use log::warn;

#[derive(Debug, Clone)]
pub struct ParmHandle(pub HAPI_ParmId);

impl ParmHandle {
    pub fn from_name(name: &str, node: &HoudiniNode) -> Result<Self> {
        let id = unsafe {
            let name = CString::new(name)?;
            let mut id = MaybeUninit::uninit();
            ffi::HAPI_GetParmIdFromName(
                node.session.ptr(),
                node.handle.0,
                name.as_ptr(),
                id.as_mut_ptr(),
            )
            .result_with_session(|| node.session.clone())?;
            id.assume_init()
        };
        Ok(ParmHandle(id))
    }
    pub fn info<'s>(&self, node: &'s HoudiniNode) -> Result<ParmInfo<'s>> {
        let mut info = ParmInfo {
            inner: unsafe { HAPI_ParmInfo_Create() },
            session: &node.session,
        };
        unsafe {
            HAPI_GetParmInfo(
                node.session.ptr(),
                node.handle.0,
                info.inner.id,
                &mut info.inner as *mut _,
            )
            .result_with_session(|| node.session.clone())?
        }
        Ok(info)
    }
}

// TODO: Should be private
pub trait ParmBaseTrait<'s> {
    type ReturnType;

    fn base(&self) -> &ParameterBase<'s>;
    fn array_index(&self) -> i32;
    fn values_array(&self) -> Result<Vec<Self::ReturnType>>;
    fn single_value(&self) -> Result<Self::ReturnType>;
}

pub trait ParameterTrait<'s>: ParmBaseTrait<'s> {
    fn name(&self) -> Result<String>;
    fn get_value(&self) -> Result<ParmValue<<Self as ParmBaseTrait<'s>>::ReturnType>>;
}

impl<'s, T> ParameterTrait<'s> for T
where
    T: ParmBaseTrait<'s>,
{
    fn name(&self) -> Result<String> {
        self.base().name()
    }

    fn get_value(&self) -> Result<ParmValue<<T as ParmBaseTrait<'s>>::ReturnType>> {
        let index = self.array_index();
        let size = self.base().info.size();
        if size == 1 {
            return Ok(ParmValue::Single(self.single_value()?));
        } else {
            let mut values = self.values_array()?;
            debug_assert_eq!(values.len(), size as usize);
            Ok(match size {
                1 => ParmValue::Single(values.pop().unwrap()),
                2 => ParmValue::Tuple2((values.remove(0), values.remove(0))),
                3 => ParmValue::Tuple3((values.remove(0), values.remove(0), values.remove(0))),
                4 => ParmValue::Tuple4((
                    values.remove(0),
                    values.remove(0),
                    values.remove(0),
                    values.remove(0),
                )),
                _ => ParmValue::Array(values),
            })
        }
    }
}

pub struct ParameterBase<'session> {
    pub info: ParmInfo<'session>,
    pub node: &'session HoudiniNode,
    pub name: Option<CString>,
}
pub struct FloatParameter<'session> {
    base: ParameterBase<'session>,
}

pub struct IntParameter<'session> {
    base: ParameterBase<'session>,
}

pub struct StringParameter<'session> {
    base: ParameterBase<'session>,
}

#[derive(Debug)]
pub enum ParmValue<T> {
    Single(T),
    Tuple2((T, T)),
    Tuple3((T, T, T)),
    Tuple4((T, T, T, T)),
    Array(Vec<T>),
}

impl<T> From<T> for ParmValue<T> {
    fn from(v: T) -> Self {
        Self::Single(v)
    }
}

impl<'a, T> From<[&'a T; 2]> for ParmValue<&'a T> {
    fn from(v: [&'a T; 2]) -> Self {
        Self::Tuple2((v[0], v[1]))
    }
}

pub enum Parameter<'session> {
    Float(FloatParameter<'session>),
    Int(IntParameter<'session>),
    String(StringParameter<'session>),
    Other,
}

impl<'node> Parameter<'node> {
    pub(crate) fn new(
        node: &'node HoudiniNode,
        info: HAPI_ParmInfo,
        name: Option<CString>,
    ) -> Parameter<'node> {
        let base = ParameterBase {
            info: ParmInfo {
                inner: info,
                session: &node.session,
            },
            name,
            node,
        };
        match base.info.parm_type() {
            ParmType::Int => Parameter::Int(IntParameter { base }),
            ParmType::Float => Parameter::Float(FloatParameter { base }),
            ParmType::String => Parameter::String(StringParameter { base }),
            _ => Parameter::Other,
        }
    }
}

impl<'s> ParmBaseTrait<'s> for FloatParameter<'s> {
    type ReturnType = f32;

    fn base(&self) -> &ParameterBase<'s> {
        &self.base
    }

    fn array_index(&self) -> i32 {
        self.base.info.float_values_index()
    }

    fn values_array(&self) -> Result<Vec<Self::ReturnType>> {
        let index = self.base.info.float_values_index();
        let count = self.base.info.size();
        let mut values = vec![0.; count as usize];
        unsafe {
            ffi::HAPI_GetParmFloatValues(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                values.as_mut_ptr(),
                index,
                count,
            )
            .result_with_session(|| self.base.node.session.clone())?;
        }
        Ok(values)
    }

    fn single_value(&self) -> Result<Self::ReturnType> {
        let name = self.base.c_name()?;
        let mut value = MaybeUninit::uninit();
        unsafe {
            ffi::HAPI_GetParmFloatValue(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                name.as_ptr(),
                0,
                value.as_mut_ptr(),
            )
            .result_with_session(|| self.base.node.session.clone());
            Ok(value.assume_init())
        }
    }
}

impl<'s> FloatParameter<'s> {
    pub fn set_value<T>(&self, val: T) -> Result<()>
    where
        T: Into<ParmValue<f32>>,
    {
        fn set(self_: &FloatParameter, val_: &[f32]) -> Result<()> {
            if val_.len() > self_.base.info.size() as usize {
                warn!("Trying to set parm \"{}\" to incorrect value size", self_.base.name()?);
            }
            unsafe {
                ffi::HAPI_SetParmFloatValues(
                    self_.base.node.session.ptr(),
                    self_.base.node.handle.0,
                    val_.as_ptr(),
                    self_.base.info.float_values_index(),
                    self_.base.info.size(),
                )
            };
            Ok(())
        }
        match val.into() {
            ParmValue::Single(v) => set(self, &[v])?,
            ParmValue::Tuple2((v1, v2)) => set(self, &[v1, v2])?,
            ParmValue::Tuple3((v1, v2, v3)) => set(self, &[v1, v2, v3])?,
            ParmValue::Tuple4((v1, v2, v3, v4)) => set(self, &[v1, v2, v3, v4])?,
            ParmValue::Array(v) => set(self, &v)?,
        }
        Ok(())
    }
}

impl<'s> ParmBaseTrait<'s> for IntParameter<'s> {
    type ReturnType = i32;

    fn base(&self) -> &ParameterBase<'s> {
        &self.base
    }

    fn array_index(&self) -> i32 {
        self.base.info.int_values_index()
    }

    fn values_array(&self) -> Result<Vec<Self::ReturnType>> {
        let index = self.base.info.int_values_index();
        let count = self.base.info.size();
        let mut values = vec![0; count as usize];
        unsafe {
            ffi::HAPI_GetParmIntValues(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                values.as_mut_ptr(),
                index,
                count,
            )
            .result_with_session(|| self.base.node.session.clone())?;
        }
        Ok(values)
    }

    fn single_value(&self) -> Result<Self::ReturnType> {
        let name = self.base.c_name()?;
        let mut value = MaybeUninit::uninit();
        unsafe {
            ffi::HAPI_GetParmIntValue(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                name.as_ptr(),
                0,
                value.as_mut_ptr(),
            )
            .result_with_session(|| self.base.node.session.clone());
            Ok(value.assume_init())
        }
    }
}

impl<'s> ParmBaseTrait<'s> for StringParameter<'s> {
    type ReturnType = String;

    fn base(&self) -> &ParameterBase<'s> {
        &self.base
    }

    fn array_index(&self) -> i32 {
        self.base.info.int_values_index()
    }

    fn values_array(&self) -> Result<Vec<Self::ReturnType>> {
        let index = self.base.info.string_values_index();
        let count = self.base.info.size();
        let mut handles = vec![];
        unsafe {
            ffi::HAPI_GetParmStringValues(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                1,
                handles.as_mut_ptr(),
                index,
                count,
            )
            .result_with_session(|| self.base.node.session.clone())?;
        }
        crate::stringhandle::get_string_batch(&handles, &self.base.node.session)
    }

    fn single_value(&self) -> Result<Self::ReturnType> {
        let name = self.base.c_name()?;
        let mut handle = MaybeUninit::uninit();
        unsafe {
            ffi::HAPI_GetParmStringValue(
                self.base.node.session.ptr(),
                self.base.node.handle.0,
                name.as_ptr(),
                0,
                1,
                handle.as_mut_ptr(),
            )
            .result_with_session(|| self.base.node.session.clone());
            self.base.node.session.get_string(handle.assume_init())
        }
    }
}

impl<'s> StringParameter<'s> {
    pub fn set_value<T, R>(&self, val: T) -> Result<()>
    where
        R: AsRef<str>,
        T: Into<ParmValue<R>>,
    {
        fn set_comp(self_: &StringParameter, val_: &str, cmp: i32) -> Result<()> {
            let val = CString::new(val_)?;
            unsafe {
                ffi::HAPI_SetParmStringValue(
                    self_.base.node.session.ptr(),
                    self_.base.node.handle.0,
                    val.as_ptr(),
                    self_.base.info.id().0,
                    0,
                )
            };
            Ok(())
        }
        match val.into() {
            ParmValue::Single(v) => set_comp(self, v.as_ref(), 0)?,
            ParmValue::Tuple2((v1, v2)) => {
                set_comp(self, v1.as_ref(), 0)?;
                set_comp(self, v2.as_ref(), 1)?;
            }
            ParmValue::Tuple3((v1, v2, v3)) => {
                set_comp(self, v1.as_ref(), 0)?;
                set_comp(self, v2.as_ref(), 1)?;
                set_comp(self, v3.as_ref(), 2)?;
            }
            ParmValue::Tuple4((v1, v2, v3, v4)) => {
                set_comp(self, v1.as_ref(), 0)?;
                set_comp(self, v2.as_ref(), 1)?;
                set_comp(self, v3.as_ref(), 2)?;
                set_comp(self, v4.as_ref(), 3)?;
            }
            ParmValue::Array(v) => {
                for (i, v) in v.iter().enumerate() {
                    set_comp(self, v.as_ref(), i as i32)?;
                }
            }
        }
        Ok(())
    }
}

impl<'node> ParameterBase<'node> {
    pub fn name(&self) -> Result<String> {
        match self.name.as_ref() {
            None => self.info.name(),
            Some(n) => Ok(n.to_string_lossy().to_string()),
        }
    }

    pub(crate) fn c_name(&self) -> Result<Cow<CString>> {
        match self.name.as_ref() {
            None => Ok(Cow::Owned(self.info.name_cstr()?)),
            Some(n) => Ok(Cow::Borrowed(n)),
        }
    }
}

impl std::fmt::Debug for ParameterBase<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parameter[{name} of type {type}]",
               name=self.info.name().unwrap(),
                type=self.info.parm_type().as_ref())
    }
}

pub struct ParmInfo<'session> {
    pub(crate) inner: HAPI_ParmInfo,
    pub(crate) session: &'session Session,
}

macro_rules! _get_str {
    ($m:ident->$f:ident) => {
        pub fn $m(&self) -> Result<String> {
            self.session.get_string(self.inner.$f)
        }
    };
}

impl<'session> ParmInfo<'session> {
    pub(crate) fn from_name(name: &CStr, node: &'session HoudiniNode) -> Result<Self> {
        let info = unsafe {
            let mut info = MaybeUninit::uninit();
            ffi::HAPI_GetParmInfoFromName(
                node.session.ptr(),
                node.handle.0,
                name.as_ptr(),
                info.as_mut_ptr(),
            )
            .result_with_session(|| node.session.clone())?;
            info.assume_init()
        };

        Ok(ParmInfo {
            inner: info,
            session: &node.session,
        })
    }

    pub(crate) fn name_cstr(&self) -> Result<CString> {
        crate::stringhandle::get_cstring(self.inner.nameSH, &self.session)
    }

    get!(id->id->[handle: ParmHandle]);
    get!(parent_id->parentId->[handle: ParmHandle]);
    get!(child_index->childIndex->i32);
    get!(parm_type->type_->ParmType);
    get!(script_type->scriptType->PrmScriptType);
    get!(permissions->permissions->Permissions);
    get!(tag_count->tagCount->i32);
    get!(size->size->i32);
    get!(choice_count->choiceCount->i32);
    get!(choice_list_type->choiceListType->ChoiceListType);
    get!(has_min->hasMin->bool);
    get!(has_max->hasMax->bool);
    get!(has_uimin->hasUIMin->bool);
    get!(has_uimax->hasUIMax->bool);
    get!(min->min->f32);
    get!(max->max->f32);
    get!(uimin->UIMin->f32);
    get!(uimax->UIMax->f32);
    get!(invisible->invisible->bool);
    get!(disabled->disabled->bool);
    get!(spare->spare->bool);
    get!(join_next->joinNext->bool);
    get!(label_none->labelNone->bool);
    get!(int_values_index->intValuesIndex->i32);
    get!(float_values_index->floatValuesIndex->i32);
    get!(string_values_index->stringValuesIndex->i32);
    get!(choice_index->choiceIndex->i32);
    get!(input_node_type->inputNodeType->NodeType);
    get!(input_node_flag->inputNodeFlag->NodeFlags);
    get!(is_child_of_multi_parm->isChildOfMultiParm->bool);
    get!(instance_num->instanceNum->i32);
    get!(instance_length->instanceLength->i32);
    get!(instance_count->instanceCount->i32);
    get!(instance_start_offset->instanceStartOffset->i32);
    get!(ramp_type->rampType->RampType);
    _get_str!(type_info->typeInfoSH);
    _get_str!(name->nameSH);
    _get_str!(label->labelSH);
    _get_str!(template_name->templateNameSH);
    _get_str!(help->helpSH);
    _get_str!(visibility_condition->visibilityConditionSH);
    _get_str!(disabled_condition->disabledConditionSH);
}
