#[cfg(target_os = "android")]
use tracing::{Event, Subscriber};
#[cfg(target_os = "android")]
use tracing_subscriber::{Layer, registry::Registry};
#[cfg(target_os = "android")]
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

#[cfg(target_os = "android")]
extern "C" {
    fn __android_log_write(prio: i32, tag: *const libc::c_char, text: *const libc::c_char);
}

#[cfg(target_os = "android")]
pub fn android_log_message(tag: &str, message: &str) {
    unsafe {
        __android_log_write(
            3, // Log priority: LogInfo
            tag.as_ptr() as *const libc::c_char,
            message.as_ptr() as *const libc::c_char
        );
    }
}
#[cfg(target_os = "android")]
struct AndroidLogLayer;
#[cfg(target_os = "android")]

impl<S> Layer<S> for AndroidLogLayer
    where
        S: Subscriber,
{
    fn on_event(&self, event: &Event, _context: tracing_subscriber::layer::Context<S>) {
        // Convert the event to a string and log it
        let message = format!("{:?}", event);
        crate::fetch_identity::android_log_message("my_tag", &message);
    }
}

#[cfg(target_os = "android")]
pub fn setup_logs() {
    let subscriber = tracing_subscriber::registry()
        .with(AndroidLogLayer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(std::io::stderr)
        )
        .with(
            tracing_subscriber::EnvFilter::new(
                "info,dash_sdk=trace,h2=info",
            )
        );

    //tracing::subscriber::set_global_default(subscriber)
    //    .expect("Unable to set global default subscriber");

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        crate::fetch_identity::android_log_message("platform-mobile", &*format!("Unable to set global default subscriber: {}", e));
    }
}

#[cfg(not(target_os = "android"))]
pub fn setup_logs() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "info,dash_sdk=trace,h2=info",
        ))
        .pretty()
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .try_init()
        .ok();
}