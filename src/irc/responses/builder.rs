//! Modulo que se centra en las funcionalidades referentes a la representacion de canales.
use std::collections::HashMap;

use super::{response::Response, ResponseType};
use crate::irc::responses::InternalType;
use crate::irc::{
    constants::ERR_SERVERERR, message::MessageError, responses::parsing_irc_defined_error_message,
};
pub struct ResponseBuilder {
    numeric_response: Vec<usize>,
    content: HashMap<usize, Vec<String>>,
    internal_response: Vec<InternalType>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            numeric_response: Vec::new(),
            content: HashMap::new(),
            internal_response: Vec::new(),
        }
    }

    fn push_response(&mut self, response: usize) -> &mut Vec<String> {
        self.content.insert(response, Vec::new());
        self.numeric_response.push(response);
        match self.content.get_mut(&response) {
            Some(v) => v,
            None => panic!("This shouldn't happen"),
        }
    }

    pub fn add_from_error(self, error: MessageError) -> Self {
        let response = match error {
            MessageError::IRCDefined(v) => v,
            _ => ERR_SERVERERR,
        };
        let response_message = parsing_irc_defined_error_message(response);
        self.add_content_for_response(response, response_message)
    }

    pub fn add_content_for_response(mut self, response: usize, content: String) -> Self {
        let res = match self.content.get_mut(&response) {
            Some(v) => v,
            None => self.push_response(response),
        };
        res.push(content);
        self
    }

    pub fn add_internal_response(mut self, response: InternalType) -> Self {
        self.internal_response.push(response);
        self
    }

    pub fn build(mut self) -> Vec<ResponseType> {
        let mut responses = Vec::new();
        if self.numeric_response.is_empty() && self.internal_response.is_empty() {
            responses.push(ResponseType::NoResponse);
            return responses;
        }
        for response in self.numeric_response {
            match self.content.get_mut(&response) {
                Some(v) => {
                    if v.is_empty() {
                        responses.push(ResponseType::Code(response))
                    } else {
                        for content in v {
                            responses.push(ResponseType::Content(Response::new(
                                response,
                                content.to_owned(),
                            )));
                        }
                    }
                }
                None => responses.push(ResponseType::Code(response)),
            };
        }
        for response in self.internal_response {
            responses.push(ResponseType::InternalResponse(response));
        }
        responses
    }
}
