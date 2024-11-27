use crate::bindings;
use crate::error::{Error, Result};
use crate::message::Message;
use crate::types::{ConnectionState, QoS};
use std::ffi::{CStr, CString};
use std::sync::Once;
use std::sync::{Arc, Mutex};

static INIT: Once = Once::new();

struct CallbackContext {
    message_callback: Box<dyn Fn(&Message) + Send + Sync>,
    state_callback: Box<dyn Fn(ConnectionState) + Send + Sync>,
}

pub struct Client {
    session: *mut bindings::mqtt_session_t,
}

impl Client {
    pub fn new<F1, F2>(client_id: &str, on_message: F1, on_state_change: F2) -> Result<Self>
    where
        F1: Fn(&Message) + Send + Sync + 'static,
        F2: Fn(ConnectionState) + Send + Sync + 'static,
    {
        // Initialize API once
        INIT.call_once(|| {
            let app_name = CString::new("RustMQTTClient").expect("Invalid app name");
            let app_version = CString::new("1.0").expect("Invalid version string");
            let debug = 0;
            let log_file = std::ptr::null();

            unsafe {
                bindings::mqtt_initialize(app_name.as_ptr(), app_version.as_ptr(), debug, log_file);
            }
        });

        let client_id = CString::new(client_id)?;
        let callback_context = Arc::new(Mutex::new(CallbackContext {
            message_callback: Box::new(on_message),
            state_callback: Box::new(on_state_change),
        }));

        let context_ptr = Arc::into_raw(callback_context) as *mut std::ffi::c_void;

        let session = unsafe {
            bindings::mqtt_create_session(
                client_id.as_ptr(),
                Some(Self::message_callback),
                Some(Self::state_callback),
                Some(Self::error_callback),
                context_ptr,
            )
        };

        if session.is_null() {
            return Err(Error::InitializationError);
        }

        Ok(Self { session })
    }

    // Rest of implementation remains the same...
    pub fn connect(&mut self, broker_url: &str, port: u16) -> Result<()> {
        let broker_url = CString::new(broker_url)?;

        let result = unsafe { bindings::mqtt_set_broker(self.session, broker_url.as_ptr(), port) };

        if result != 0 {
            return Err(Error::ConnectionError);
        }

        let result = unsafe { bindings::mqtt_session_start(self.session) };

        if result != 0 {
            Err(Error::ConnectionError)
        } else {
            Ok(())
        }
    }

    pub fn subscribe(&self, topic: &str, qos: QoS) -> Result<i64> {
        let topic = CString::new(topic)?;

        let handle = unsafe { bindings::mqtt_subscribe(self.session, topic.as_ptr(), qos.into()) };

        if handle < 0 {
            Err(Error::SubscriptionError)
        } else {
            Ok(handle)
        }
    }

    pub fn unsubscribe(&self, handle: i64) -> Result<()> {
        let result = unsafe { bindings::mqtt_unsubscribe(self.session, handle) };

        if result != 0 {
            Err(Error::SubscriptionError)
        } else {
            Ok(())
        }
    }

    pub fn publish(&self, message: &Message) -> Result<i64> {
        let topic = CString::new(&*message.topic)?;

        let message_id = unsafe {
            bindings::mqtt_publish(
                self.session,
                topic.as_ptr(),
                message.payload.as_ptr(),
                message.payload.len(),
                message.qos.into(),
                message.retained as i32,
            )
        };

        if message_id < 0 {
            Err(Error::PublicationError)
        } else {
            Ok(message_id)
        }
    }

    pub fn state(&self) -> ConnectionState {
        let state = unsafe { bindings::mqtt_session_get_state(self.session) };
        state.into()
    }

    unsafe extern "C" fn message_callback(
        message: *const bindings::mqtt_message_data_t,
        context: *mut std::ffi::c_void,
    ) {
        if message.is_null() || context.is_null() {
            return;
        }

        let context = Arc::from_raw(context as *const Mutex<CallbackContext>);

        if let Ok(guard) = context.lock() {
            let msg = Message {
                topic: CStr::from_ptr((*message).topic)
                    .to_string_lossy()
                    .into_owned(),
                payload: std::slice::from_raw_parts((*message).payload, (*message).payload_length)
                    .to_vec(),
                qos: match (*message).qos {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    2 => QoS::ExactlyOnce,
                    _ => return,
                },
                retained: (*message).retained != 0,
            };

            (guard.message_callback)(&msg);
        }

        std::mem::forget(context);
    }

    unsafe extern "C" fn state_callback(
        state: bindings::mqtt_session_state_t,
        context: *mut std::ffi::c_void,
    ) {
        if !context.is_null() {
            let context = Arc::from_raw(context as *const Mutex<CallbackContext>);
            if let Ok(guard) = context.lock() {
                (guard.state_callback)(state.into());
            }
            std::mem::forget(context);
        }
    }

    unsafe extern "C" fn error_callback(
        error_code: std::os::raw::c_int,
        message: *const std::os::raw::c_char,
        _context: *mut std::ffi::c_void,
    ) {
        if !message.is_null() {
            eprintln!(
                "MQTT Error {}: {}",
                error_code,
                CStr::from_ptr(message).to_string_lossy()
            );
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            bindings::mqtt_session_stop(self.session);
            bindings::mqtt_destroy_session(self.session);
        }
    }
}

unsafe impl Send for Client {}
unsafe impl Sync for Client {}
