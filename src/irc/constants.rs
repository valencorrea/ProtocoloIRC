pub const ERR_NEEDMOREPARAMS: usize = 461;
pub const ERR_NONICKNAMEGIVEN: usize = 431;
pub const ERR_NICKNAMEINUSE: usize = 433;
pub const ERR_PASSWDMISMATCH: usize = 464;
pub const ERR_NORECIPIENT: usize = 411;
pub const ERR_NOSUCHNICK: usize = 401;
pub const ERR_NOTEXTTOSEND: usize = 412;
pub const ERR_NOSUCHCHANNEL: usize = 403;
pub const ERR_BADCHANNELKEY: usize = 475;
// pub const ERR_BADCHANMASK: usize = 476;
pub const ERR_NOTONCHANNEL: usize = 442;
pub const ERR_NOSUCHSERVER: usize = 402;
pub const ERR_USERONCHANNEL: usize = 443;
pub const ERR_INVITEONLYCHAN: usize = 473;
pub const ERR_CHANOPRIVSNEEDED: usize = 482;
pub const ERR_USERNOTINCHANNEL: usize = 441;
pub const ERR_CANNOTSENDTOCHAN: usize = 404;
pub const ERR_UNKNOWNMODE: usize = 472;
pub const ERR_SAMEUSER: usize = 1401;
pub const ERR_SERVERERR: usize = 999;
pub const ERR_CHANLIMIT: usize = 1403;
pub const RPL_PWDSET: usize = 1201;
pub const RPL_NICKSET: usize = 1202;
pub const RPL_REGISTERED: usize = 1203;
pub const ERR_REGMISSING: usize = 1405;
pub const ERR_ALREADYREGISTRED: usize = 462;
pub const RPL_YOUREOPER: usize = 381;
pub const RPL_LIST: usize = 322;
pub const RPL_LISTSTART: usize = 321;
pub const RPL_LISTEND: usize = 323;
pub const RPL_INVITING: usize = 341;
pub const RPL_WHOREPLY: usize = 352;
pub const RPL_ENDOFWHO: usize = 315;
pub const RPL_WHOISUSER: usize = 311;
pub const RPL_ENDOFWHOIS: usize = 318;
pub const RPL_WHOISOPERATOR: usize = 313;
pub const RPL_WHOISCHANNELS: usize = 319;
pub const RPL_TOPIC: usize = 332;
pub const RPL_NOTOPIC: usize = 331;
pub const RPL_UMODEIS: usize = 221;
pub const RPL_CHANNELMODEIS: usize = 324;
pub const RPL_AWAY: usize = 301;
pub const RPL_NOAWAY: usize = 305;
pub const RPL_UNAWAY: usize = 306;
pub const RPL_PART: usize = 307;

/*
 * Constants related to channel user information
 */
//JOINED <Channel> <User>
pub const RPL_NAMREPLY: usize = 353;
pub const RPL_ENDOFNAMES: usize = 366;
//OUT OF CHANNEL <Channel> <User>
pub const RPL_CHANNELOUT: usize = 1354;

/*
 * Constants realated to user information
 */
// NEW USER <User>
pub const RPL_NICKIN: usize = 2000;
// CHANGE NICK <Old> <New>
pub const RPL_NICKCHANGE: usize = 2001;
// USER OUT <User>
pub const RPL_NICKOUT: usize = 2010;

pub const RPL_SUCLOGIN: usize = 3000;

pub const INFO_PASSWORD: &str = "123";
pub const DEFAULT_SERVERNAME: &str = "Unknown";
