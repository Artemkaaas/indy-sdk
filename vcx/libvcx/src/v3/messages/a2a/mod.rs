pub mod message_family;
pub mod message_type;
pub mod protocol_registry;

use self::message_type::MessageType;
use self::message_family::MessageFamilies;

use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;

use v3::messages::connection::invite::Invitation;
use v3::messages::connection::request::Request;
use v3::messages::connection::response::SignedResponse;
use v3::messages::connection::problem_report::ProblemReport as ConnectionProblemReport;
use v3::messages::trust_ping::ping::Ping;
use v3::messages::trust_ping::ping_response::PingResponse;
use v3::messages::forward::Forward;
use v3::messages::error::ProblemReport as CommonProblemReport;
use v3::messages::issuance::credential_proposal::CredentialProposal;
use v3::messages::ack::Ack;
use v3::messages::outofband::invitation::Invitation as OutofbandInvitation;
use v3::messages::outofband::handshake_reuse::HandshakeReuse;
use v3::messages::outofband::handshake_reuse_accepted::HandshakeReuseAccepted;

use v3::messages::issuance::credential_offer::CredentialOffer;
use v3::messages::issuance::credential_request::CredentialRequest;
use v3::messages::issuance::credential::Credential;

use v3::messages::proof_presentation::presentation_proposal::PresentationProposal;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::proof_presentation::presentation::Presentation;

use v3::messages::discovery::query::Query;
use v3::messages::discovery::disclose::Disclose;

use v3::messages::basic_message::message::BasicMessage;

use v3::messages::questionanswer::question::Question;
use v3::messages::questionanswer::answer::Answer;

use v3::messages::committedanswer::question::Question as CommitedQuestion;
use v3::messages::committedanswer::answer::Answer as CommitedAnswer;

use v3::messages::invite_action::invite::Invite as InviteForAction;

#[derive(Debug, PartialEq)]
pub enum A2AMessage {
    /// routing
    Forward(Forward),

    /// DID Exchange
    ConnectionInvitation(Invitation),
    ConnectionRequest(Request),
    ConnectionResponse(SignedResponse),
    ConnectionProblemReport(ConnectionProblemReport),

    /// trust ping
    Ping(Ping),
    PingResponse(PingResponse),

    /// notification
    Ack(Ack),
    CommonProblemReport(CommonProblemReport),

    /// credential issuance
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(Ack),
    CredentialReject(CommonProblemReport),

    /// proof presentation
    PresentationProposal(PresentationProposal),
    PresentationRequest(PresentationRequest),
    Presentation(Presentation),
    PresentationAck(Ack),
    PresentationReject(CommonProblemReport),

    /// discovery features
    Query(Query),
    Disclose(Disclose),

    /// basic message
    BasicMessage(BasicMessage),

    /// questionanswer
    Question(Question),
    Answer(Answer),

    /// committedanswer
    CommittedQuestion(CommitedQuestion),
    CommittedAnswer(CommitedAnswer),

    /// Out-of-Band
    OutOfBandInvitation(OutofbandInvitation),
    HandshakeReuse(HandshakeReuse),
    HandshakeReuseAccepted(HandshakeReuseAccepted),

    // invite-action
    InviteForAction(InviteForAction),

    /// Any Raw Message
    Generic(Value),
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        trace!("deserializing aries a2a message");

        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        let message_type: MessageType = match serde_json::from_value(value["@type"].clone()) {
            Ok(message_type) => message_type,
            Err(_) => return Ok(A2AMessage::Generic(value))
        };

        match (message_type.family, message_type.type_.as_str()) {
            (MessageFamilies::Routing, A2AMessage::FORWARD) => {
                Forward::deserialize(value)
                    .map(|msg| A2AMessage::Forward(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION) => {
                Invitation::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionInvitation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST) => {
                Request::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE) => {
                SignedResponse::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::TrustPing, A2AMessage::PING) => {
                Ping::deserialize(value)
                    .map(|msg| A2AMessage::Ping(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE) => {
                PingResponse::deserialize(value)
                    .map(|msg| A2AMessage::PingResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT) => {
                ConnectionProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Notification, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::Ack(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::CommonProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL) => {
                Credential::deserialize(value)
                    .map(|msg| A2AMessage::Credential(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL) => {
                CredentialProposal::deserialize(value)
                    .map(|msg| A2AMessage::CredentialProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER) => {
                CredentialOffer::deserialize(value)
                    .map(|msg| A2AMessage::CredentialOffer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL) => {
                CredentialRequest::deserialize(value)
                    .map(|msg| A2AMessage::CredentialRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::CredentialAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::CredentialReject(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION) => {
                PresentationProposal::deserialize(value)
                    .map(|msg| A2AMessage::PresentationProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION) => {
                PresentationRequest::deserialize(value)
                    .map(|msg| A2AMessage::PresentationRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PRESENTATION) => {
                Presentation::deserialize(value)
                    .map(|msg| A2AMessage::Presentation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::PresentationAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::PresentationReject(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY) => {
                Query::deserialize(value)
                    .map(|msg| A2AMessage::Query(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE) => {
                Disclose::deserialize(value)
                    .map(|msg| A2AMessage::Disclose(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE) => {
                BasicMessage::deserialize(value)
                    .map(|msg| A2AMessage::BasicMessage(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::QuestionAnswer, A2AMessage::QUESTION) => {
                Question::deserialize(value)
                    .map(|msg| A2AMessage::Question(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::QuestionAnswer, A2AMessage::ANSWER) => {
                Answer::deserialize(value)
                    .map(|msg| A2AMessage::Answer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Committedanswer, A2AMessage::QUESTION) => {
                CommitedQuestion::deserialize(value)
                    .map(|msg| A2AMessage::CommittedQuestion(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Committedanswer, A2AMessage::ANSWER) => {
                CommitedAnswer::deserialize(value)
                    .map(|msg| A2AMessage::CommittedAnswer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Outofband, A2AMessage::OUTOFBAND_INVITATION) => {
                OutofbandInvitation::deserialize(value)
                    .map(|msg| A2AMessage::OutOfBandInvitation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE) => {
                HandshakeReuse::deserialize(value)
                    .map(|msg| A2AMessage::HandshakeReuse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED) => {
                HandshakeReuseAccepted::deserialize(value)
                    .map(|msg| A2AMessage::HandshakeReuseAccepted(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::InviteAction, A2AMessage::INVITE_FOR_ACTION) => {
                InviteForAction::deserialize(value)
                    .map(|msg| A2AMessage::InviteForAction(msg))
                    .map_err(de::Error::custom)
            }
            (_, _) => {
                warn!("Unexpected @type field: {}", value["@type"]);
                Ok(A2AMessage::Generic(value))
            }
        }
    }
}

fn set_a2a_message_type_with_did<T>(msg: T, family: MessageFamilies, name: &str) -> Result<serde_json::Value, serde_json::Error> where T: Serialize {
    let mut value = ::serde_json::to_value(msg)?;
    let type_ = ::serde_json::to_value(MessageType::build_with_did(family, name))?;
    value.as_object_mut().unwrap().insert("@type".into(), type_);
    Ok(value)
}

fn set_a2a_message_type_with_endpoint<T>(msg: T, family: MessageFamilies, name: &str) -> Result<serde_json::Value, serde_json::Error> where T: Serialize {
    let mut value = ::serde_json::to_value(msg)?;
    let type_ = ::serde_json::to_value(MessageType::build_with_endpoint(family, name))?;
    value.as_object_mut().unwrap().insert("@type".into(), type_);
    Ok(value)
}

impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        trace!("serializing aries a2a message");

        let value = match self {
            A2AMessage::Forward(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Routing, A2AMessage::FORWARD),
            A2AMessage::ConnectionInvitation(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION),
            A2AMessage::ConnectionRequest(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST),
            A2AMessage::ConnectionResponse(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE),
            A2AMessage::ConnectionProblemReport(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT),
            A2AMessage::Ping(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::TrustPing, A2AMessage::PING),
            A2AMessage::PingResponse(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE),
            A2AMessage::Ack(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Notification, A2AMessage::ACK),
            A2AMessage::CommonProblemReport(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT),
            A2AMessage::CredentialOffer(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER),
            A2AMessage::Credential(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL),
            A2AMessage::CredentialProposal(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL),
            A2AMessage::CredentialRequest(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL),
            A2AMessage::CredentialAck(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::ACK),
            A2AMessage::CredentialReject(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::CredentialIssuance, A2AMessage::PROBLEM_REPORT),
            A2AMessage::PresentationProposal(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION),
            A2AMessage::PresentationRequest(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION),
            A2AMessage::Presentation(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::PresentProof, A2AMessage::PRESENTATION),
            A2AMessage::PresentationAck(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::PresentProof, A2AMessage::ACK),
            A2AMessage::PresentationReject(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::PresentProof, A2AMessage::PROBLEM_REPORT),
            A2AMessage::Query(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY),
            A2AMessage::Disclose(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE),
            A2AMessage::BasicMessage(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE),
            A2AMessage::OutOfBandInvitation(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Outofband, A2AMessage::OUTOFBAND_INVITATION),
            A2AMessage::HandshakeReuse(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE),
            A2AMessage::HandshakeReuseAccepted(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED),
            A2AMessage::Question(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::QuestionAnswer, A2AMessage::QUESTION),
            A2AMessage::Answer(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::QuestionAnswer, A2AMessage::ANSWER),
            A2AMessage::CommittedQuestion(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Committedanswer, A2AMessage::QUESTION),
            A2AMessage::CommittedAnswer(msg) => set_a2a_message_type_with_did(msg, MessageFamilies::Committedanswer, A2AMessage::ANSWER),
            A2AMessage::InviteForAction(msg) => set_a2a_message_type_with_endpoint(msg, MessageFamilies::InviteAction, A2AMessage::INVITE_FOR_ACTION),
            A2AMessage::Generic(msg) => Ok(msg.clone())
        }.map_err(ser::Error::custom)?;

        value.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    #[cfg(test)]
    pub fn id() -> MessageId {
        MessageId(String::from("testid"))
    }

    pub fn new() -> MessageId {
        MessageId::default()
    }
}

impl ToString for MessageId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for MessageId {
    #[cfg(all(test, not(feature = "aries")))]
    fn default() -> MessageId {
        MessageId::id()
    }

    #[cfg(any(not(test), feature = "aries"))]
    fn default() -> MessageId {
        use utils::uuid;
        MessageId(uuid::uuid())
    }
}

impl A2AMessage {
    pub const FORWARD: &'static str = "forward";
    pub const CONNECTION_INVITATION: &'static str = "invitation";
    pub const CONNECTION_REQUEST: &'static str = "request";
    pub const CONNECTION_RESPONSE: &'static str = "response";
    pub const CONNECTION_PROBLEM_REPORT: &'static str = "problem_report";
    pub const PING: &'static str = "ping";
    pub const PING_RESPONSE: &'static str = "ping_response";
    pub const ACK: &'static str = "ack";
    pub const PROBLEM_REPORT: &'static str = "problem-report";
    pub const CREDENTIAL_OFFER: &'static str = "offer-credential";
    pub const CREDENTIAL: &'static str = "issue-credential";
    pub const PROPOSE_CREDENTIAL: &'static str = "propose-credential";
    pub const REQUEST_CREDENTIAL: &'static str = "request-credential";
    pub const PROPOSE_PRESENTATION: &'static str = "propose-presentation";
    pub const REQUEST_PRESENTATION: &'static str = "request-presentation";
    pub const PRESENTATION: &'static str = "presentation";
    pub const QUERY: &'static str = "query";
    pub const DISCLOSE: &'static str = "disclose";
    pub const BASIC_MESSAGE: &'static str = "message";
    pub const OUTOFBAND_INVITATION: &'static str = "invitation";
    pub const OUTOFBAND_HANDSHAKE_REUSE: &'static str = "handshake-reuse";
    pub const OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED: &'static str = "handshake-reuse-accepted";
    pub const QUESTION: &'static str = "question";
    pub const ANSWER: &'static str = "answer";
    pub const INVITE_FOR_ACTION: &'static str = "invite";
}

#[macro_export]
macro_rules! a2a_message {
    ($type:ident) => (
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$type(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    );

    ($type:ident, $a2a_message_kind:ident) => (
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$a2a_message_kind(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    );
}
