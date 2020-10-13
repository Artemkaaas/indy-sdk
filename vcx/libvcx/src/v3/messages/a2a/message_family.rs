use settings::Actors;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumIter)]
pub enum MessageFamilies {
    Routing,
    Connections,
    Notification,
    Signature,
    CredentialIssuance,
    ReportProblem,
    PresentProof,
    TrustPing,
    DiscoveryFeatures,
    Basicmessage,
    Outofband,
    QuestionAnswer,
    Committedanswer,
    InviteAction,
    Unknown(String)
}

impl MessageFamilies {
    pub const DID: &'static str = "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec";
    pub const ENDPOINT: &'static str = "https://didcomm.org";

    pub fn version(&self) -> &'static str {
        match self {
            MessageFamilies::Routing => "1.0",
            MessageFamilies::Connections => "1.0",
            MessageFamilies::Notification => "1.0",
            MessageFamilies::Signature => "1.0",
            MessageFamilies::CredentialIssuance => "1.0",
            MessageFamilies::ReportProblem => "1.0",
            MessageFamilies::PresentProof => "1.0",
            MessageFamilies::TrustPing => "1.0",
            MessageFamilies::DiscoveryFeatures => "1.0",
            MessageFamilies::Basicmessage => "1.0",
            MessageFamilies::Outofband => "1.0",
            MessageFamilies::QuestionAnswer => "1.0",
            MessageFamilies::Committedanswer => "1.0",
            MessageFamilies::InviteAction => "0.9",
            MessageFamilies::Unknown(_) => "1.0"
        }
    }

    pub fn id(&self) -> String {
        match self {
            MessageFamilies::Routing |
            MessageFamilies::Connections |
            MessageFamilies::Notification |
            MessageFamilies::Signature |
            MessageFamilies::CredentialIssuance |
            MessageFamilies::ReportProblem |
            MessageFamilies::PresentProof |
            MessageFamilies::TrustPing |
            MessageFamilies::DiscoveryFeatures |
            MessageFamilies::Basicmessage |
            MessageFamilies::QuestionAnswer |
            MessageFamilies::Committedanswer |
            MessageFamilies::Unknown(_) => format!("{}/{}/{}", Self::DID, self.to_string(), self.version().to_string()),
            MessageFamilies::Outofband |
            MessageFamilies::InviteAction => format!("{}/{}/{}", Self::ENDPOINT, self.to_string(), self.version().to_string()),
        }
    }

    pub fn actors(&self) -> Option<(Option<Actors>, Option<Actors>)> {
        match self {
            MessageFamilies::Routing => None,
            MessageFamilies::Connections => Some((Some(Actors::Inviter), Some(Actors::Invitee))),
            MessageFamilies::Notification => None,
            MessageFamilies::Signature => None,
            MessageFamilies::CredentialIssuance => Some((Some(Actors::Issuer), Some(Actors::Holder))),
            MessageFamilies::ReportProblem => None,
            MessageFamilies::PresentProof => Some((Some(Actors::Prover), Some(Actors::Verifier))),
            MessageFamilies::TrustPing => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageFamilies::DiscoveryFeatures => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageFamilies::Basicmessage => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageFamilies::Outofband => Some((None, Some(Actors::Receiver))),
            MessageFamilies::QuestionAnswer => Some((None, Some(Actors::Receiver))),
            MessageFamilies::Committedanswer => Some((None, Some(Actors::Receiver))),
            MessageFamilies::InviteAction => Some((Some(Actors::Inviter), Some(Actors::Invitee))),
            MessageFamilies::Unknown(_) => None
        }
    }
}

impl From<String> for MessageFamilies {
    fn from(family: String) -> Self {
        match family.as_str() {
            "routing" => MessageFamilies::Routing,
            "connections" => MessageFamilies::Connections,
            "signature" => MessageFamilies::Signature,
            "notification" => MessageFamilies::Notification,
            "issue-credential" => MessageFamilies::CredentialIssuance,
            "report-problem" => MessageFamilies::ReportProblem,
            "present-proof" => MessageFamilies::PresentProof,
            "trust_ping" => MessageFamilies::TrustPing,
            "discover-features" => MessageFamilies::DiscoveryFeatures,
            "basicmessage" => MessageFamilies::Basicmessage,
            "out-of-band" => MessageFamilies::Outofband,
            "questionanswer" => MessageFamilies::QuestionAnswer,
            "committedanswer" => MessageFamilies::Committedanswer,
            "invite-action" => MessageFamilies::InviteAction,
            family @ _ => MessageFamilies::Unknown(family.to_string())
        }
    }
}

impl ::std::string::ToString for MessageFamilies {
    fn to_string(&self) -> String {
        match self {
            MessageFamilies::Routing => "routing".to_string(),
            MessageFamilies::Connections => "connections".to_string(),
            MessageFamilies::Notification => "notification".to_string(),
            MessageFamilies::Signature => "signature".to_string(),
            MessageFamilies::CredentialIssuance => "issue-credential".to_string(),
            MessageFamilies::ReportProblem => "report-problem".to_string(),
            MessageFamilies::PresentProof => "present-proof".to_string(),
            MessageFamilies::TrustPing => "trust_ping".to_string(),
            MessageFamilies::DiscoveryFeatures => "discover-features".to_string(),
            MessageFamilies::Basicmessage => "basicmessage".to_string(),
            MessageFamilies::Outofband => "out-of-band".to_string(),
            MessageFamilies::QuestionAnswer => "questionanswer".to_string(),
            MessageFamilies::Committedanswer => "committedanswer".to_string(),
            MessageFamilies::InviteAction => "invite-action".to_string(),
            MessageFamilies::Unknown(family) => family.to_string()
        }
    }
}

impl Default for MessageFamilies {
    fn default() -> MessageFamilies {
        MessageFamilies::Unknown(String::new())
    }
}
