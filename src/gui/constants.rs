type GtkID = &'static str;
type Example = &'static str;

const PASS_COMMAND: (GtkID, Example) = ("pass_msg", "PASS secretPassword");
const NICK_COMMAND: (GtkID, Example) = ("nick_msg", ":Caro NICK Carola99");
const USER_COMMAND: (GtkID, Example) = ("user_msg", "USER Andra tolmoon tolsun :Agustin Andrade");
const OPER_COMMAND: (GtkID, Example) = ("oper_msg", "OPER Tomi secretPassword");
const QUIT_COMMAND: (GtkID, Example) = ("quit_msg", "QUIT :Gone to have lunch");
const PRIVMSG_COMMAND: (GtkID, Example) =
    ("priv_msg", "PRIVMSG Valen :Do you wanna go for a walk?");
const NOTICE_COMMAND: (GtkID, Example) = ("notice_msg", "NOTICE Tomi :What do you want for lunch?");
const JOIN_COMMAND: (GtkID, Example) = ("join_msg", "JOIN #impl-displ-for-group,#git");
const PART_COMMAND: (GtkID, Example) = ("part_msg", "PART #avisos");
const NAMES_COMMAND: (GtkID, Example) = ("names_msg", "NAMES #rust,#general");
const LIST_COMMAND: (GtkID, Example) = ("list_msg", "LIST #impl-displ-for-group,#git");
const INVITE_COMMAND: (GtkID, Example) = ("invite_msg", "INVITE Tomi #rust");
const WHO_COMMAND: (GtkID, Example) = ("who_msg", "WHO *.tina");
const WHOIS_COMMAND: (GtkID, Example) = ("whois_msg", "WHOIS Valen");
const MODE_COMMAND: (GtkID, Example) = ("mode_msg", "MODE #avisos +im");
const TOPIC_COMMAND: (GtkID, Example) = ("topic_msg", "TOPIC #music :trap");
const KICK_COMMAND: (GtkID, Example) = ("kick_msg", "KICK #jokes Tomi :boring jokes");
const AWAY_COMMAND: (GtkID, Example) = ("away_msg", "AWAY :Gone to go for a beer.");

pub const IRC_WELCOME: &str = "Welcome to IRC :)";
// pub const EMPTY_CHANNEL: &str = " +tn -psimlk";

pub const COMMANDS: [(GtkID, Example); 18] = [
    PASS_COMMAND,
    NICK_COMMAND,
    USER_COMMAND,
    OPER_COMMAND,
    QUIT_COMMAND,
    PRIVMSG_COMMAND,
    NOTICE_COMMAND,
    JOIN_COMMAND,
    PART_COMMAND,
    NAMES_COMMAND,
    LIST_COMMAND,
    INVITE_COMMAND,
    WHO_COMMAND,
    WHOIS_COMMAND,
    MODE_COMMAND,
    TOPIC_COMMAND,
    KICK_COMMAND,
    AWAY_COMMAND,
];
