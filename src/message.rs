use crate::QoS;

#[derive(Debug, Clone)]
pub struct Message {
    pub(crate) topic: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) qos: QoS,
    pub(crate) retained: bool,
}

impl Message {
    pub fn new<T: Into<String>, P: Into<Vec<u8>>>(topic: T, payload: P) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            qos: QoS::AtMostOnce,
            retained: false,
        }
    }

    pub fn with_qos(mut self, qos: QoS) -> Self {
        self.qos = qos;
        self
    }

    pub fn with_retain(mut self, retained: bool) -> Self {
        self.retained = retained;
        self
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn qos(&self) -> QoS {
        self.qos
    }

    pub fn is_retained(&self) -> bool {
        self.retained
    }
}
