use std::collections::VecDeque;

use async_trait::async_trait;

use tokio_mc::client::Client;
use tokio_mc::frame::{ProtocolError, Request, Response};
use tokio_mc::Error;

#[derive(Debug)]
pub struct MockClient {
    expected_responses: VecDeque<Response>,
    received_requests: Vec<Request<'static>>,
    should_fail: bool,
    fail_error: Option<Error>,
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            expected_responses: VecDeque::new(),
            received_requests: Vec::new(),
            should_fail: false,
            fail_error: None,
        }
    }

    pub fn expect_response(mut self, response: Response) -> Self {
        self.expected_responses.push_back(response);
        self
    }

    pub fn fail_with(mut self, error: Error) -> Self {
        self.should_fail = true;
        self.fail_error = Some(error);
        self
    }

    pub fn requests(&self) -> &[Request<'static>] {
        &self.received_requests
    }

    pub fn verify(&self) {
        assert!(
            self.expected_responses.is_empty(),
            "{} expected responses left unused",
            self.expected_responses.len()
        );
    }
}

#[async_trait]
impl Client for MockClient {
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        self.received_requests.push(request.into_owned());

        if self.should_fail {
            let error = self
                .fail_error
                .take()
                .unwrap_or_else(|| Error::Protocol(ProtocolError::OutOfRange));
            return Err(error);
        }

        self.expected_responses
            .pop_front()
            .ok_or_else(|| Error::Protocol(ProtocolError::OutOfRange))
    }
}
