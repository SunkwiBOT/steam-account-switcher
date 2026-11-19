use super::{Login, CODE_PERIOD_SECONDS};
use crate::util::now_unix;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use steamguard::protobufs::steammessages_auth_steamclient::EAuthTokenPlatformType;
use steamguard::steamapi::{ApiRequest, ApiResponse, BuildableRequest};
use steamguard::transport::{Transport, TransportError, WebApiTransport};
use steamguard::userlogin::{DeviceDetails, UserLogin};

pub fn now_seconds() -> u64 {
    now_unix() as u64
}

pub fn seconds_remaining(time: u64) -> u64 {
    let remainder = time % CODE_PERIOD_SECONDS;
    if remainder == 0 {
        CODE_PERIOD_SECONDS
    } else {
        CODE_PERIOD_SECONDS - remainder
    }
}

pub const USER_AGENT: &str = concat!("SteamAccountSwitcher/", env!("CARGO_PKG_VERSION"));

static HTTP_CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
});

pub fn http_client() -> reqwest::blocking::Client {
    HTTP_CLIENT.clone()
}

pub fn transport() -> WebApiTransport {
    WebApiTransport::new(http_client())
}

pub(super) fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "desktop".to_string())
}

// ! Steam's `EOSType` for "unknown", used by unofficial clients not tied to a specific OS build.
const OS_TYPE_UNKNOWN: i32 = -500;
// ! Steam's `EGamingDeviceType` for a standard desktop PC.
const GAMING_DEVICE_TYPE_PC: u32 = 528;

pub(super) fn device_details() -> DeviceDetails {
    DeviceDetails {
        friendly_name: format!("{} (Steam Account Switcher)", hostname()),
        platform_type: EAuthTokenPlatformType::k_EAuthTokenPlatformType_MobileApp,
        os_type: OS_TYPE_UNKNOWN,
        gaming_device_type: GAMING_DEVICE_TYPE_PC,
    }
}

pub fn new_login(control: Arc<AtomicBool>) -> Login {
    UserLogin::new(
        LoginTransport {
            inner: transport(),
            cancelled: control,
            deadline: Instant::now() + Duration::from_secs(180),
        },
        device_details(),
    )
}

// ! Stops steamguard's blocking poll loop at the next request after cancelling or timing out.
#[derive(Debug, Clone)]
pub struct LoginTransport {
    pub(super) inner: WebApiTransport,
    pub(super) cancelled: Arc<AtomicBool>,
    pub(super) deadline: Instant,
}

impl Transport for LoginTransport {
    fn send_request<Req: BuildableRequest + protobuf::MessageFull, Res: protobuf::MessageFull>(
        &self,
        request: ApiRequest<Req>,
    ) -> Result<ApiResponse<Res>, TransportError> {
        if self.cancelled.load(Ordering::SeqCst) || Instant::now() >= self.deadline {
            self.cancelled.store(true, Ordering::SeqCst);
            return Err(TransportError::Unknown(anyhow::anyhow!(
                "Login cancelled or expired. Start again."
            )));
        }
        self.inner.send_request(request)
    }

    fn close(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
