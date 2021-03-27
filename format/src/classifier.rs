use crate::entry::EntryMethod;
use list::*;

#[allow(dead_code)]
pub enum Arg<'a> {
    IsNumber,
    IsExact(Option<&'a String>),
    IsIn(&'a [&'static str]),
    Is(&'static str),
    Exists,
    Lacks,
    IsEmpty,
    IsNotEmpty,
    Anything,
    IsUrl,
    Or(&'a Arg<'a>, &'a Arg<'a>),
}

impl<'a> Arg<'a> {
    pub fn matches(&self, value: Option<&String>) -> Result<(), &'static str> {
        match self {
            Arg::IsNumber => {
                if let Some(value) = value {
                    if value.parse::<u64>().is_ok() {
                        Ok(())
                    } else {
                        Err("Expected Number, found String")
                    }
                } else {
                    Err("Expected Number, found Null")
                }
            }
            Arg::Is(expected_value) => {
                if value.map(|s| s.as_str()) == Some(expected_value) {
                    Ok(())
                } else {
                    Err("Expected a specific value, got something else")
                }
            }
            Arg::IsIn(values) => {
                for expected_value in values.iter() {
                    if Some(*expected_value) == value.map(|s| s.as_str()) {
                        return Ok(());
                    }
                }
                Err("Unexpected value")
            }
            Arg::Exists => {
                if value.is_some() {
                    Ok(())
                } else {
                    Err("Expected something, found Null")
                }
            }
            Arg::Lacks => {
                if value.is_none() {
                    Ok(())
                } else {
                    Err("Unexpected value")
                }
            }
            Arg::IsEmpty => {
                if value == Some(&"".to_string()) {
                    Ok(())
                } else {
                    Err("Expected an empty value, got something else")
                }
            }
            Arg::IsNotEmpty | Arg::IsUrl => {
                if let Some(value) = value {
                    if !value.is_empty() {
                        Ok(())
                    } else {
                        Err("Expected non-empty String")
                    }
                } else {
                    Err("Expected String, found Null")
                }
            }
            Arg::IsExact(expected_value) => {
                if &value == expected_value {
                    Ok(())
                } else {
                    Err("Expected a specific value, got something else")
                }
            }
            Arg::Anything => Ok(()),
            Arg::Or(c1, c2) => {
                if c2.matches(value).is_ok() {
                    Ok(())
                } else {
                    c1.matches(value)
                }
            }
        }
    }
}

pub struct Verifyer<'a> {
    entry_type: &'a str,
    workflow: Arg<'a>,
    template: Arg<'a>,
    method_type: Arg<'a>,
    configs: [Arg<'a>; 9],
}

impl<'a> Verifyer<'a> {
    pub const fn new(
        entry_type: &'a str,
        workflow: Arg<'a>,
        template: Arg<'a>,
        method_type: Arg<'a>,
        configs: [Arg<'a>; 9],
    ) -> Verifyer<'a> {
        Verifyer {
            entry_type,
            workflow,
            template,
            method_type,
            configs,
        }
    }

    pub fn matches(&self, entry: &EntryMethod) -> Result<(), (&'static str, &'static str)> {
        if self.entry_type != entry.entry_type {
            return Err(("entry_type", "does not match"))
        }
        self.workflow
            .matches(entry.workflow.as_ref())
            .map_err(|e| ("workflow", e))?;
        self.template
            .matches(Some(&entry.template))
            .map_err(|e| ("template", e))?;
        self.method_type
            .matches(entry.method_type.as_ref())
            .map_err(|e| ("method_type", e))?;
        self.configs[0]
            .matches(entry.config1.as_ref())
            .map_err(|e| ("1", e))?;
        self.configs[1]
            .matches(entry.config2.as_ref())
            .map_err(|e| ("2", e))?;
        self.configs[2]
            .matches(entry.config3.as_ref())
            .map_err(|e| ("3", e))?;
        self.configs[3]
            .matches(entry.config4.as_ref())
            .map_err(|e| ("4", e))?;
        self.configs[4]
            .matches(entry.config5.as_ref())
            .map_err(|e| ("5", e))?;
        self.configs[5]
            .matches(entry.config6.as_ref())
            .map_err(|e| ("6", e))?;
        self.configs[6]
            .matches(entry.config7.as_ref())
            .map_err(|e| ("7", e))?;
        self.configs[7]
            .matches(entry.config8.as_ref())
            .map_err(|e| ("8", e))?;
        self.configs[8]
            .matches(entry.config9.as_ref())
            .map_err(|e| ("9", e))?;
        Ok(())
    }
}

mod list {
    use super::Arg::*;
    use super::*;

    pub const INSTAGRAM_VISIT_PROFILE: Verifyer = Verifyer::new(
        "instagram_visit_profile",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            Or(&Lacks, &IsNumber),
            IsNotEmpty,
            Or(&Lacks, &IsNotEmpty),
            IsIn(&["Complete", "Delay"]),
            IsNumber,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );

    pub const INSTAGRAM_VISIT_PROFILE_WITH_QUESTION: Verifyer = Verifyer::new(
        "instagram_visit_profile",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            Or(&Lacks, &IsNumber),
            IsNotEmpty,
            Or(&Lacks, &IsNotEmpty),
            Is("Question"),
            IsNumber,
            IsNotEmpty,
            IsNotEmpty,
            IsEmpty,
        ],
    );
    
    pub const INSTAGRAM_VIEW_POST: Verifyer = Verifyer::new(
        "instagram_view_post",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            Lacks,
            Lacks,
            Or(&Lacks, &IsNumber),
            Or(&Lacks, &IsNumber),
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const INSTAGRAM_ENTER: Verifyer = Verifyer::new(
        "instagram_enter",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const CUSTOM_ACTION_QUESTION: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        Is("question"),
        Is("Ask a question"),
        [
            IsNotEmpty,
            Lacks,
            Or(&Lacks, &IsNotEmpty),
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Is("50"),
        ],
    );
    
    pub const CUSTOM_ACTION_ASK_QUESTION: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        IsEmpty,
        Is("Ask a question"),
        [
            IsNotEmpty,
            Lacks,
            IsNotEmpty,
            IsNotEmpty,
            Lacks,
            Lacks,
            Is("0"),
            Lacks,
            Lacks,
        ],
    );
    
    pub const CUSTOM_ACTION_VISIT_QUESTION: Verifyer = Verifyer::new(
        "custom_action",
        Is("VisitQuestion"),
        Is("visit"),
        Is("Allow question or tracking"),
        [
            IsNotEmpty,
            IsNotEmpty,
            IsNotEmpty,
            IsNotEmpty,
            Or(&IsNotEmpty, &Lacks),
            Is("simple"),
            Lacks,
            Or(&IsNotEmpty, &Lacks),
            Lacks,
        ],
    );

    pub const CUSTOM_ACTION_CHOOSE_OPTION: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        Is("choose_option"),
        Is("Use tracking"),
        [
            IsNotEmpty,
            Is("unique"),
            IsNotEmpty,
            Lacks,
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );
    
    pub const CUSTOM_ACTION_BLOG_COMMENT: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        Is("blog_comment"),
        Is("Allow question or tracking"),
        [
            IsNotEmpty,
            Is("comment"),
            Anything,
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );
    
    pub const CUSTOM_ACTION_BASIC: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        IsEmpty,
        Is("None"),
        [
            IsNotEmpty,
            Lacks,
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            IsNumber,
            Lacks,
            Lacks,
        ],
    );
    
    pub const CUSTOM_ACTION_VISIT_AUTO: Verifyer = Verifyer::new(
        "custom_action",
        Or(&IsEmpty, &Is("VisitAuto")),
        Is("visit"),
        Is("Use tracking"),
        [
            IsNotEmpty,
            IsNotEmpty,
            IsNotEmpty,
            Lacks,
            Lacks,
            Is("simple"),
            Lacks,
            Or(&IsNotEmpty, &Lacks),
            Lacks,
        ],
    );

    pub const CUSTOM_ACTION_VISIT_DELAY: Verifyer = Verifyer::new(
        "custom_action",
        Is("VisitDelay"),
        Is("visit"),
        Is("Use tracking"),
        [
            IsNotEmpty, // raw text to display
            IsNotEmpty, // seems to be an uid, ex: visit-394265639250
            IsNotEmpty, // html text to display
            Lacks,
            Lacks,
            Is("simple"),
            IsNumber, // seconds to wait
            IsNotEmpty, // json object representing a link (often useless)
            Lacks,
        ],
    );
    
    pub const CUSTOM_ACTION_BONUS: Verifyer = Verifyer::new(
        "custom_action",
        Lacks,
        Is("bonus"),
        Is("None"),
        [
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );
    
    pub const EMAIL_SUBSCRIBE: Verifyer = Verifyer::new(
        "email_subscribe",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Or(&IsNotEmpty, &Lacks),
            Lacks,
            Is("Off"),
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const FACEBOOK_ENTER: Verifyer = Verifyer::new(
        "facebook_enter",
        Lacks,
        IsEmpty,
        Lacks,
        [
            Or(&IsNotEmpty, &IsUrl),
            IsIn(&["Complete", "Like"]),
            Or(&Lacks, &IsUrl),
            Or(&Lacks, &IsNotEmpty),
            Or(&Lacks, &IsNumber),
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const FACEBOOK_VISIT_COMPLETE: Verifyer = Verifyer::new(
        "facebook_visit",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            IsNotEmpty,
            IsNumber,
            Is("Complete"),
            Is("Complete"),
            IsNumber,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const FACEBOOK_VISIT_LIKE: Verifyer = Verifyer::new(
        "facebook_visit",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            IsNotEmpty,
            IsNumber,
            Is("Like"),
            Is("Complete"),
            IsNumber,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const FACEBOOK_VISIT_LIKE_WITH_QUESTION: Verifyer = Verifyer::new(
        "facebook_visit",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            IsNotEmpty,
            IsNumber,
            Is("Like"),
            Is("Question"),
            IsNumber,
            IsNotEmpty,
            IsNotEmpty,
            IsEmpty,
        ],
    );
    
    pub const FACEBOOK_VIEW_POST: Verifyer = Verifyer::new(
        "facebook_view_post",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            IsNotEmpty,
            IsIn(&["post", "photo", "video"]),
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const PINTEREST_VISIT_COMPLETE: Verifyer = Verifyer::new(
        "pinterest_visit",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Is("Complete"),
            Is("Complete"),
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const PINTEREST_VISIT_FOLLOW: Verifyer = Verifyer::new(
        "pinterest_visit",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Is("Follow"),
            Is("Complete"),
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const YOUTUBE_VISIT_CHANNEL: Verifyer = Verifyer::new(
        "youtube_visit_channel",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Anything,
            Is("Complete"),
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );

    pub const YOUTUBE_VISIT_CHANNEL_WITH_DELAY: Verifyer = Verifyer::new(
        "youtube_visit_channel",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl, // url of the channel
            IsNotEmpty, // name of the channel
            Is("Delay"),
            IsNumber, // seconds to wait
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const YOUTUBE_VISIT_CHANNEL_WITH_QUESTION: Verifyer = Verifyer::new(
        "youtube_visit_channel",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl, // url of the channel
            IsNotEmpty, // username
            Is("Question"),
            IsNumber, // unknown number
            IsNotEmpty, // the question
            Or(&IsNotEmpty, &Lacks), // responses, optionnal
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const YOUTUBE_ENTER: Verifyer = Verifyer::new(
        "youtube_enter",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const TWITCHTV_FOLLOW: Verifyer = Verifyer::new(
        "twitchtv_follow",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );
    
    pub const TWITCHTV_ENTER: Verifyer = Verifyer::new(
        "twitchtv_enter",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );

    pub const TWITTER_ENTER: Verifyer = Verifyer::new(
        "twitter_enter",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const TWITTER_RETWEET: Verifyer = Verifyer::new(
        "twitter_retweet",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl,
            IsNotEmpty,
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const TWITTER_TWEET: Verifyer = Verifyer::new(
        "twitter_tweet",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const TWITTER_FOLLOW: Verifyer = Verifyer::new(
        "twitter_follow",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty,
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const DISCORD_JOIN_SERVER: Verifyer = Verifyer::new(
        "discord_join_server",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty, // text to display
            IsUrl, // invitation link
            IsNumber,
            IsNotEmpty, // server name
            IsNotEmpty, // unknown hex number
            IsNotEmpty, // channel name
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const LINKEDIN_FOLLOW: Verifyer = Verifyer::new(
        "linkedin_follow",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl, // profile url
            IsNotEmpty, // text to display
            IsNotEmpty, // user name
            IsNumber,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            IsEmpty,
        ],
    );

    pub const STEAM_JOIN_GROUP: Verifyer = Verifyer::new(
        "steam_join_group",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsUrl, // group url
            IsNotEmpty, // group name
            IsNotEmpty, // group name
            IsNumber,
            IsNumber, // 1
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const LOYALTY: Verifyer = Verifyer::new(
        "loyalty",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty, // text to display
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );

    pub const SHARE_ACTION: Verifyer = Verifyer::new(
        "share_action",
        Lacks,
        IsEmpty,
        Lacks,
        [
            IsNotEmpty, // text to display
            Lacks,
            IsNotEmpty, // more text to display
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
            Lacks,
        ],
    );
}

#[derive(Debug, PartialEq)]
pub enum EntryType {
    InstagramEnter,
    InstagramViewPost,
    InstagramVisitProfile,
    InstagramVisitProfileWithQuestion,
    CustomActionAskQuestion,
    CustomActionQuestion,
    CustomActionChooseOption,
    CustomActionVisitQuestion,
    CustomActionBlogComment,
    CustomActionBasic,
    CustomActionVisitAuto,
    CustomActionVisitDelay,
    CustomActionBonus,
    EmailSubscribe,
    FacebookEnter,
    FacebookVisitComplete,
    FacebookVisitLike,
    FacebookVisitWithQuestion,
    FacebookViewPost,
    PinterestVisitComplete,
    PinterestVisitFollow,
    TwitterEnter,
    TwitterRetweet,
    TwitterTweet,
    TwitterFollow,
    YoutubeVisitChannel,
    YoutubeVisitChannelWithDelay,
    YoutubeVisitChannelWithQuestion,
    YoutubeEnter,
    TwitchEnter,
    TwitchFollow,
    DiscordJoinServer,
    LinkedInFollow,
    SteamJoinGroup,
    Loyalty,
    ShareAction,
}

impl EntryType {
    pub fn get_request_type(&self) -> RequestType {
        use EntryType::*;
        use EntryType::{TwitterRetweet, TwitterTweet, TwitterFollow};
        use RequestType::*;
        use serde_json::Value::*;

        match self {
            InstagramEnter => Enter,
            InstagramViewPost => Simple(String("Done".to_string()), true, false),
            InstagramVisitProfile => Simple(String("V".to_string()), false, false),
            InstagramVisitProfileWithQuestion => Answer(",", 8),
            CustomActionAskQuestion => Answer(",", 5),
            CustomActionQuestion => Answer(",", 5),
            CustomActionVisitQuestion => Answer(",", 5),
            CustomActionChooseOption => Answer("\r\n", 5),
            CustomActionBlogComment => TextInput,
            CustomActionBasic => Simple(String("Done".to_string()), true, false),
            CustomActionVisitAuto => Simple(String("V".to_string()), false, false),
            CustomActionVisitDelay => SimpleWithDelay((String("V".to_string()), false, false), 7),
            CustomActionBonus => Simple(Null, false, false),
            EmailSubscribe => Simple(Null, false, false),
            FacebookEnter => Enter,
            FacebookVisitComplete => Simple(String("V".to_string()), false, false),
            FacebookVisitLike => Simple(String("V".to_string()), true, false),
            FacebookVisitWithQuestion => Answer(",", 6),
            FacebookViewPost => Simple(Null, false, false),
            PinterestVisitComplete => Simple(String("V".to_string()), false, false),
            PinterestVisitFollow => Simple(String("V".to_string()), true, false),
            TwitterEnter => Enter,
            TwitterRetweet => RequestType::TwitterRetweet,
            TwitterTweet => RequestType::TwitterTweet(true),
            TwitterFollow => RequestType::TwitterFollow,
            YoutubeVisitChannel => Simple(String("V".to_string()), false, false),
            YoutubeVisitChannelWithQuestion => Answer(",", 6),
            YoutubeVisitChannelWithDelay => SimpleWithDelay((String("V".to_string()), true, false), 4),
            YoutubeEnter => Enter,
            TwitchEnter => Enter,
            TwitchFollow => Simple(Null, false, false),
            LinkedInFollow => Simple(String("V".to_string()), false, false),
            DiscordJoinServer => Unimplemented("The bot does not support Discord yet."),
            SteamJoinGroup => Unimplemented("The bot does not support Steam groups auto-join yet."),
            Loyalty => Unimplemented("The bot does not support loyalty entries yet."),
            ShareAction => RequestType::TwitterTweet(false),
        }
    }
}

pub enum RequestType {
    Enter,
    TextInput,
    Answer(&'static str, u8),
    Simple(serde_json::Value, bool, bool),
    SimpleWithDelay((serde_json::Value, bool, bool), u8),
    Unimplemented(&'static str),
    TwitterRetweet,
    TwitterTweet(bool), // true when tweet data is included, false when we must share
    TwitterFollow,
}

#[allow(dead_code)]
pub fn classify(entry: &EntryMethod) -> Option<EntryType> {
    use EntryType::*;

    match entry {
        entry if PINTEREST_VISIT_COMPLETE.matches(&entry).is_ok() => Some(PinterestVisitComplete),
        entry if PINTEREST_VISIT_FOLLOW.matches(&entry).is_ok() => Some(PinterestVisitFollow),
        entry if INSTAGRAM_ENTER.matches(&entry).is_ok() => Some(InstagramEnter),
        entry if INSTAGRAM_VIEW_POST.matches(&entry).is_ok() => Some(InstagramViewPost),
        entry if INSTAGRAM_VISIT_PROFILE.matches(&entry).is_ok() => Some(InstagramVisitProfile),
        entry if INSTAGRAM_VISIT_PROFILE_WITH_QUESTION.matches(&entry).is_ok() => Some(InstagramVisitProfileWithQuestion),
        entry if CUSTOM_ACTION_QUESTION.matches(&entry).is_ok() => Some(CustomActionQuestion),
        entry if CUSTOM_ACTION_ASK_QUESTION.matches(&entry).is_ok() => Some(CustomActionAskQuestion),
        entry if CUSTOM_ACTION_VISIT_QUESTION.matches(&entry).is_ok() => Some(CustomActionVisitQuestion),
        entry if CUSTOM_ACTION_CHOOSE_OPTION.matches(&entry).is_ok() => Some(CustomActionChooseOption),
        entry if CUSTOM_ACTION_BLOG_COMMENT.matches(&entry).is_ok() => Some(CustomActionBlogComment),
        entry if CUSTOM_ACTION_BASIC.matches(&entry).is_ok() => Some(CustomActionBasic),
        entry if CUSTOM_ACTION_VISIT_AUTO.matches(&entry).is_ok() => Some(CustomActionVisitAuto),
        entry if CUSTOM_ACTION_VISIT_DELAY.matches(&entry).is_ok() => Some(CustomActionVisitDelay),
        entry if CUSTOM_ACTION_BONUS.matches(&entry).is_ok() => Some(CustomActionBonus),
        entry if EMAIL_SUBSCRIBE.matches(&entry).is_ok() => Some(EmailSubscribe),
        entry if FACEBOOK_ENTER.matches(&entry).is_ok() => Some(FacebookEnter),
        entry if FACEBOOK_VISIT_COMPLETE.matches(&entry).is_ok() => Some(FacebookVisitComplete),
        entry if FACEBOOK_VISIT_LIKE_WITH_QUESTION.matches(&entry).is_ok() => Some(FacebookVisitWithQuestion),
        entry if FACEBOOK_VISIT_LIKE.matches(&entry).is_ok() => Some(FacebookVisitLike),
        entry if FACEBOOK_VIEW_POST.matches(&entry).is_ok() => Some(FacebookViewPost),
        entry if TWITTER_ENTER.matches(&entry).is_ok() => Some(TwitterEnter),
        entry if TWITTER_RETWEET.matches(&entry).is_ok() => Some(TwitterRetweet),
        entry if TWITTER_TWEET.matches(&entry).is_ok() => Some(TwitterTweet),
        entry if TWITTER_FOLLOW.matches(&entry).is_ok() => Some(TwitterFollow),
        entry if YOUTUBE_VISIT_CHANNEL.matches(&entry).is_ok() => Some(YoutubeVisitChannel),
        entry if YOUTUBE_VISIT_CHANNEL_WITH_QUESTION.matches(&entry).is_ok() => Some(YoutubeVisitChannelWithQuestion),
        entry if YOUTUBE_VISIT_CHANNEL_WITH_DELAY.matches(&entry).is_ok() => Some(YoutubeVisitChannelWithDelay),
        entry if YOUTUBE_ENTER.matches(&entry).is_ok() => Some(YoutubeEnter),
        entry if TWITCHTV_ENTER.matches(&entry).is_ok() => Some(TwitchEnter),
        entry if TWITCHTV_FOLLOW.matches(&entry).is_ok() => Some(TwitchFollow),
        entry if DISCORD_JOIN_SERVER.matches(&entry).is_ok() => Some(DiscordJoinServer),
        entry if LINKEDIN_FOLLOW.matches(&entry).is_ok() => Some(LinkedInFollow),
        entry if STEAM_JOIN_GROUP.matches(&entry).is_ok() => Some(SteamJoinGroup),
        entry if LOYALTY.matches(&entry).is_ok() => Some(Loyalty),
        entry if SHARE_ACTION.matches(&entry).is_ok() => Some(ShareAction),
        _ => None
    }
}