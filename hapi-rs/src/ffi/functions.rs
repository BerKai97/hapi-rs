#![allow(clippy::missing_safety_doc)]

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ptr::null;

use crate::{
    errors::Result,
    node::{HoudiniNode, NodeHandle},
    parameter::ParmHandle,
    session::{CookOptions, Session, SessionOptions},
    stringhandle::StringsArray,
};

use super::raw;
use crate::ffi::PartInfo;

macro_rules! uninit {
    () => {
        MaybeUninit::uninit()
    };
}

pub fn get_parm_float_values(node: NodeHandle, session: &Session, start: i32, count: i32) -> Result<Vec<f32>> {
    let mut values = vec![0.; count as usize];
    unsafe {
        raw::HAPI_GetParmFloatValues(
            session.ptr(),
            node.0,
            values.as_mut_ptr(),
            start,
            count,
        )
        .result_with_session(|| session.clone())?
    }
    Ok(values)
}

pub fn get_parm_int_values(node: NodeHandle, session: &Session, start: i32, length: i32) -> Result<Vec<i32>> {
    let mut values = vec![0; length as usize];
    unsafe {
        raw::HAPI_GetParmIntValues(
            session.ptr(),
            node.0,
            values.as_mut_ptr(),
            start,
            length,
        )
        .result_with_session(|| session.clone())?
    }
    Ok(values)
}

pub fn get_parm_string_values(node: NodeHandle, session: &Session, start: i32, length: i32) -> Result<StringsArray> {
    let mut handles = vec![0; length as usize];
    unsafe {
        raw::HAPI_GetParmStringValues(
            session.ptr(),
            node.0,
            1,
            handles.as_mut_ptr(),
            start,
            length,
        )
        .result_with_session(|| session.clone())?
    }
    crate::stringhandle::get_strings_array(&handles, session)
}

pub fn get_parm_float_value(node: &HoudiniNode, name: &CStr, index: i32) -> Result<f32> {
    let mut value = uninit!();

    unsafe {
        raw::HAPI_GetParmFloatValue(
            node.session.ptr(),
            node.handle.0,
            name.as_ptr(),
            index,
            value.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        Ok(value.assume_init())
    }
}

pub fn get_parm_int_value(node: &HoudiniNode, name: &CStr, index: i32) -> Result<i32> {
    let mut value = uninit!();

    unsafe {
        raw::HAPI_GetParmIntValue(
            node.session.ptr(),
            node.handle.0,
            name.as_ptr(),
            index,
            value.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        Ok(value.assume_init())
    }
}

pub fn get_parm_string_value(node: &HoudiniNode, name: &CStr, index: i32) -> Result<String> {
    let mut handle = uninit!();
    let handle = unsafe {
        raw::HAPI_GetParmStringValue(
            node.session.ptr(),
            node.handle.0,
            name.as_ptr(),
            index,
            1,
            handle.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        handle.assume_init()
    };
    node.session.get_string(handle)
}

pub fn get_parm_node_value(node: &HoudiniNode, name: &CStr) -> Result<Option<NodeHandle>> {
    unsafe {
        let mut id = uninit!();
        raw::HAPI_GetParmNodeValue(
            node.session.ptr(),
            node.handle.0,
            name.as_ptr(),
            id.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        let id = id.assume_init();
        Ok(if id == -1 {
            None
        } else {
            Some(NodeHandle(id, ()))
        })
    }
}

pub fn set_parm_float_value(node: &HoudiniNode, name: &CStr, index: i32, value: f32) -> Result<()> {
    unsafe {
        raw::HAPI_SetParmFloatValue(
            node.session.ptr(),
            node.handle.0,
            name.as_ptr(),
            index,
            value,
        )
        .result_with_session(|| node.session.clone())
    }
}

pub fn set_parm_float_values(
    node: NodeHandle,
    session: &Session,
    start: i32,
    length: i32,
    values: &[f32],
) -> Result<()> {
    if values.len() as i32 > length {
        log::warn!("Array length is greater than parm length: {:?}", values);
    }
    let length = values.len().min(length as usize);
    unsafe {
        raw::HAPI_SetParmFloatValues(
            session.ptr(),
            node.0,
            values.as_ptr(),
            start,
            length as i32,
        )
        .result_with_session(|| session.clone())
    }
}

pub fn set_parm_int_values(
    node: NodeHandle,
    session: &Session,
    start: i32,
    length: i32,
    values: &[i32],
) -> Result<()> {
    unsafe {
        raw::HAPI_SetParmIntValues(
            session.ptr(),
            node.0,
            values.as_ptr(),
            start,
            length,
        )
        .result_with_session(|| session.clone())
    }
}

pub fn set_parm_int_value(node: NodeHandle, session: &Session, name: &CStr, index: i32, value: i32) -> Result<()> {
    unsafe {
        raw::HAPI_SetParmIntValue(
            session.ptr(),
            node.0,
            name.as_ptr(),
            index,
            value,
        )
        .result_with_session(|| session.clone())
    }
}

pub fn set_parm_string_value(
    node: NodeHandle,
    session: &Session,
    parm: &ParmHandle,
    index: i32,
    value: &CStr,
) -> Result<()> {
    unsafe {
        raw::HAPI_SetParmStringValue(
            session.ptr(),
            node.0,
            value.as_ptr(),
            parm.0,
            index,
        )
        .result_with_session(|| session.clone())
    }
}

pub fn set_parm_string_values<T>(node: NodeHandle, session: &Session, parm: &ParmHandle, values: &[T]) -> Result<()>
where
    T: AsRef<CStr>,
{
    for (i, v) in values.iter().enumerate() {
        set_parm_string_value(node, session, parm, i as i32, v.as_ref())?;
    }
    Ok(())
}

pub fn get_parm_choice_list(
    node: NodeHandle,
    session: &Session,
    index: i32,
    length: i32,
) -> Result<Vec<raw::HAPI_ParmChoiceInfo>> {
    unsafe {
        let mut structs = vec![raw::HAPI_ParmChoiceInfo_Create(); length as usize];
        raw::HAPI_GetParmChoiceLists(
            session.ptr(),
            node.0,
            structs.as_mut_ptr(),
            index,
            length,
        )
        .result_with_session(|| session.clone())?;
        Ok(structs)
    }
}

pub fn get_parm_expression(node: NodeHandle, session: &Session, parm: &CStr, index: i32) -> Result<String> {
    let handle = unsafe {
        let mut handle = uninit!();
        raw::HAPI_GetParmExpression(
            session.ptr(),
            node.0,
            parm.as_ptr(),
            index,
            handle.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        handle.assume_init()
    };
    crate::stringhandle::get_string(handle, &session)
}

pub fn parm_has_expression(node: NodeHandle, session: &Session, parm: &CStr, index: i32) -> Result<bool> {
    let ret = unsafe {
        let mut ret = uninit!();
        raw::HAPI_ParmHasExpression(
            session.ptr(),
            node.0,
            parm.as_ptr(),
            index,
            ret.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        ret.assume_init()
    };
    Ok(ret > 0)
}

pub fn set_parm_expression(
    node: NodeHandle,
    session: &Session,
    parm: ParmHandle,
    value: &CStr,
    index: i32,
) -> Result<()> {
    unsafe {
        raw::HAPI_SetParmExpression(
            session.ptr(),
            node.0,
            value.as_ptr(),
            parm.0,
            index,
        )
        .result_with_session(|| session.clone())
    }
}

pub fn get_parm_info(node: NodeHandle, session: &Session, parm: ParmHandle) -> Result<raw::HAPI_ParmInfo> {
    unsafe {
        let mut info = uninit!();
        super::raw::HAPI_GetParmInfo(session.ptr(), node.0, parm.0, info.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn get_parm_info_from_name(node: NodeHandle, session: &Session, name: &CStr) -> Result<raw::HAPI_ParmInfo> {
    unsafe {
        let mut info = uninit!();
        super::raw::HAPI_GetParmInfoFromName(
            session.ptr(),
            node.0,
            name.as_ptr(),
            info.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn get_parm_id_from_name(name: &CStr, node: NodeHandle, session: &Session) -> Result<i32> {
    unsafe {
        let mut id = uninit!();
        crate::ffi::raw::HAPI_GetParmIdFromName(
            session.ptr(),
            node.0,
            name.as_ptr(),
            id.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        Ok(id.assume_init())
    }
}

pub fn get_node_info(node: NodeHandle, session: &Session) -> Result<raw::HAPI_NodeInfo> {
    unsafe {
        let mut info = uninit!();
        super::raw::HAPI_GetNodeInfo(session.ptr(), node.0, info.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn is_node_valid(info: &super::NodeInfo) -> Result<bool> {
    unsafe {
        let mut answer = uninit!();
        raw::HAPI_IsNodeValid(
            info.session.ptr(),
            info.inner.id,
            info.inner.uniqueHoudiniNodeId,
            answer.as_mut_ptr(),
        )
        .result_with_session(|| info.session.clone())?;
        Ok(answer.assume_init() == 1)
    }
}

pub fn delete_node(node: HoudiniNode) -> Result<()> {
    unsafe {
        raw::HAPI_DeleteNode(node.session.ptr(), node.handle.0)
            .result_with_session(|| node.session.clone())
    }
}

pub fn get_node_path(node: &HoudiniNode, relative_to: Option<&HoudiniNode>) -> Result<String> {
    unsafe {
        let mut sh = uninit!();
        raw::HAPI_GetNodePath(
            node.session.ptr(),
            node.handle.0,
            relative_to.map(|n| n.handle.0).unwrap_or(-1),
            sh.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        crate::stringhandle::get_string(sh.assume_init(), &node.session)
    }
}

pub fn cook_node(node: &HoudiniNode, options: &CookOptions) -> Result<()> {
    unsafe {
        raw::HAPI_CookNode(node.session.ptr(), node.handle.0, options.ptr())
            .result_with_session(|| node.session.clone())
    }
}

pub fn load_library_from_file(path: &CStr, session: &Session, _override: bool) -> Result<i32> {
    unsafe {
        let mut lib_id = uninit!();
        raw::HAPI_LoadAssetLibraryFromFile(
            session.ptr(),
            path.as_ptr(),
            _override as i8,
            lib_id.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        Ok(lib_id.assume_init())
    }
}

pub fn get_asset_info(node: &HoudiniNode) -> Result<raw::HAPI_AssetInfo> {
    unsafe {
        let mut info = uninit!();
        raw::HAPI_GetAssetInfo(node.session.ptr(), node.handle.0, info.as_mut_ptr())
            .result_with_session(|| node.session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn get_asset_count(library_id: i32, session: &Session) -> Result<i32> {
    unsafe {
        let mut num_assets = uninit!();
        raw::HAPI_GetAvailableAssetCount(session.ptr(), library_id, num_assets.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(num_assets.assume_init())
    }
}

pub fn get_asset_names(
    library_id: i32,
    num_assets: i32,
    session: &Session,
) -> Result<StringsArray> {
    let handles = unsafe {
        let mut names = vec![0; num_assets as usize];
        raw::HAPI_GetAvailableAssets(session.ptr(), library_id, names.as_mut_ptr(), num_assets)
            .result_with_session(|| session.clone())?;
        names
    };
    crate::stringhandle::get_strings_array(&handles, session)
}

#[derive(Default, Debug)]
pub struct ParmValueCount {
    pub parm_count: i32,
    pub int_count: i32,
    pub float_count: i32,
    pub string_count: i32,
    pub choice_count: i32,
}

pub fn get_asset_def_parm_count(
    library_id: i32,
    asset: &CStr,
    session: &Session,
) -> Result<ParmValueCount> {
    let mut parms = ParmValueCount::default();
    unsafe {
        raw::HAPI_GetAssetDefinitionParmCounts(
            session.ptr(),
            library_id,
            asset.as_ptr(),
            &mut parms.parm_count as *mut _,
            &mut parms.int_count as *mut _,
            &mut parms.float_count as *mut _,
            &mut parms.string_count as *mut _,
            &mut parms.choice_count as *mut _,
        )
        .result_with_session(|| session.clone())?;
    }
    Ok(parms)
}

pub fn get_asset_def_parm_info(
    library_id: i32,
    asset: &CStr,
    count: i32,
    session: &Session,
) -> Result<Vec<raw::HAPI_ParmInfo>> {
    unsafe {
        let mut parms = vec![raw::HAPI_ParmInfo_Create(); count as usize];
        raw::HAPI_GetAssetDefinitionParmInfos(
            session.ptr(),
            library_id,
            asset.as_ptr(),
            parms.as_mut_ptr(),
            0,
            count,
        )
        .result_with_session(|| session.clone())?;
        Ok(parms)
    }
}

pub fn get_string_batch_size(handles: &[i32], session: &Session) -> Result<i32> {
    unsafe {
        let mut length = uninit!();
        raw::HAPI_GetStringBatchSize(
            session.ptr(),
            handles.as_ptr(),
            handles.len() as i32,
            length.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        Ok(length.assume_init())
    }
}

/// Note: contiguous array of null-terminated strings
pub fn get_string_batch(length: i32, session: &Session) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; length as usize];
    unsafe {
        raw::HAPI_GetStringBatch(session.ptr(), buffer.as_mut_ptr() as *mut _, length as i32)
            .result_with_session(|| session.clone())?;
    }
    buffer.truncate(length as usize);
    Ok(buffer)
}

pub fn get_string_buff_len(session: &Session, handle: i32) -> Result<i32> {
    unsafe {
        let mut length = uninit!();
        raw::HAPI_GetStringBufLength(session.ptr(), handle, length.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(length.assume_init())
    }
}

pub fn get_string(session: &Session, handle: i32, length: i32) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; length as usize];
    unsafe {
        raw::HAPI_GetString(session.ptr(), handle, buffer.as_mut_ptr() as *mut _, length)
            .result_with_message("get_string failed")?;
        buffer.truncate(length as usize - 1);
    }
    Ok(buffer)
}

pub fn get_status_string(
    session: &Session,
    status: raw::StatusType,
    verbosity: raw::StatusVerbosity,
) -> Result<String> {
    let mut length = uninit!();
    unsafe {
        raw::HAPI_GetStatusStringBufLength(session.ptr(), status, verbosity, length.as_mut_ptr())
            .result_with_message("GetStatusStringBufLength failed")?;
        let length = length.assume_init();
        let mut buf = vec![0u8; length as usize];
        if length > 0 {
            raw::HAPI_GetStatusString(session.ptr(), status, buf.as_mut_ptr() as *mut i8, length)
                .result_with_message("GetStatusString failed")?;
            buf.truncate(length as usize - 1);
            Ok(String::from_utf8_unchecked(buf))
        } else {
            Ok(String::new())
        }
    }
}

pub fn create_inprocess_session() -> Result<raw::HAPI_Session> {
    let mut ses = uninit!();
    unsafe {
        raw::HAPI_CreateInProcessSession(ses.as_mut_ptr())
            .result_with_message("Session::new_in_process failed")?;
        Ok(ses.assume_init())
    }
}

pub fn set_server_env_str(session: &Session, key: &CStr, value: &CStr) -> Result<()> {
    unsafe {
        raw::HAPI_SetServerEnvString(session.ptr(), key.as_ptr(), value.as_ptr())
            .result_with_session(|| session.clone())
    }
}

pub fn set_server_env_int(session: &Session, key: &CStr, value: i32) -> Result<()> {
    unsafe {
        raw::HAPI_SetServerEnvInt(session.ptr(), key.as_ptr(), value)
            .result_with_session(|| session.clone())
    }
}

pub fn get_server_env_var_count(session: &Session) -> Result<i32> {
    unsafe {
        let mut val = uninit!();
        raw::HAPI_GetServerEnvVarCount(session.ptr(), val.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(val.assume_init())
    }
}

pub fn get_server_env_var_list(session: &Session, count: i32) -> Result<Vec<i32>> {
    unsafe {
        let mut handles = vec![0; count as usize];
        raw::HAPI_GetServerEnvVarList(session.ptr(), handles.as_mut_ptr(), 0, count)
            .result_with_session(|| session.clone())?;
        Ok(handles)
    }
}

pub fn get_server_env_str(session: &Session, key: &CStr) -> Result<String> {
    unsafe {
        let mut val = uninit!();
        raw::HAPI_GetServerEnvString(session.ptr(), key.as_ptr(), val.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        session.get_string(val.assume_init())
    }
}

pub fn get_server_env_int(session: &Session, key: &CStr) -> Result<i32> {
    unsafe {
        let mut val = uninit!();
        raw::HAPI_GetServerEnvInt(session.ptr(), key.as_ptr(), val.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(val.assume_init())
    }
}

pub fn start_thrift_pipe_server(
    file: &CStr,
    options: &raw::HAPI_ThriftServerOptions,
) -> Result<i32> {
    let mut pid = uninit!();
    unsafe {
        raw::HAPI_StartThriftNamedPipeServer(options as *const _, file.as_ptr(), pid.as_mut_ptr())
            .result_with_message("Could not start thrift server")?;
        Ok(pid.assume_init())
    }
}

pub fn start_thrift_socket_server(
    port: i32,
    options: &raw::HAPI_ThriftServerOptions,
) -> Result<i32> {
    let mut pid = uninit!();
    unsafe {
        raw::HAPI_StartThriftSocketServer(options as *const _, port, pid.as_mut_ptr())
            .result_with_message("Could not start thrift server")?;
        Ok(pid.assume_init())
    }
}

pub fn new_thrift_piped_session(path: &CStr) -> Result<raw::HAPI_Session> {
    let mut handle = uninit!();
    let session = unsafe {
        raw::HAPI_CreateThriftNamedPipeSession(handle.as_mut_ptr(), path.as_ptr())
            .result_with_message("Could not start piped session")?;
        handle.assume_init()
    };
    Ok(session)
}

pub fn new_thrift_socket_session(port: i32, host: &CStr) -> Result<raw::HAPI_Session> {
    let mut handle = uninit!();
    let session = unsafe {
        raw::HAPI_CreateThriftSocketSession(handle.as_mut_ptr(), host.as_ptr(), port)
            .result_with_message("Could not start socket session")?;
        handle.assume_init()
    };
    Ok(session)
}

pub fn initialize_session(session: &raw::HAPI_Session, options: &SessionOptions) -> Result<()> {
    unsafe {
        raw::HAPI_Initialize(
            session as *const _,
            options.cook_opt.ptr(),
            options.unsync as i8,
            -1,
            options
                .env_files
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(null()),
            options
                .otl_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(null()),
            options
                .dso_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(null()),
            options
                .img_dso_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(null()),
            options
                .aud_dso_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(null()),
        )
        .result_with_message("Could not initialize session")
    }
}

pub fn cleanup_session(session: &Session) -> Result<()> {
    unsafe { raw::HAPI_Cleanup(session.ptr()).result_with_session(|| session.clone()) }
}

pub fn close_session(session: &Session) -> Result<()> {
    unsafe { raw::HAPI_CloseSession(session.ptr()).result_with_session(|| session.clone()) }
}

pub fn is_session_initialized(session: &Session) -> Result<bool> {
    unsafe {
        match raw::HAPI_IsInitialized(session.ptr()) {
            raw::HapiResult::Success => Ok(true),
            raw::HapiResult::NotInitialized => Ok(false),
            e => Err(e.into()),
        }
    }
}

pub fn save_hip(session: &Session, name: &CStr) -> Result<()> {
    unsafe {
        raw::HAPI_SaveHIPFile(session.ptr(), name.as_ptr(), 0)
            .result_with_session(|| session.clone())
    }
}

pub fn load_hip(session: &Session, name: &CStr, cook: bool) -> Result<()> {
    unsafe {
        raw::HAPI_LoadHIPFile(session.ptr(), name.as_ptr(), cook as i8)
            .result_with_session(|| session.clone())
    }
}

pub fn merge_hip(session: &Session, name: &CStr, cook: bool) -> Result<i32> {
    unsafe {
        let mut id = uninit!();
        raw::HAPI_MergeHIPFile(session.ptr(), name.as_ptr(), cook as i8, id.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(id.assume_init())
    }
}

pub fn interrupt(session: &Session) -> Result<()> {
    unsafe { raw::HAPI_Interrupt(session.ptr()).result_with_session(|| session.clone()) }
}

pub fn get_status(session: &Session, flag: raw::StatusType) -> Result<raw::State> {
    let status = unsafe {
        let mut status = uninit!();
        raw::HAPI_GetStatus(session.ptr(), flag, status.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        status.assume_init()
    };
    Ok(raw::State::from(status))
}

pub fn is_session_valid(session: &Session) -> bool {
    unsafe {
        matches!(
            raw::HAPI_IsSessionValid(session.ptr()),
            raw::HapiResult::Success
        )
    }
}

pub fn get_cooking_total_count(session: &Session) -> Result<i32> {
    unsafe {
        let mut count = uninit!();
        raw::HAPI_GetCookingTotalCount(session.ptr(), count.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(count.assume_init())
    }
}

pub fn get_cooking_current_count(session: &Session) -> Result<i32> {
    unsafe {
        let mut count = uninit!();
        raw::HAPI_GetCookingCurrentCount(session.ptr(), count.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(count.assume_init())
    }
}

pub fn get_connection_error(clear: bool) -> Result<String> {
    unsafe {
        let mut length = uninit!();
        raw::HAPI_GetConnectionErrorLength(length.as_mut_ptr())
            .result_with_message("HAPI_GetConnectionErrorLength failed")?;
        let length = length.assume_init();
        if length > 0 {
            let mut buf = vec![0u8; length as usize];
            raw::HAPI_GetConnectionError(buf.as_mut_ptr() as *mut _, length, clear as i8)
                .result_with_message("HAPI_GetConnectionError failed")?;
            Ok(String::from_utf8_unchecked(buf))
        } else {
            Ok(String::new())
        }
    }
}

pub fn get_total_cook_count(
    node: &HoudiniNode,
    node_types: raw::NodeType,
    node_flags: raw::NodeFlags,
    recursive: bool,
) -> Result<i32> {
    let mut count = uninit!();
    unsafe {
        raw::HAPI_GetTotalCookCount(
            node.session.ptr(),
            node.handle.0,
            node_types.0,
            node_flags.0,
            recursive as i8,
            count.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        Ok(count.assume_init())
    }
}

pub fn create_node(
    name: &CStr,
    label: Option<&CStr>,
    session: &Session,
    parent: Option<NodeHandle>,
    cook: bool,
) -> Result<raw::HAPI_NodeId> {
    unsafe {
        let mut id = uninit!();
        raw::HAPI_CreateNode(
            session.ptr(),
            parent.map_or(-1, |h| h.0),
            name.as_ptr(),
            label.map_or(null(), |v| v.as_ptr()),
            cook as i8,
            id.as_mut_ptr(),
        )
        .result_with_session(|| session.clone())?;
        Ok(id.assume_init())
    }
}

pub fn create_input_node(session: &Session, name: &CStr) -> Result<raw::HAPI_NodeId> {
    let mut id = uninit!();
    unsafe {
        raw::HAPI_CreateInputNode(session.ptr(), id.as_mut_ptr(), name.as_ptr())
            .result_with_session(|| session.clone())?;
        Ok(id.assume_init())
    }
}

pub fn get_manager_node(session: &Session, node_type: raw::NodeType) -> Result<raw::HAPI_NodeId> {
    unsafe {
        let mut id = uninit!();
        raw::HAPI_GetManagerNodeId(session.ptr(), node_type, id.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(id.assume_init())
    }
}

pub fn get_compose_child_node_list(
    node: &HoudiniNode,
    types: raw::NodeType,
    flags: raw::NodeFlags,
    recursive: bool,
) -> Result<Vec<i32>> {
    unsafe {
        let mut count = uninit!();
        raw::HAPI_ComposeChildNodeList(
            node.session.ptr(),
            node.handle.0,
            types.0,
            flags.0,
            recursive as i8,
            count.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;

        let count = count.assume_init();
        let mut obj_infos = vec![0i32; count as usize];
        raw::HAPI_GetComposedChildNodeList(
            node.session.ptr(),
            node.handle.0,
            obj_infos.as_mut_ptr(),
            count,
        )
        .result_with_session(|| node.session.clone())?;
        Ok(obj_infos)
    }
}

pub fn get_composed_object_list(
    session: &Session,
    parent_id: raw::HAPI_NodeId,
) -> Result<Vec<raw::HAPI_ObjectInfo>> {
    unsafe {
        let mut count = uninit!();
        raw::HAPI_ComposeObjectList(session.ptr(), parent_id, null(), count.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        let count = count.assume_init();
        let mut obj_infos = vec![raw::HAPI_ObjectInfo_Create(); count as usize];
        raw::HAPI_GetComposedObjectList(session.ptr(), parent_id, obj_infos.as_mut_ptr(), 0, count)
            .result_with_session(|| session.clone())?;
        Ok(obj_infos)
    }
}

pub fn get_parameters(node: &HoudiniNode) -> Result<Vec<raw::HAPI_ParmInfo>> {
    unsafe {
        let mut parms = vec![raw::HAPI_ParmInfo_Create(); node.info.parm_count() as usize];
        raw::HAPI_GetParameters(
            node.session.ptr(),
            node.handle.0,
            parms.as_mut_ptr(),
            0,
            node.info.parm_count(),
        )
        .result_with_session(|| node.session.clone())?;
        Ok(parms)
    }
}

pub fn query_node_input(node: &HoudiniNode, idx: i32) -> Result<i32> {
    let mut inp_idx = uninit!();
    unsafe {
        raw::HAPI_QueryNodeInput(node.session.ptr(), node.handle.0, idx, inp_idx.as_mut_ptr())
            .result_with_session(|| node.session.clone())?;
        Ok(inp_idx.assume_init())
    }
}

pub fn check_for_specific_errors(
    node: &HoudiniNode,
    error_bits: raw::ErrorCode,
) -> Result<raw::ErrorCode> {
    unsafe {
        let mut code = uninit!();
        raw::HAPI_CheckForSpecificErrors(
            node.session.ptr(),
            node.handle.0,
            error_bits.0 as i32,
            code.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;
        Ok(raw::ErrorCode(code.assume_init() as u32))
    }
}

pub unsafe fn get_composed_cook_result(
    node: &HoudiniNode,
    verbosity: raw::StatusVerbosity,
) -> Result<String> {
    let mut len = uninit!();
    raw::HAPI_ComposeNodeCookResult(
        node.session.ptr(),
        node.handle.0,
        verbosity,
        len.as_mut_ptr(),
    )
    .result_with_session(|| node.session.clone())?;
    let len = len.assume_init();
    let mut buf = vec![0u8; len as usize];
    raw::HAPI_GetComposedNodeCookResult(node.session.ptr(), buf.as_mut_ptr() as *mut i8, len)
        .result_with_session(|| node.session.clone())?;
    buf.truncate(len as usize - 1);
    Ok(String::from_utf8_unchecked(buf))
}

pub fn get_time(session: &Session) -> Result<f32> {
    unsafe {
        let mut time = uninit!();
        raw::HAPI_GetTime(session.ptr(), time.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(time.assume_init())
    }
}

pub fn set_time(session: &Session, time: f32) -> Result<()> {
    unsafe { raw::HAPI_SetTime(session.ptr(), time).result_with_session(|| session.clone()) }
}

pub fn set_timeline_options(session: &Session, options: &raw::HAPI_TimelineOptions) -> Result<()> {
    unsafe {
        raw::HAPI_SetTimelineOptions(session.ptr(), options as *const _)
            .result_with_session(|| session.clone())
    }
}

pub fn get_timeline_options(session: &Session) -> Result<raw::HAPI_TimelineOptions> {
    unsafe {
        let mut opt = uninit!();
        raw::HAPI_GetTimelineOptions(session.ptr(), opt.as_mut_ptr())
            .result_with_session(|| session.clone())?;
        Ok(opt.assume_init())
    }
}

pub fn set_use_houdini_time(session: &Session, do_use: bool) -> Result<()> {
    unsafe {
        raw::HAPI_SetUseHoudiniTime(session.ptr(), do_use as i8)
            .result_with_session(|| session.clone())
    }
}

pub fn reset_simulation(node: &HoudiniNode) -> Result<()> {
    unsafe {
        raw::HAPI_ResetSimulation(node.session.ptr(), node.handle.0)
            .result_with_session(|| node.session.clone())
    }
}

pub fn get_hipfile_node_count(session: &Session, hip_file_id: i32) -> Result<u32> {
    unsafe {
        let mut count = uninit!();
        raw::HAPI_GetHIPFileNodeCount(session.ptr(), hip_file_id, count.as_mut_ptr());
        Ok(count.assume_init() as u32)
    }
}

pub fn get_geo_display_info(node: &HoudiniNode) -> Result<raw::HAPI_GeoInfo> {
    unsafe {
        let mut info = uninit!();
        raw::HAPI_GetDisplayGeoInfo(node.session.ptr(), node.handle.0, info.as_mut_ptr())
            .result_with_session(|| node.session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn get_geo_info(node: &HoudiniNode) -> Result<raw::HAPI_GeoInfo> {
    unsafe {
        let mut info = uninit!();
        raw::HAPI_GetGeoInfo(node.session.ptr(), node.handle.0, info.as_mut_ptr())
            .result_with_session(|| node.session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn get_part_info(node: &HoudiniNode, id: i32) -> Result<raw::HAPI_PartInfo> {
    unsafe {
        let mut info = uninit!();
        super::raw::HAPI_GetPartInfo(node.session.ptr(), node.handle.0, id, info.as_mut_ptr())
            .result_with_session(|| node.session.clone())?;
        Ok(info.assume_init())
    }
}

pub fn set_part_info(node: &HoudiniNode, info: &PartInfo) -> Result<()> {
    unsafe {
        super::raw::HAPI_SetPartInfo(
            node.session.ptr(),
            node.handle.0,
            info.part_id(),
            &info.inner,
        )
        .result_with_session(|| node.session.clone())
    }
}

pub fn get_attribute_names(
    node: &HoudiniNode,
    part_id: i32,
    count: i32,
    owner: raw::AttributeOwner,
) -> Result<StringsArray> {
    let mut handles = vec![0; count as usize];
    unsafe {
        raw::HAPI_GetAttributeNames(
            node.session.ptr(),
            node.handle.0,
            part_id,
            owner,
            handles.as_mut_ptr(),
            count,
        )
        .result_with_session(|| node.session.clone())?;
    }
    crate::stringhandle::get_strings_array(&handles, &node.session)
}

pub fn get_attribute_info(
    node: &HoudiniNode,
    part_id: i32,
    owner: raw::AttributeOwner,
    name: &CStr,
) -> Result<raw::HAPI_AttributeInfo> {
    let mut info = uninit!();
    unsafe {
        raw::HAPI_GetAttributeInfo(
            node.session.ptr(),
            node.handle.0,
            part_id,
            name.as_ptr(),
            owner,
            info.as_mut_ptr(),
        )
        .result_with_session(|| node.session.clone())?;

        Ok(info.assume_init())
    }
}

pub fn add_attribute(
    node: &HoudiniNode,
    part_id: i32,
    name: &CStr,
    attr_info: &raw::HAPI_AttributeInfo,
) -> Result<()> {
    unsafe {
        raw::HAPI_AddAttribute(
            node.session.ptr(),
            node.handle.0,
            part_id,
            name.as_ptr(),
            attr_info,
        )
        .result_with_session(|| node.session.clone())
    }
}

macro_rules! get_attrib_data {
    ($tp:ty, $default:literal, $func:ident, $ffi:ident) => {
        pub fn $func(
            node: &HoudiniNode,
            part_id: i32,
            name: &CStr,
            attr_info: &raw::HAPI_AttributeInfo,
            stride: i32,
            start: i32,
            length: i32,
        ) -> Result<Vec<$tp>> {
            unsafe {
                let mut data_array = Vec::new();
                data_array.resize((length * attr_info.tupleSize) as usize, $default);
                // SAFETY: Most likely an error in C API, it should not modify the info object,
                // but for some reason it wants a mut pointer
                let attr_info = attr_info as *const _ as *mut raw::HAPI_AttributeInfo;
                // let mut data_array = vec![];
                raw::$ffi(
                    node.session.ptr(),
                    node.handle.0,
                    part_id,
                    name.as_ptr(),
                    attr_info,
                    stride,
                    data_array.as_mut_ptr(),
                    start,
                    length,
                )
                .result_with_session(|| node.session.clone())?;
                Ok(data_array)
            }
        }
    };
}

macro_rules! set_attrib_data {
    ($tp:ty, $func:ident, $ffi:ident) => {
        pub fn $func(
            node: &HoudiniNode,
            part_id: i32,
            name: &CStr,
            attr_info: &raw::HAPI_AttributeInfo,
            data_array: &[$tp],
            start: i32,
            length: i32,
        ) -> Result<()> {
            unsafe {
                raw::$ffi(
                    node.session.ptr(),
                    node.handle.0,
                    part_id,
                    name.as_ptr(),
                    attr_info,
                    data_array.as_ptr(),
                    start,
                    length,
                )
                .result_with_session(|| node.session.clone())
            }
        }
    };
}

#[rustfmt::skip]
get_attrib_data!(f32, 0.0, get_attribute_float_data, HAPI_GetAttributeFloatData);
set_attrib_data!(f32, set_attribute_float_data, HAPI_SetAttributeFloatData);
#[rustfmt::skip]
get_attrib_data!(f64, 0.0, get_attribute_float64_data, HAPI_GetAttributeFloat64Data);
set_attrib_data!(
    f64,
    set_attribute_float64_data,
    HAPI_SetAttributeFloat64Data
);
#[rustfmt::skip]
get_attrib_data!(i32, 0, get_attribute_int_data, HAPI_GetAttributeIntData);
set_attrib_data!(i32, set_attribute_int_data, HAPI_SetAttributeIntData);
#[rustfmt::skip]
get_attrib_data!(i64, 0, get_attribute_int64_data, HAPI_GetAttributeInt64Data);
set_attrib_data!(i64, set_attribute_int64_data, HAPI_SetAttributeInt64Data);

pub fn get_attribute_string_buffer(
    node: &HoudiniNode,
    part_id: i32,
    name: &CStr,
    attr_info: &raw::HAPI_AttributeInfo,
    start: i32,
    length: i32,
) -> Result<StringsArray> {
    unsafe {
        let mut handles = Vec::new();
        handles.resize((length * attr_info.tupleSize) as usize, 0);
        // SAFETY: Most likely an error in C API, it should not modify the info object,
        // but for some reason it wants a mut pointer
        let attr_info = attr_info as *const _ as *mut raw::HAPI_AttributeInfo;
        raw::HAPI_GetAttributeStringData(
            node.session.ptr(),
            node.handle.0,
            part_id,
            name.as_ptr(),
            attr_info,
            handles.as_mut_ptr(),
            start,
            length,
        )
        .result_with_session(|| node.session.clone())?;
        crate::stringhandle::get_strings_array(&handles, &node.session)
    }
}

pub fn get_face_counts(node: &HoudiniNode, part_id: i32, length: i32) -> Result<Vec<i32>> {
    let mut array = vec![0; length as usize];
    unsafe {
        raw::HAPI_GetFaceCounts(
            node.session.ptr(),
            node.handle.0,
            part_id,
            array.as_mut_ptr(),
            0,
            length,
        )
        .result_with_session(|| node.session.clone())?;
    }
    Ok(array)
}

pub fn get_group_names(
    node: &HoudiniNode,
    group_type: raw::GroupType,
    count: i32,
) -> Result<StringsArray> {
    let mut handles = vec![0; count as usize];
    unsafe {
        raw::HAPI_GetGroupNames(
            node.session.ptr(),
            node.handle.0,
            group_type,
            handles.as_mut_ptr(),
            count,
        )
        .result_with_session(|| node.session.clone())?;
    }
    crate::stringhandle::get_strings_array(&handles, &node.session)
}

pub fn commit_geo(node: &HoudiniNode) -> Result<()> {
    unsafe {
        raw::HAPI_CommitGeo(node.session.ptr(), node.handle.0)
            .result_with_session(|| node.session.clone())
    }
}

pub fn save_geo_to_file(node: &HoudiniNode, filename: &CStr) -> Result<()> {
    unsafe {
        raw::HAPI_SaveGeoToFile(node.session.ptr(), node.handle.0, filename.as_ptr())
            .result_with_session(|| node.session.clone())
    }
}

pub fn load_geo_from_file(node: &HoudiniNode, filename: &CStr) -> Result<()> {
    unsafe {
        raw::HAPI_LoadGeoFromFile(node.session.ptr(), node.handle.0, filename.as_ptr())
            .result_with_session(|| node.session.clone())
    }
}

pub fn set_geo_vertex_list(node: &HoudiniNode, part_id: i32, list: &[i32]) -> Result<()> {
    unsafe {
        raw::HAPI_SetVertexList(
            node.session.ptr(),
            node.handle.0,
            part_id,
            list.as_ptr(),
            0,
            list.len() as i32,
        )
        .result_with_session(|| node.session.clone())
    }
}
pub fn set_geo_face_counts(node: &HoudiniNode, part_id: i32, list: &[i32]) -> Result<()> {
    unsafe {
        raw::HAPI_SetFaceCounts(
            node.session.ptr(),
            node.handle.0,
            part_id,
            list.as_ptr(),
            0,
            list.len() as i32,
        )
        .result_with_session(|| node.session.clone())
    }
}
