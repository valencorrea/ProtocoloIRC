//! Modulo que se centra en las funcionalidades referentes al serializador de mensajes.
use super::{utils::generate_string, Command, COMMA};
use std::slice::Iter;

pub struct MessageSerializer {
    prefix: String,
    command: String,
    parameters: String,
}

impl MessageSerializer {
    pub fn new(prefix: Option<&[u8]>, command: Command) -> Self {
        Self {
            prefix: Self::prepend_colon(prefix),
            command: command.to_str().to_owned(),
            parameters: String::new(),
        }
    }

    pub fn add_parameter(self, parameter: &[u8]) -> Self {
        self.append_param(generate_string(parameter))
    }

    pub fn add_csl_params(self, parameters: &[&[u8]]) -> Self {
        self.add_parameter(&parameters.join(&COMMA))
    }

    pub fn add_number(self, parameter: u32) -> Self {
        self.append_param(parameter.to_string())
    }

    pub fn add_trailing_params(mut self, trailing: &[&[u8]]) -> Self {
        let mut iter = trailing.iter();
        let first = iter.next();
        if let Some(v) = first {
            self = self.append_param(Self::prepend_colon(Some(*v)));
        }
        self.add_iter(iter)
    }

    pub fn serialize(&self) -> String {
        let s = format!("{} {} {}\r\n", self.prefix, self.command, self.parameters)
            .trim()
            .to_owned();
        s
    }

    fn append_param(mut self, param: String) -> Self {
        if !self.parameters.is_empty() {
            self.parameters.push(' ');
        }

        self.parameters.push_str(&param);

        self
    }

    fn add_iter(mut self, iter: Iter<&[u8]>) -> Self {
        for param in iter {
            self = self.add_parameter(param)
        }

        self
    }

    fn prepend_colon(prefix: Option<&[u8]>) -> String {
        match prefix {
            Some(v) => Self::prepend_char(generate_string(v), ':'),
            None => String::new(),
        }
    }

    fn prepend_char(mut token: String, ch: char) -> String {
        token.insert(0, ch);
        token
    }
}

#[cfg(test)]
mod test {
    use crate::irc::message::Command;

    use super::MessageSerializer;

    #[test]
    fn test_serializer_with_no_params() {
        let serializer = MessageSerializer::new(Some(b"prefix"), Command::Nick);
        let serialized = serializer.serialize();

        assert_eq!(":prefix NICK", serialized);
    }

    #[test]
    fn test_serializer_no_params_no_prefix() {
        let serializer = MessageSerializer::new(None, Command::Nick);
        let serialized = serializer.serialize();

        assert_eq!("NICK", serialized);
    }

    #[test]
    fn test_serializer_add_simple_param() {
        let serializer = MessageSerializer::new(None, Command::Password).add_parameter(b"hola");
        let serialized = serializer.serialize();

        assert_eq!("PASS hola", serialized);
    }

    #[test]
    fn test_serializer_add_multiple_simple_param() {
        let serializer = MessageSerializer::new(None, Command::Password)
            .add_parameter(b"hola")
            .add_parameter(b"chau");
        let serialized = serializer.serialize();

        assert_eq!("PASS hola chau", serialized);
    }

    #[test]
    fn test_serializer_add_numeric_param() {
        let serializer = MessageSerializer::new(None, Command::Password).add_number(164u32);
        let serialized = serializer.serialize();

        assert_eq!("PASS 164", serialized);
    }

    // #[test]
    // fn test_serializer_add_csl_params() {
    //     let serializer =
    //         MessageSerializer::new(None, Command::Password).add_csl_params(&vec![b"hola", b"chau"]);
    //     let serialized = serializer.serialize();

    //     assert_eq!("PASS hola,chau", serialized);
    // }

    // #[test]
    // fn test_serializer_add_trailing_params() {
    //     let serializer = MessageSerializer::new(None, Command::Password)
    //         .add_trailing_params(vec![b"hola", b"chau"].as_ref());
    //     let serialized = serializer.serialize();

    //     assert_eq!("PASS :hola chau", serialized);
    // }
}
