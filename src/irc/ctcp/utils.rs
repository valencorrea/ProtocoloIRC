use glib::Cast;
use gtk::traits::{ButtonExt, ContainerExt, ToggleButtonExt};
use gtk::{ButtonBox, Widget};

use crate::irc::ctcp::constants::{
    CHAT_PROTOCOL, ERR_INVALIDIP, ERR_INVALIDPORT, ERR_INVALIDPROTOCOL, ERR_INVALIDSIZE, IP_MAX_LEN,
};
use crate::irc::ctcp::message::send::DccSend;
use crate::irc::message::generic_message::GenericMessage;
use crate::irc::message::notice::Notice;
use crate::irc::message::utils::{generate_string_from_vec, try_parse_number};
use crate::irc::message::MessageError::DCCDefined;
use crate::irc::message::{utils, FromGeneric, MessageError, DOT};
use std::collections::vec_deque::VecDeque;

use super::constants::{MAX_PORT, MIN_PORT};
use super::message::chat::DccChat;
use super::message::close::DccClose;
use super::message::resume::DccResume;
use super::DccMessage;

pub const PATH_FILES_UPLOAD: &str = "../uploads";
pub const PATH_FILES_DOWNLOAD: &str = "../downloads";

pub fn get_ctcp_message_from_notice(incoming: &str) -> Option<String> {
    let notice: Notice<'_> = Notice::from_generic(GenericMessage::parse(incoming).ok()?).ok()?;

    let msg = generate_string_from_vec(&notice.text);

    if msg.len() != 2 && msg.len() < 2 {
        return None;
    }

    let cmd_b = msg.as_bytes();
    if *cmd_b.first()? != 1u8 || *cmd_b.last()? != 1u8 {
        return None;
    }

    let end = msg.len() - 1;
    Some(msg[1..end].to_string())
}

// valen:0x1 CTCP DCC CHAT chat <ip> <port>
pub fn get_ctcp_message(incoming: &str) -> Option<&str> {
    let splitted_msg: Vec<&str> = incoming[..incoming.len()].splitn(2, ":").collect();

    if splitted_msg.len() != 2 {
        return None;
    }

    let cmd = splitted_msg.get(1)?.trim();
    if cmd.len() < 2 {
        return None;
    }

    let cmd_b = cmd.as_bytes();
    if *cmd_b.first()? != 1u8 || *cmd_b.last()? != 1u8 {
        return None;
    }

    Some(&cmd[1..cmd.len() - 1])
}

fn get_first_token(incoming: &str) -> Option<&str> {
    let splitted_msg: Vec<&str> = incoming
        .split(" ")
        .filter(|token| !(*token).is_empty())
        .collect();
    splitted_msg.first().copied()
}

fn evaluate_first_token<'a>(incoming: &'a str, to: &'static str) -> Option<&'a str> {
    let first = get_first_token(incoming)?;

    if first.to_uppercase() != to {
        return None;
    }

    let next = incoming.find(first)? + to.len();
    Some(&incoming[next..])
}

pub fn get_ctcp_command(incoming: &str) -> Option<&str> {
    evaluate_first_token(incoming, "CTCP")
}

pub fn get_dcc_command(incoming: &str) -> Option<&str> {
    evaluate_first_token(incoming, "DCC")
}

pub fn upgrade_dcc_command(incoming: &str) -> Option<Box<dyn DccMessage>> {
    let first = get_first_token(incoming)?.to_uppercase();

    match first.as_str() {
        "SEND" => {
            let rest = evaluate_first_token(incoming, "SEND")?;
            match DccSend::parse(rest) {
                Ok(a) => Some(Box::new(a)),
                Err(_) => None,
            }
        }
        "RESUME" => {
            let rest = evaluate_first_token(incoming, "RESUME")?;
            match DccResume::parse(rest) {
                Ok(a) => Some(Box::new(a)),
                Err(_) => None,
            }
        }
        "CHAT" => {
            let rest = evaluate_first_token(incoming, "CHAT")?;
            match DccChat::parse(rest) {
                Ok(a) => Some(Box::new(a)),
                Err(_) => None,
            }
        }
        "CLOSE" => {
            let rest = evaluate_first_token(incoming, "CLOSE")?;
            match DccClose::parse(rest) {
                Ok(a) => Some(Box::new(a)),
                Err(_) => None,
            }
        }
        _ => {
            println!("UPGRADE DCC COMMAND FAILED");
            None
        }
    }
}

pub fn validate_filename(filename: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let invalid = [
        b"\\",
        b"/",
        b"\"",
        &[0u8],
        b":",
        b"?",
        b"<",
        b">",
        b"|",
        b" ",
    ];

    let f = filename.ok_or(MessageError::InvalidFormat)?;

    if f.is_empty() {
        return Err(MessageError::InvalidFormat);
    }

    for c in f {
        if invalid.contains(&&[*c]) {
            return Err(MessageError::InvalidFormat);
        }
    }
    Ok(f)
}

pub fn form_ctcp_cmd(cmd: &str) -> String {
    format!("\u{1}CTCP {}\u{1}", cmd)
}

pub fn to_dcc_command(incoming: &str) -> Option<Box<dyn DccMessage>> {
    let complete = get_ctcp_message(incoming)?;
    let ctcp = get_ctcp_command(complete)?;
    let dcc = get_dcc_command(ctcp)?;

    upgrade_dcc_command(dcc)
}

pub fn to_dcc_command_from_notice(incoming: &str) -> Option<Box<dyn DccMessage>> {
    let complete = get_ctcp_message_from_notice(incoming)?;
    let ctcp = get_ctcp_command(&complete)?;
    let dcc = get_dcc_command(ctcp)?;

    upgrade_dcc_command(dcc)
}

pub fn to_notice_command(nickname: String, dcc_command: String) -> String {
    format!("NOTICE {} :{}", nickname, form_ctcp_cmd(&dcc_command))
}

pub fn get_selected_nick(user_container: &ButtonBox) -> Option<String> {
    user_container
        .children()
        .iter()
        .find_map(|w: &Widget| {
            let b = w.clone().downcast::<gtk::RadioButton>().unwrap();
            match b.is_active() {
                true => b.label(),
                false => None,
            }
        })
        .map(|v| v.to_string())
}

#[cfg(test)]
mod test {

    mod test_get_ctcp_message {
        use crate::irc::ctcp::utils::{form_ctcp_cmd, get_ctcp_message};

        #[test]
        fn get_ctcp_message_correct_format() {
            let first = format!("alguien:{}\r\n", form_ctcp_cmd("DCC CHAT"));
            let second = format!("alguien:{}\r\n", form_ctcp_cmd("DCC SEND"));
            let third = format!("alguien:{}\r\n", form_ctcp_cmd("ALGO : asdf"));

            assert_eq!(get_ctcp_message(&first).unwrap(), "CTCP DCC CHAT");
            assert_eq!(get_ctcp_message(&second).unwrap(), "CTCP DCC SEND");
            assert_eq!(get_ctcp_message(&third).unwrap(), "CTCP ALGO : asdf");
        }

        #[test]
        fn get_ctcp_message_missing() {
            let first = format!("alguien:\u{1}{}\r\n", "DCC CHAT");
            let second = format!("alguien:{}\u{1}\r\n", "CTCP DCC SEND");
            let third = format!("alguien:{}\r\n", "ALGO : asdf");

            assert!(get_ctcp_message(&first).is_none());
            assert!(get_ctcp_message(&second).is_none());
            assert!(get_ctcp_message(&third).is_none());
        }
    }

    mod test_get_ctcp_command {
        use crate::irc::ctcp::utils::get_ctcp_command;

        #[test]
        fn get_ctcp_command_correct_format() {
            let first = "CTCP DCC CHAT";
            let second = "   CTCP DCC SEND";
            let third = "CTCP    Otras    cosas  que   vengan despues";

            assert_eq!(get_ctcp_command(&first).unwrap(), " DCC CHAT");
            assert_eq!(get_ctcp_command(&second).unwrap(), " DCC SEND");
            assert_eq!(
                get_ctcp_command(&third).unwrap(),
                "    Otras    cosas  que   vengan despues"
            );
        }

        #[test]
        fn get_ctcp_incorrect_format() {
            let first = "CCP bad";
            let second = "   Client-To-Client-Protocol DCC SEND";

            assert!(get_ctcp_command(&first).is_none());
            assert!(get_ctcp_command(&second).is_none());
        }
    }

    mod test_validate_filename {
        use crate::irc::ctcp::utils::validate_filename;

        #[test]
        fn valid_filenames() {
            let fn1 = b"holiwis.txt";
            let fn2 = b"UwU";
            let fn3 = b"jajan't";

            assert_eq!(validate_filename(Some(fn1)).unwrap(), b"holiwis.txt");
            assert_eq!(validate_filename(Some(fn2)).unwrap(), b"UwU");
            assert_eq!(validate_filename(Some(fn3)).unwrap(), b"jajan't");
        }

        #[test]
        fn invalid_filenames() {
            let fn1 = b"";
            let fn2 = b"/UwUn't";
            let fn3 = b"?ni alla";

            assert!(validate_filename(Some(fn1)).is_err());
            assert!(validate_filename(Some(fn2)).is_err());
            assert!(validate_filename(Some(fn3)).is_err());
        }
    }
}

pub fn validate_dcc_params_len(
    params: &VecDeque<&[u8]>,
    max_lenght: usize,
    min_lenght: usize,
    error: usize,
) -> Result<(), MessageError> {
    return utils::validate_params_len(params, max_lenght, min_lenght, DCCDefined(error));
}

pub fn validate_pos_number(s: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let file_size = match s {
        None => return Err(DCCDefined(ERR_INVALIDSIZE)),
        Some(size) => size,
    };

    Ok(file_size)
}

pub fn validate_port(p: Option<&[u8]>) -> Result<String, MessageError> {
    let port = match p {
        None => return Err(DCCDefined(ERR_INVALIDPORT)),
        Some(auxi_port) => try_parse_number(auxi_port)?,
    };

    if port < MIN_PORT || port > MAX_PORT {
        return Err(DCCDefined(ERR_INVALIDPORT));
    }

    Ok(port.to_string())
}

// todo revisar los errores de retorno
pub fn validate_ip(i: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let ip = match i {
        None => return Err(DCCDefined(ERR_INVALIDIP)),
        Some(auxi_ip) => auxi_ip,
    };

    if ip.starts_with(&[DOT]) || ip.ends_with(&[DOT]) {
        return Err(DCCDefined(ERR_INVALIDIP));
    };

    let split_address = ip.split(|byte| *byte == DOT).collect::<Vec<&[u8]>>();

    if split_address.len() > IP_MAX_LEN {
        return Err(DCCDefined(ERR_INVALIDIP));
    };

    Ok(ip)
}

// poner que solo aceptamos chat por convencion
pub fn validate_protocol(p: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let protocol = match p {
        None => return Err(DCCDefined(ERR_INVALIDPROTOCOL)),
        Some(auxi_protocol) => auxi_protocol,
    };

    if protocol.is_empty() || !protocol.eq(CHAT_PROTOCOL.as_bytes()) {
        return Err(DCCDefined(ERR_INVALIDPROTOCOL));
    };

    Ok(protocol)
}
