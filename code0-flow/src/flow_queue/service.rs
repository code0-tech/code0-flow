use std::ops::Add;

/// # Queue Prefix
/// - Every incoming message will have the `Send` prefix
/// - Every processed message (answer) from taurus will have the `Receive` prefix
pub enum QueuePrefix {
    Send,
    Receive,
}

/// Supported Protocols
pub enum QueueProtocol {
    Rest,
    WebSocket,
}

/// Implementation to turn a protocol into a str
impl QueueProtocol {

    /// Function to turn a protocol into a str
    ///
    /// # Example:
    /// ```
    /// use code0_flow::flow_queue::service::QueueProtocol;
    /// let proto_str = QueueProtocol::Rest.as_str().to_string();
    /// let result = "REST".to_string();
    ///
    /// assert_eq!(result, proto_str);
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            QueueProtocol::Rest => "REST",
            QueueProtocol::WebSocket => "WS",
        }
    }
}

/// Implementation to add a prefix and a protocol to a queue name
impl Add<QueueProtocol> for QueuePrefix {
    type Output = String;

    /// Function to add a prefix and a protocol to a queue name
    ///
    /// # Example:
    /// ```
    /// use code0_flow::flow_queue::service::{QueuePrefix, QueueProtocol};
    /// let send_rest_queue_name = QueuePrefix::Send + QueueProtocol::Rest;
    /// let result = "S_REST".to_string();
    ///
    /// assert_eq!(result, send_rest_queue_name);
    /// ```
    fn add(self, rhs: QueueProtocol) -> Self::Output {
        match self {
            QueuePrefix::Send => "S_".to_string() + rhs.as_str(),
            QueuePrefix::Receive => "R_".to_string() + rhs.as_str(),
        }
    }
}