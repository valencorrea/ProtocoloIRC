//! Modulo que se centra en las funcionalidades genericas necesarias para resolver
//! los distintos modulos
use std::{collections::vec_deque::VecDeque, net::SocketAddr, str::FromStr};

use super::{
    ChannelModes, ModesAction, UserModes, AMPERSAND, ASTERISK, BELL, COMMA, DOLLAR, HASH,
    QUESTION_MARK,
};
use crate::irc::{
    constants::ERR_NEEDMOREPARAMS,
    message::{
        Command, MessageError, MessageError::*, BRACKET_CLOSE, BRACKET_OPEN, COLON, CR, DOT, LF,
        MAX_HOSTNAME_LABEL_LENGTH, MAX_HOSTNAME_LENGTH, NUL, SPACE,
    },
};
use std::str;

pub fn split_message(message: &str) -> VecDeque<&[u8]> {
    let bytes = message.as_bytes();

    let separated = bytes
        .split(|byte| *byte == SPACE)
        .filter(|word| !(*word).is_empty())
        .collect::<VecDeque<&[u8]>>();

    separated
}

pub fn starts_with(param: &[u8], c: u8) -> bool {
    param.starts_with(&[c])
}

pub fn starts_with_colon(param: &[u8]) -> bool {
    starts_with(param, COLON)
}

fn strip(param: &[u8], c: u8) -> Option<&[u8]> {
    param.strip_prefix(&[c])
}

pub fn strip_colon(param: &[u8]) -> Option<&[u8]> {
    strip(param, COLON)
}

pub fn check_none(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    match n {
        Some(v) => Ok(v),
        None => Err(InvalidFormat),
    }
}

fn validate_name(name: &[u8]) -> Result<&[u8], MessageError> {
    let special = [b'-', b'[', b']', b'\\', b'`', b'^', b'{', b'}'];

    let mut iter = name.iter();
    match iter.next() {
        Some(v) => {
            if !v.is_ascii_alphabetic() {
                return Err(InvalidFormat);
            }
        }
        None => return Err(InvalidFormat),
    };

    for c in iter {
        if !(c.is_ascii_alphanumeric() || special.contains(c)) {
            return Err(InvalidFormat);
        }
    }

    Ok(name)
}

pub fn validate_name_invalid_none(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    match n {
        Some(v) => validate_name(v),
        None => Err(InvalidFormat),
    }
}

pub fn validate_name_valid_none(n: Option<&[u8]>) -> Result<Option<&[u8]>, MessageError> {
    match n {
        Some(v) => Ok(Some(validate_name(v)?)),
        None => Ok(None),
    }
}

pub fn validate_irc_params_len(
    params: &VecDeque<&[u8]>,
    max_lenght: usize,
    min_lenght: usize,
    error: usize,
) -> Result<(), MessageError> {
    return validate_params_len(params, max_lenght, min_lenght, IRCDefined(error));
}

#[allow(dead_code)]
pub fn validate_str_invalid_none(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    match n {
        Some(v) => Ok(v),
        None => Err(InvalidFormat),
    }
}

#[allow(dead_code)]
pub fn validate_str_starting_w_colon(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let name = match n {
        Some(v) => v,
        None => return Err(InvalidFormat),
    };

    if name[0] != COLON {
        return Err(InvalidFormat);
    }

    validate_str_invalid_none(Some(&name[1..name.len()]))
}

pub fn validate_realname(mut params: VecDeque<&[u8]>) -> Result<Vec<&[u8]>, MessageError> {
    if params.is_empty() {
        return Err(InvalidFormat);
    }

    first_param_starts_with_colon(&params)?;

    strip_colon_from_first_param(&mut params)?;

    Ok(Vec::from(params))
}

pub fn validate_hostname(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    match n {
        Some(v) => {
            let stringified = generate_string(v);
            let split = stringified.split(':').collect::<Vec<&str>>();
            if split.len() > 2 && SocketAddr::from_str(&stringified).is_err() {
                return Err(InvalidFormat);
            }
            let name = match split.first() {
                Some(v) => v.as_bytes(),
                None => return Err(InvalidFormat),
            };

            validate_length(MAX_HOSTNAME_LENGTH, name.len())?;

            let series = name
                .split(|label| *label == DOT)
                .collect::<VecDeque<&[u8]>>();

            for label in series {
                validate_label_in_hostname(label)?;
            }

            Ok(v)
        }
        None => Err(InvalidFormat),
    }
}

fn validate_length(lenght_max: usize, lenght_input: usize) -> Result<(), MessageError> {
    if (lenght_input == 0) || (lenght_input > lenght_max) {
        return Err(InvalidFormat);
    }
    Ok(())
}

fn validate_label_in_hostname(label: &[u8]) -> Result<(), MessageError> {
    validate_length(MAX_HOSTNAME_LABEL_LENGTH, label.len())?;

    for l in label {
        if !l.is_ascii_alphanumeric() {
            return Err(InvalidFormat);
        }
    }
    Ok(())
}

pub fn nonwhite(u: &u8) -> bool {
    *u != SPACE && *u != NUL && *u != CR && *u != LF
}

pub fn validate_user(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let name = match n {
        Some(v) => v,
        None => return Err(InvalidFormat),
    };

    if name.is_empty() {
        return Err(InvalidFormat);
    }

    for s in name {
        if !nonwhite(s) {
            return Err(InvalidFormat);
        }
    }

    Ok(name)
}

pub fn validate_password(n: Option<&[u8]>) -> Result<&[u8], MessageError> {
    check_none(n) // TODO since irc doesnt details it, should we build our own validations?
}

#[allow(dead_code)]
pub fn validate_message(n: Option<&[u8]>) -> Result<Option<&[u8]>, MessageError> {
    match n {
        Some(_) => Ok(Some(validate_str_starting_w_colon(n)?)),
        None => Ok(None),
    }
}

pub fn generate_string(token: &[u8]) -> String {
    match str::from_utf8(token) {
        Ok(v) => v.to_owned(),
        Err(_) => String::new(),
    }
}

pub fn generate_string_from_vec(tokens: &[&[u8]]) -> String {
    let mut s = String::new();
    for token in tokens {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&generate_string(token))
    }
    s
}

pub fn try_parse_number(value: &[u8]) -> Result<u32, MessageError> {
    match std::str::from_utf8(value) {
        Ok(s) => match s.parse::<u32>() {
            Ok(v) => Ok(v),
            Err(_) => Err(InvalidFormat),
        },
        Err(_) => Err(InvalidFormat),
    }
}

pub fn validate_params_len(
    params: &VecDeque<&[u8]>,
    max_lenght: usize,
    min_lenght: usize,
    error: MessageError,
) -> Result<(), MessageError> {
    if params.len() > max_lenght {
        return Err(TooManyParams);
    }

    if params.len() < min_lenght {
        return Err(error);
    }
    Ok(())
}

pub fn validate_command(input: Command, expected: Command) -> Result<(), MessageError> {
    if input != expected {
        return Err(InvalidCommand);
    }
    Ok(())
}

pub fn split_csl(csl: Option<&[u8]>) -> Result<Vec<&[u8]>, MessageError> {
    let _csl = match csl {
        Some(v) => v.split(|byte| *byte == COMMA).collect::<Vec<&[u8]>>(),
        None => return Err(InvalidFormat),
    };

    Ok(_csl)
}

pub fn validate_hostmask(hostmask: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let _hostmask = match hostmask {
        Some(v) => v,
        None => return Err(InvalidFormat),
    };

    if _hostmask.is_empty() {
        return Err(InvalidFormat);
    }

    let is_hostmask = _hostmask.starts_with(&[DOLLAR]);

    if !is_hostmask {
        return Err(InvalidHostmask);
    }

    if !_hostmask.contains(&DOT) {
        return Err(InvalidHostmask);
    }

    let last = _hostmask
        .rsplitn(2, |tok| *tok == DOT)
        .collect::<Vec<&[u8]>>()[0];

    if last.contains(&QUESTION_MARK) || last.contains(&ASTERISK) {
        return Err(InvalidHostmask);
    }

    Ok(_hostmask)
}

pub fn validate_receiver(receiver: &[u8]) -> Result<(), MessageError> {
    if receiver.is_empty() {
        return Err(InvalidFormat);
    }

    let valid_hostmask = validate_hostmask(Some(receiver));

    if valid_hostmask.is_ok() {
        return Ok(());
    }

    if let Err(e) = valid_hostmask {
        if e != MessageError::InvalidHostmask {
            println!("Invalid format?");
            return Err(e);
        }
    }

    let valid_channel = validate_channel(Some(receiver));

    if valid_channel.is_ok() {
        return Ok(());
    }

    validate_name(receiver)?;

    Ok(())
}

pub fn validate_receivers(receivers: &[&[u8]]) -> Result<(), MessageError> {
    for receiver in receivers.iter() {
        validate_receiver(receiver)?;
    }

    Ok(())
}

fn first_param_starts_with_colon(params: &VecDeque<&[u8]>) -> Result<(), MessageError> {
    match params.front() {
        Some(v) => {
            if !starts_with_colon(v) {
                return Err(InvalidFormat);
            }
        }
        None => return Err(InvalidFormat),
    }

    Ok(())
}

fn strip_colon_from_first_param(params: &mut VecDeque<&[u8]>) -> Result<(), MessageError> {
    match params.pop_front() {
        Some(v) => {
            match strip_colon(v) {
                Some(v) => params.push_front(v),
                None => return Err(InvalidFormat),
            };
        }
        None => return Err(InvalidFormat),
    }

    Ok(())
}

pub fn validate_text(mut params: VecDeque<&[u8]>) -> Result<Vec<&[u8]>, MessageError> {
    first_param_starts_with_colon(&params)?;

    strip_colon_from_first_param(&mut params)?;

    Ok(Vec::from(params))
}

fn is_valid_chstring_byte(ch: &u8) -> bool {
    !(*ch == SPACE || *ch == BELL || *ch == NUL || *ch == CR || *ch == LF || *ch == COMMA)
}

pub fn validate_channel(c: Option<&[u8]>) -> Result<&[u8], MessageError> {
    let channel = match c {
        Some(v) => v,
        None => return Err(InvalidFormat),
    };

    if !(starts_with(channel, HASH) || starts_with(channel, AMPERSAND)) {
        return Err(InvalidFormat);
    }

    if channel.is_empty() {
        return Err(InvalidFormat);
    }

    for c in channel.iter() {
        if !is_valid_chstring_byte(c) {
            return Err(InvalidFormat);
        }
    }

    Ok(channel)
}

pub fn validate_channels(channels: Option<&[u8]>) -> Result<Vec<&[u8]>, MessageError> {
    let channels = split_csl(channels)?;

    for channel in channels.iter() {
        let _ = validate_channel(Some(*channel))?;
    }

    Ok(channels)
}

pub fn split_csl_none(keys: Option<&[u8]>) -> Option<Vec<&[u8]>> {
    let _csl = match keys {
        Some(v) => v.split(|byte| *byte == COMMA).collect::<Vec<&[u8]>>(),
        None => return None,
    };

    Some(_csl)
}

pub fn validate_realname_valid_none(n: Option<&[u8]>) -> Result<Option<&[u8]>, MessageError> {
    //TODO refactor
    let name = match n {
        Some(v) => v,
        None => return Ok(None),
    };

    if name[0] != COLON {
        return Err(InvalidFormat);
    }

    validate_name_valid_none(Some(&name[1..name.len()]))
}

pub fn validate_nickmask(nickmask: Option<&[u8]>) -> Result<&[u8], MessageError> {
    match nickmask {
        Some(v) => Ok(v),
        None => Err(InvalidFormat),
    }
}

pub fn validate_nickmasks(nickmasks: Option<&[u8]>) -> Result<Vec<&[u8]>, MessageError> {
    let nick_masks = split_csl(nickmasks)?;

    for nickmask in &nick_masks {
        let _ = validate_nickmask(Some(nickmask))?;
    }

    Ok(nick_masks)
}

pub fn validate_o_param(o: Option<&[u8]>) -> Result<bool, MessageError> {
    match o {
        Some(v) => {
            if v.eq(b"o") {
                Ok(true)
            } else {
                Err(InvalidFormat)
            }
        }
        None => Ok(false),
    }
}

pub fn validate_channel_modes(
    mut params: VecDeque<&[u8]>,
) -> Result<ModesAction<ChannelModes>, MessageError> {
    match params.pop_front() {
        Some(action) => {
            if action.len() != 2 {
                return Err(MessageError::InvalidFormat);
            }
            match action[0] {
                b'+' => Ok(ModesAction::Add(ChannelModes::parse(action[1], params)?)),
                b'-' => Ok(ModesAction::Remove(ChannelModes::parse(action[1], params)?)),
                _ => Err(MessageError::InvalidFormat),
            }
        }
        None => Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)),
    }
}

pub fn validate_user_modes(
    mut params: VecDeque<&[u8]>,
) -> Result<ModesAction<UserModes>, MessageError> {
    match params.pop_front() {
        Some(action) => {
            if action.len() != 2 {
                return Err(MessageError::InvalidFormat);
            }
            match action[0] {
                b'+' => Ok(ModesAction::Add(UserModes::parse(action[1])?)),
                b'-' => Ok(ModesAction::Remove(UserModes::parse(action[1])?)),
                _ => Err(MessageError::InvalidFormat),
            }
        }
        None => Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)),
    }
}

pub fn no_such_nick(nick: &[u8]) -> String {
    format!("{} :No such nick/channel", generate_string(nick))
}

fn top(n: &[u8]) -> usize {
    match n.len() {
        0 => 0,
        i => i - 1,
    }
}

pub fn retrieve_hostname(n: &[u8]) -> Result<Option<&[u8]>, MessageError> {
    {
        if n[0] == BRACKET_OPEN && n[top(n)] == BRACKET_CLOSE {
            let a = match n.strip_prefix(&[BRACKET_OPEN]) {
                Some(v) => v,
                None => return Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)),
            };
            Ok(a.strip_suffix(&[BRACKET_CLOSE]))
        } else {
            Ok(None)
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;

    mod test_parse_number {
        use super::*;

        #[test]
        fn test_parse_number_valid_slice() {
            let to_parse_one = 32u32.to_string();
            let to_parse_two = 456481u32.to_string();
            let to_parse_three = b"1234";

            let res_one = try_parse_number(to_parse_one.as_bytes());
            let res_two = try_parse_number(to_parse_two.as_bytes());
            let res_three = try_parse_number(to_parse_three);

            assert_eq!(res_one.unwrap(), 32u32);
            assert_eq!(res_two.unwrap(), 456481u32);
            assert_eq!(res_three.unwrap(), 1234);
        }

        #[test]
        fn test_parse_number_invalid_slice() {
            let to_parse_one = b"-123";
            let to_parse_two = b"0x12345";

            let res_one = try_parse_number(to_parse_one);
            let res_two = try_parse_number(to_parse_two);

            assert!(res_one.is_err());
            assert!(res_two.is_err());
        }
    }

    mod test_valid_nickname {
        use crate::irc::message::{utils::validate_name, MessageError};

        #[test]
        fn test_valid_nicknames_are_valid() {
            let valid_one: &[u8] = b"valid";
            let valid_special: &[u8] = b"another-valid";
            let valid_numerical: &[u8] = b"numerical123";
            let valid_combination: &[u8] = b"numerical[123]";

            assert_eq!(validate_name(valid_one), Ok(valid_one));
            assert_eq!(validate_name(valid_special), Ok(valid_special));
            assert_eq!(validate_name(valid_numerical), Ok(valid_numerical));
            assert_eq!(validate_name(valid_combination), Ok(valid_combination));
        }

        #[test]
        fn test_invalid_nicknames() {
            let starts_no_alpha: &[u8] = b"1asdf";
            let invalid_special: &[u8] = b"invalid_special";

            assert_eq!(
                validate_name(starts_no_alpha),
                Err(MessageError::InvalidFormat)
            );
            assert_eq!(
                validate_name(invalid_special),
                Err(MessageError::InvalidFormat)
            );
        }
    }

    mod test_valid_realname {
        use crate::irc::message::utils::validate_realname;
        use crate::irc::message::MessageError::InvalidFormat;
        use std::collections::vec_deque::VecDeque;

        #[test]
        fn test_valid_realname_without_space_is_valid() {
            let valid: &[u8] = b":valid";
            let valid_res: &[u8] = b"valid";
            let mut valid_v = VecDeque::new();
            valid_v.push_back(valid);

            assert_eq!(validate_realname(valid_v).unwrap()[0], valid_res);
        }

        #[test]
        fn test_valid_realname_with_space_is_valid() {
            let mut valid_v: VecDeque<&[u8]> = VecDeque::new();
            valid_v.push_back(b":another");
            valid_v.push_back(b"valid");

            let res = validate_realname(valid_v).unwrap();
            assert_eq!(res[0], b"another");
            assert_eq!(res[1], b"valid");
        }

        #[test]
        fn test_valid_realname_without_colon_is_invalid() {
            let mut valid_v: VecDeque<&[u8]> = VecDeque::new();
            valid_v.push_back(b"invalid");

            assert_eq!(validate_realname(valid_v).unwrap_err(), InvalidFormat);
        }

        #[test]
        fn test_valid_realname_empty_is_invalid() {
            let valid_v: VecDeque<&[u8]> = VecDeque::new();
            assert_eq!(validate_realname(valid_v).unwrap_err(), InvalidFormat);
        }
    }

    mod test_valid_hostname {
        use crate::irc::message::utils::validate_hostname;
        use crate::irc::message::MessageError::InvalidFormat;
        use crate::irc::message::MAX_HOSTNAME_LABEL_LENGTH;
        use crate::irc::message::MAX_HOSTNAME_LENGTH;

        #[test]
        fn test_valid_hostname() {
            let valid_one: &[u8] = b"10.0.0.0";
            let valid_two: &[u8] = b"128.10.0.0";
            let valid_three: &[u8] = b"10.0.0.77";

            assert_eq!(validate_hostname(Some(valid_one)), Ok(valid_one));
            assert_eq!(validate_hostname(Some(valid_two)), Ok(valid_two));
            assert_eq!(validate_hostname(Some(valid_three)), Ok(valid_three));
        }

        #[test]
        fn test_valid_hostname_local() {
            let valid: &[u8] = b"0";

            assert_eq!(validate_hostname(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_hostname_port() {
            let valid: &[u8] = b"127.0.0.1:8080";
            let valid2: &[u8] = b"127.0.0.1:10000";

            assert_eq!(validate_hostname(Some(valid)), Ok(valid));
            assert_eq!(validate_hostname(Some(valid2)), Ok(valid2));
        }

        #[test]
        fn test_invalid_hostname_empty() {
            let invalid: &[u8] = b"";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_exceeds_length() {
            let mut invalid = String::new();

            for _i in 0..(MAX_HOSTNAME_LENGTH + 1) {
                invalid.push('a');
            }

            assert_eq!(
                validate_hostname(Some(invalid.as_bytes())),
                Err(InvalidFormat)
            );
        }

        #[test]
        fn test_invalid_hostname_label_exceeds_length() {
            let mut invalid = String::new();

            for _i in 0..2 {
                for _j in 0..(MAX_HOSTNAME_LABEL_LENGTH + 1) {
                    invalid.push('a');
                }
                invalid.push('.');
            }

            assert_eq!(
                validate_hostname(Some(invalid.as_bytes())),
                Err(InvalidFormat)
            );
        }

        #[test]
        fn test_invalid_hostname_empty_label() {
            let mut invalid = String::new();

            for _i in 0..2 {
                invalid.push('.');
            }

            assert_eq!(
                validate_hostname(Some(invalid.as_bytes())),
                Err(InvalidFormat)
            );
        }

        #[test]
        fn test_invalid_hostname_with_space() {
            let invalid: &[u8] = b"10.0.0 77";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_exclamation() {
            let invalid: &[u8] = b"10.0.0!77";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_at() {
            let invalid: &[u8] = b"10.0.0@77";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_one_dot() {
            let invalid: &[u8] = b".";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_two_dots() {
            let invalid: &[u8] = b"10..";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_final_dot() {
            let invalid: &[u8] = b"10.";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_two_invalid_characters() {
            let invalid: &[u8] = b"10!!0";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_hostname_with_only_space() {
            let invalid: &[u8] = b" ";

            assert_eq!(validate_hostname(Some(invalid)), Err(InvalidFormat));
        }
    }

    mod test_validate_user {
        use crate::irc::message::utils::validate_user;
        use crate::irc::message::MessageError::InvalidFormat;

        #[test]
        fn test_valid_user_only_letters() {
            let valid: &[u8] = b"valid";

            assert_eq!(validate_user(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_user_only_numbers() {
            let valid: &[u8] = b"1234";

            assert_eq!(validate_user(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_user_only_spec_characters() {
            let valid: &[u8] = b"!#$%&/()=?"; // note that ¡¿ are not ascii

            assert_eq!(validate_user(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_user_with_letters_numbers_spec_characters() {
            let valid: &[u8] = b"uS3r:)";

            assert_eq!(validate_user(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_invalid_user_with_space() {
            let invalid: &[u8] = b"in valid";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_user_with_nul() {
            let invalid: &[u8] = b"";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_user_with_cr() {
            let invalid: &[u8] = b"in\rvalid";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_user_with_lf() {
            let invalid: &[u8] = b"in\nvalid";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_user_with_cr_lf() {
            let invalid: &[u8] = b"in\r\nvalid";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }

        #[test]
        fn test_invalid_user_with_manual_cr_lf() {
            let invalid: &[u8] = b"in
            valid";

            assert_eq!(validate_user(Some(invalid)), Err(InvalidFormat));
        }
    }

    mod test_validate_password {
        use crate::irc::message::utils::validate_password;
        use crate::irc::message::MessageError::InvalidFormat;

        #[test]
        fn test_valid_password_only_letters() {
            let valid: &[u8] = b"valid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_only_numbers() {
            let valid: &[u8] = b"1234";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_only_spec_characters() {
            let valid: &[u8] = b"!#$%&/()=?"; // note that ¡¿ are not ascii

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_letters_numbers_spec_characters() {
            let valid: &[u8] = b"uS3r:)";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_space() {
            let valid: &[u8] = b"va lid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_cr() {
            let valid: &[u8] = b"va\rlid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_lf() {
            let valid: &[u8] = b"va\nlid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_cr_lf() {
            let valid: &[u8] = b"va\r\nlid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_valid_password_with_manual_cr_lf() {
            let valid: &[u8] = b"va
            lid";

            assert_eq!(validate_password(Some(valid)), Ok(valid));
        }

        #[test]
        fn test_invalid_password_with_nul() {
            let invalid = None;

            assert_eq!(validate_password(invalid), Err(InvalidFormat));
        }
    }

    // mod test_validate_message {
    //     use crate::irc::message::utils::validate_message;

    //     #[test]
    //     fn test_valid_message_only_letters() {
    //         let valid: Option<&[u8]> = Some(b":valid");
    //         let res: Option<&[u8]> = Some(b"valid");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_only_numbers() {
    //         let valid: Option<&[u8]> = Some(b":1234");
    //         let res: Option<&[u8]> = Some(b"1234");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_only_spec_characters() {
    //         let valid: Option<&[u8]> = Some(b":!#$%&/()=?"); // note that ¡¿ are not ascii
    //         let res: Option<&[u8]> = Some(b"!#$%&/()=?");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_letters_numbers_spec_characters() {
    //         let valid: Option<&[u8]> = Some(b":uS3r");
    //         let res: Option<&[u8]> = Some(b"uS3r");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_space() {
    //         let valid: Option<&[u8]> = Some(b":va lid");
    //         let res: Option<&[u8]> = Some(b"va lid");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_cr() {
    //         let valid: Option<&[u8]> = Some(b":va\rlid");
    //         let res: Option<&[u8]> = Some(b"va\rlid");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_lf() {
    //         let valid: Option<&[u8]> = Some(b":va\nlid");
    //         let res: Option<&[u8]> = Some(b"va\nlid");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_cr_lf() {
    //         let valid: Option<&[u8]> = Some(b":va\r\nlid");
    //         let res: Option<&[u8]> = Some(b"va\r\nlid");

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_manual_cr_lf() {
    //         let valid: Option<&[u8]> = Some(
    //             b":va
    //         lid",
    //         );
    //         let res: Option<&[u8]> = Some(
    //             b"va
    //         lid",
    //         );

    //         assert_eq!(validate_message(valid).unwrap(), res);
    //     }

    //     #[test]
    //     fn test_valid_message_with_null() {
    //         let valid: Option<&[u8]> = None;

    //         assert!(validate_message(valid).unwrap().is_none());
    //     }
    // }

    mod test_split {
        use crate::irc::message::utils::split_message;

        #[test]
        fn test_valid_splits() {
            let test_one = "This is a test";
            let test_two = "This is another test with longer a longer sentence";
            let test_three = "This test with   a   free   spaces  ";

            assert_eq!(split_message(test_one).len(), 4);
            assert_eq!(split_message(test_two).len(), 9);
            assert_eq!(split_message(test_three).len(), 6);
        }

        #[test]
        fn test_empty() {
            let empty = "";
            let pure_space = "       ";

            assert_eq!(split_message(empty).len(), 0);
            assert_eq!(split_message(pure_space).len(), 0);
        }
    }

    mod test_split_csl {
        use crate::irc::message::{utils::split_csl, MessageError};

        #[test]
        fn test_split_csl_list_is_correct() {
            let list = b"comma,separated,list";

            let splitted = split_csl(Some(list)).unwrap();

            assert_eq!(splitted.len(), 3);

            assert_eq!(splitted[0], b"comma");
            assert_eq!(splitted[1], b"separated");
            assert_eq!(splitted[2], b"list");
        }

        #[test]
        fn test_split_csl_empties() {
            let list = b"comma,,separated,list";

            let splitted = split_csl(Some(list)).unwrap();

            assert_eq!(splitted.len(), 4);

            assert_eq!(splitted[0], b"comma");
            assert_eq!(splitted[1], b"");
            assert_eq!(splitted[2], b"separated");
            assert_eq!(splitted[3], b"list");
        }

        #[test]
        fn test_split_csl_empty() {
            let list = b"";

            let splitted = split_csl(Some(list)).unwrap();

            assert_eq!(splitted.len(), 1);

            assert_eq!(splitted[0], b"");
        }

        #[test]
        fn test_split_csl_none() {
            let splitted = split_csl(None).unwrap_err();

            assert_eq!(splitted, MessageError::InvalidFormat)
        }
    }

    mod test_validate_receiver {
        use crate::irc::message::{utils::validate_receiver, MessageError};

        #[test]
        fn test_valid_receiver() {
            let receiver = b"Angel";
            let receiver_2 = b"e132";
            let res = validate_receiver(receiver).unwrap();
            let res_2 = validate_receiver(receiver_2).unwrap();

            assert_eq!(res, ());
            assert_eq!(res_2, ());
        }

        #[test]
        fn test_valid_receiver_hostmask() {
            let hostmask = b"#*.edu";
            let hostmask_2 = b"$*.fi";

            let res = validate_receiver(hostmask).unwrap();
            let res_2 = validate_receiver(hostmask_2).unwrap();

            assert_eq!(res, ());
            assert_eq!(res_2, ());
        }

        #[test]
        fn test_invalid_receiver_hostmask() {
            let inv = b"#\n";
            let inv2 = b"$?";

            let res = validate_receiver(inv).unwrap_err();
            let res_2 = validate_receiver(inv2).unwrap_err();

            assert_eq!(res, MessageError::InvalidFormat);
            assert_eq!(res_2, MessageError::InvalidFormat);
        }
    }

    mod test_validate_text {
        use std::collections::vec_deque::VecDeque;

        use crate::irc::message::{utils::validate_text, MessageError};

        #[test]
        fn test_valid_text() {
            let mut text: VecDeque<&[u8]> = VecDeque::new();
            text.push_back(b":This");
            text.push_back(b"is");
            text.push_back(b":valid");
            text.push_back(b"text");

            let res = validate_text(text).unwrap();

            assert_eq!(res.len(), 4);
            assert_eq!(res[0], b"This");
            assert_eq!(res[1], b"is");
            assert_eq!(res[2], b":valid");
            assert_eq!(res[3], b"text");
        }

        #[test]
        fn test_invalid_text() {
            let mut text: VecDeque<&[u8]> = VecDeque::new();
            text.push_back(b"This");
            text.push_back(b"is");
            text.push_back(b":invalid");
            text.push_back(b"text");

            let res = validate_text(text).unwrap_err();

            assert_eq!(res, MessageError::InvalidFormat);
        }
    }

    mod test_validate_channel {
        use crate::irc::message::{utils::validate_channel, MessageError, BELL, HASH};

        #[test]
        fn test_validate_channels_valid_channels() {
            let channel1 = b"#twilight_zone";
            let channel2 = b"&group5";

            let res1 = validate_channel(Some(channel1)).unwrap();
            let res2 = validate_channel(Some(channel2)).unwrap();

            assert_eq!(res1, channel1);
            assert_eq!(res2, channel2);
        }

        #[test]
        fn test_invalid_channels() {
            let inv1 = b"jeje";
            let inv2 = b"$asdf";
            let inv3 = &[HASH, BELL, 48, 49];

            let res1 = validate_channel(Some(inv1)).unwrap_err();
            let res2 = validate_channel(Some(inv2)).unwrap_err();
            let res3 = validate_channel(Some(inv3)).unwrap_err();

            assert_eq!(res1, MessageError::InvalidFormat);
            assert_eq!(res2, MessageError::InvalidFormat);
            assert_eq!(res3, MessageError::InvalidFormat);
        }
    }
    mod test_split_csl_none {
        use crate::irc::message::utils::split_csl_none;

        #[test]
        fn test_split_csl_list_is_correct() {
            let list = b"comma,separated,list";

            let splitted = split_csl_none(Some(list)).unwrap();

            assert_eq!(splitted.len(), 3);

            assert_eq!(splitted[0], b"comma");
            assert_eq!(splitted[1], b"separated");
            assert_eq!(splitted[2], b"list");
        }

        #[test]
        fn test_split_csl_empties() {
            let list = b"comma,,separated,list";

            let splitted = split_csl_none(Some(list)).unwrap();

            assert_eq!(splitted.len(), 4);

            assert_eq!(splitted[0], b"comma");
            assert_eq!(splitted[1], b"");
            assert_eq!(splitted[2], b"separated");
            assert_eq!(splitted[3], b"list");
        }

        #[test]
        fn test_split_csl_empty() {
            let list = b"";

            let splitted = split_csl_none(Some(list)).unwrap();

            assert_eq!(splitted.len(), 1);

            assert_eq!(splitted[0], b"");
        }

        #[test]
        fn test_split_csl_none() {
            let splitted = split_csl_none(None);

            assert_eq!(splitted, None)
        }
    }
    mod test_valid_realname_valid_none {
        use crate::irc::message::utils::validate_realname_valid_none;
        use crate::irc::message::MessageError::InvalidFormat;

        #[test]
        fn test_valid_realname_without_space_is_valid() {
            let valid: &[u8] = b":valid";
            let valid_res: &[u8] = b"valid";

            assert_eq!(
                validate_realname_valid_none(Some(valid)),
                Ok(Some(valid_res))
            );
        }

        #[test]
        fn test_valid_realname_with_space_is_invalid() {
            let invalid: &[u8] = b":another valid";

            assert_eq!(
                validate_realname_valid_none(Some(invalid)),
                Err(InvalidFormat)
            );
        }

        #[test]
        fn test_valid_realname_without_colon_is_invalid() {
            let invalid: &[u8] = b"invalid";

            assert_eq!(
                validate_realname_valid_none(Some(invalid)),
                Err(InvalidFormat)
            );
        }

        #[test]
        fn test_valid_realname_none_is_valid() {
            assert_eq!(validate_realname_valid_none(None), Ok(None));
        }

        #[test]
        fn test_invalid_realname_without_ascii_characters() {
            let invalid_one: &[u8] = b":1invalid";
            let invalid_two: &[u8] = b":@invalid";
            let invalid_three: &[u8] = b":#invalid";

            assert_eq!(
                validate_realname_valid_none(Some(invalid_one)),
                Err(InvalidFormat)
            );
            assert_eq!(
                validate_realname_valid_none(Some(invalid_two)),
                Err(InvalidFormat)
            );
            assert_eq!(
                validate_realname_valid_none(Some(invalid_three)),
                Err(InvalidFormat)
            );
        }
    }
    mod test_validate_o_param {
        use crate::irc::message::utils::validate_o_param;
        use crate::irc::message::MessageError::InvalidFormat;
        #[test]
        fn test_valid_o() {
            let valid_one: &[u8] = b"o";

            assert!(validate_o_param(Some(valid_one)).unwrap());
        }
        #[test]
        fn test_invalid_o() {
            let valid_one: &[u8] = b"asd";

            assert_eq!(
                validate_o_param(Some(valid_one)).unwrap_err(),
                InvalidFormat
            );
        }
        #[test]
        fn test_none_o() {
            assert_eq!(validate_o_param(None).unwrap(), false);
        }
    }
}
