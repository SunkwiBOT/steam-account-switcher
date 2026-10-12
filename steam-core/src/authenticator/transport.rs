use super::{Login, CODE_PERIOD_SECONDS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use steamguard::protobufs::steammessages_auth_steamclient::EAuthTokenPlatformType;
use steamguard::steamapi::{ApiRequest, ApiResponse, BuildableRequest};
use steamguard::transport::{Transport, TransportError, WebApiTransport};
use steamguard::userlogin::{DeviceDetails, UserLogin};

pub fn now_seconds() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0,
    }
}

pub fn seconds_remaining(time: u64) -> u64 {
    let remainder = time % CODE_PERIOD_SECONDS;
    if remainder == 0 {
        CODE_PERIOD_SECONDS
    } else {
        CODE_PERIOD_SECONDS - remainder
    }
}

pub fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!(
            "SteamAccountSwitcher/{}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
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

pub(super) fn device_details() -> DeviceDetails {
    DeviceDetails {
        friendly_name: format!("{} (Steam Account Switcher)", hostname()),
        platform_type: EAuthTokenPlatformType::k_EAuthTokenPlatformType_MobileApp,
        os_type: -500,
        gaming_device_type: 528,
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

/// Stops the library's blocking poll loop at the next request after cancelling
/// or timing out. The control remains in the store while Login is checked out.
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
