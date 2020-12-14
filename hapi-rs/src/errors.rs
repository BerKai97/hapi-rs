use crate::{auto::bindings as ffi, session::Session};
use std::borrow::Cow;
pub use ffi::HapiResult;

pub type Result<T> = std::result::Result<T, HapiError>;

#[derive(Debug)]
pub struct HapiError {
    pub kind: Kind,
    pub message: Option<Cow<'static, str>>,
    pub(crate) session: Option<Session>,
}

#[derive(Debug)]
pub enum Kind {
    Hapi(HapiResult),
    CookError,
    NullByte,
}

impl Kind {
    fn description(&self) -> &str {
        use HapiResult::*;

        match self {
            Kind::Hapi(Success) => "SUCCESS",
            Kind::Hapi(Failure) => "FAILURE",
            Kind::Hapi(AlreadyInitialized) => "ALREADY_INITIALIZED",
            Kind::Hapi(NotInitialized) => "NOT_INITIALIZED",
            Kind::Hapi(CantLoadfile) => "CANT_LOADFILE",
            Kind::Hapi(ParmSetFailed) => "PARM_SET_FAILED",
            Kind::Hapi(InvalidArgument) => "PARM_SET_FAILED",
            Kind::Hapi(CantLoadGeo) => "CANT_LOAD_GEO",
            Kind::Hapi(CantGeneratePreset) => "CANT_GENERATE_PRESET",
            Kind::Hapi(CantLoadPreset) => "CANT_LOAD_PRESET",
            Kind::Hapi(AssetDefAlreadyLoaded) => "ASSET_DEF_ALREADY_LOADED",
            Kind::Hapi(NoLicenseFound) => "NO_LICENSE_FOUND",
            Kind::Hapi(DisallowedNcLicenseFound) => "DISALLOWED_NC_LICENSE_FOUND",
            Kind::Hapi(DisallowedNcAssetWithCLicense) => "DISALLOWED_NC_ASSET_WITH_C_LICENSE",
            Kind::Hapi(DisallowedNcAssetWithLcLicense) => "DISALLOWED_NC_ASSET_WITH_LC_LICENSE",
            Kind::Hapi(DisallowedLcAssetWithCLicense) => "DISALLOWED_LC_ASSET_WITH_C_LICENSE",
            Kind::Hapi(DisallowedHengineindieW3partyPlugin) => {
                "DISALLOWED_HENGINEINDIE_W_3PARTY_PLUGIN"
            }
            Kind::Hapi(AssetInvalid) => "ASSET_INVALID",
            Kind::Hapi(NodeInvalid) => "NODE_INVALID",
            Kind::Hapi(UserInterrupted) => "USER_INTERRUPTED",
            Kind::Hapi(InvalidSession) => "INVALID_SESSION",
            Kind::NullByte => "String contains null byte!",
            Kind::CookError => "Cooking error",
            Kind::Hapi(_) => unreachable!(),
        }
    }
}

impl HapiError {
    pub(crate) fn new(
        kind: Kind,
        session: Option<Session>,
        message: Option<Cow<'static, str>>,
    ) -> HapiError {
        HapiError {
            kind,
            session,
            message,
        }
    }
}

impl std::fmt::Display for HapiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            Kind::Hapi(_) => {
                if let Some(session) = &self.session {
                    check_session!(session.ptr());
                    let error = session.get_status_string(
                        ffi::StatusType::CallResult,
                        ffi::StatusVerbosity::Statusverbosity0,
                    );
                    write!(
                        f,
                        "{}: {}",
                        self.kind.description(),
                        error
                            .ok()
                            .or_else(|| self.message.as_ref().map(|s| s.to_string()))
                            .unwrap_or_else(|| String::from("Zig"))
                    )
                } else if let Some(ref msg) = self.message {
                    write!(f, "Kind:{}, Message: {}", self.kind.description(), msg)
                } else {
                    write!(f, "Kind:{}", self.kind.description())
                }
            }
            _ => unreachable!(),
        }
    }
}

impl From<std::ffi::NulError> for HapiError {
    fn from(_: std::ffi::NulError) -> Self {
        HapiError::new(Kind::NullByte, None, None)
    }
}

#[macro_export]
macro_rules! hapi_result {
    ($hapi_result:expr, $ret:expr, $session:expr, $message:expr) => {
        match $hapi_result {
            HapiResult::Success => Ok($ret),
            e => Err(HapiError::new(Kind::Hapi(e.into()), $session, $message.map(|m| Cow::from(m)))),
        }
    };
}

#[macro_export]
macro_rules! hapi_err {
    ($hapi_result:expr, $session:expr, $message:expr) => {
        Err(HapiError::new(
            Kind::Hapi($hapi_result.into()),
            $session,
            $message.map(|v|Cow::from(v)),
        ))
    };

    ($hapi_result:expr) => {
        Err(HapiError::new(Kind::Hapi($hapi_result.into()), None, None))
    };
}

impl std::error::Error for HapiError {}

impl HapiResult {
    pub(crate) fn to_result<R: Default, F>(self, err: F) -> Result<R>
        where
            F: FnOnce() -> (Option<Session>, Option<Cow<'static, str>>),
    {
        match self {
            HapiResult::Success => Ok(R::default()),
            e => {
                let (session, message) = err();
                Err(HapiError::new(Kind::Hapi(e.into()), session, message))
            }
        }
    }
    pub(crate) fn result_with_session<F>(self, op: F) -> Result<()>
    where
        F: FnOnce() -> Session,
    {
        self.to_result(|| (Some(op()), None))
    }

    pub(crate) fn result_with_message<M: Into<Cow<'static, str>>>(self, msg: M) -> Result<()> {
        self.to_result(|| (None, Some(msg.into())))
    }
}
