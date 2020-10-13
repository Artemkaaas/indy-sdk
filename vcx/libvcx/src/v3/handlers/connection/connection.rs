use error::prelude::*;
use std::collections::HashMap;

use v3::handlers::connection::states::{DidExchangeSM, Actor, ActorDidExchangeState};
use v3::handlers::connection::messages::DidExchangeMessages;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::a2a::A2AMessage;
use v3::messages::connection::invite::Invitation;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::basic_message::message::BasicMessage;
use v3::handlers::connection::types::{SideConnectionInfo, PairwiseConnectionInfo, CompletedConnection, OutofbandMeta, Invitations};
use v3::messages::outofband::invitation::Invitation as OutofbandInvitation;
use v3::messages::questionanswer::question::{Question, QuestionResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    connection_sm: DidExchangeSM
}

impl Connection {
    pub fn create(source_id: &str) -> Connection {
        trace!("Connection::create >>> source_id: {}", source_id);
        debug!("Connection {}: Creating Connection state object", source_id);

        Connection {
            connection_sm: DidExchangeSM::new(Actor::Inviter, source_id, None),
        }
    }

    pub fn create_outofband(source_id: &str, goal_code: Option<String>, goal: Option<String>,
                            handshake: bool, request_attach: Option<String>) -> Connection {
        trace!("create_outofband_connection >>> source_id: {}, goal_code: {:?}, goal: {:?}, handshake: {}, request_attach: {:?}",
               source_id, secret!(goal_code), secret!(goal), secret!(handshake), secret!(request_attach));
        debug!("Connection {}: Creating out-of-band Connection state object", source_id);

        let meta = OutofbandMeta::new(goal_code, goal, handshake, request_attach);

        Connection {
            connection_sm: DidExchangeSM::new(Actor::Inviter, source_id, Some(meta)),
        }
    }

    pub fn from_parts(source_id: String, agent_info: AgentInfo, state: ActorDidExchangeState) -> Connection {
        Connection { connection_sm: DidExchangeSM::from(source_id, agent_info, state) }
    }

    pub fn create_with_invite(source_id: &str, invitation: Invitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_invite >>> source_id: {}, invitation: {:?}", source_id, secret!(invitation));
        debug!("Connection {}: Creating Connection state object with invite", source_id);

        let mut connection = Connection {
            connection_sm: DidExchangeSM::new(Actor::Invitee, source_id, None),
        };

        connection.process_invite(invitation)?;

        Ok(connection)
    }

    pub fn create_with_outofband_invite(source_id: &str, invitation: OutofbandInvitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_outofband_invite >>> source_id: {}, invitation: {:?}", source_id, secret!(invitation));
        debug!("Connection {}: Creating Connection state object with out-of-band invite", source_id);

        invitation.validate()?;

        let mut connection = Connection {
            connection_sm: DidExchangeSM::new(Actor::Invitee, source_id, None),
        };

        connection.process_outofband_invite(invitation)?;

        Ok(connection)
    }

    pub fn source_id(&self) -> String { self.connection_sm.source_id().to_string() }

    pub fn state(&self) -> u32 { self.connection_sm.state() }

    pub fn agent_info(&self) -> &AgentInfo { self.connection_sm.agent_info() }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.connection_sm.remote_did()
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.connection_sm.remote_vk()
    }

    pub fn state_object<'a>(&'a self) -> &'a ActorDidExchangeState {
        &self.connection_sm.state_object()
    }

    pub fn get_source_id(&self) -> String {
        self.connection_sm.source_id().to_string()
    }

    pub fn process_invite(&mut self, invitation: Invitation) -> VcxResult<()> {
        trace!("Connection::process_invite >>> invitation: {:?}", secret!(invitation));
        self.step(DidExchangeMessages::InvitationReceived(invitation))
    }

    pub fn process_outofband_invite(&mut self, invitation: OutofbandInvitation) -> VcxResult<()> {
        trace!("Connection::process_outofband_invite >>> invitation: {:?}", secret!(invitation));
        self.step(DidExchangeMessages::OutofbandInvitationReceived(invitation))
    }

    pub fn get_invitation(&self) -> Option<Invitations> {
        trace!("Connection::get_invite >>>");
        return self.connection_sm.get_invitation()
    }

    pub fn get_invite_details(&self) -> VcxResult<String> {
        trace!("Connection::get_invite_details >>>");
        debug!("Connection {}: Getting invitation", self.source_id());

        let invitation = match self.get_invitation() {
            Some(invitation) => match invitation {
                Invitations::ConnectionInvitation(invitation_) => {
                    json!(invitation_.to_a2a_message()).to_string()
                },
                Invitations::OutofbandInvitation(invitation_) => {
                    json!(invitation_.to_a2a_message()).to_string()
                }
            },
            None => json!({}).to_string()
        };

        return Ok(invitation)
    }

    pub fn connect(&mut self) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.connection_sm.source_id());
        debug!("Connection {}: Starting connection establishing process", self.source_id());

        self.step(DidExchangeMessages::Connect())
    }

    pub fn update_state(&mut self, message: Option<&str>) -> VcxResult<u32> {
        trace!("Connection::update_state >>> message: {:?}", secret!(message));
        debug!("Connection {}: Updating state", self.source_id());

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let messages = self.get_messages()?;

        if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            self.agent_info().update_message_status(uid)?;
        } else {
            if let Some(prev_agent_info) = self.connection_sm.prev_agent_info().cloned() {
                let messages = prev_agent_info.get_messages()?;

                if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
                    self.handle_message(message.into())?;
                    prev_agent_info.update_message_status(uid)?;
                }
            }
        };

        let state = self.state();

        trace!("Connection::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        debug!("Connection {}: Updating message status as reviewed", self.source_id());

        self.connection_sm.agent_info().update_message_status(uid)
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<u32> {
        trace!("Connection: update_state_with_message: {}", secret!(message));
        debug!("Connection {}: Updating state with message", self.source_id());

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Cannot updated Connection state with messages: Message deserialization failed with: {:?}", err)))?;

        self.handle_message(message.into())?;

        let state = self.state();

        trace!("Connection: update_state_with_message: <<< state: {}", state);

        Ok(state)
    }

    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Connection: get_messages >>>");
        debug!("Connection {}: Getting messages", self.source_id());
        self.agent_info().get_messages()
    }

    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>>");
        debug!("Connection {}: Getting message by id {:?}", self.source_id(), msg_id);

        self.agent_info().get_message_by_id(msg_id)
    }

    pub fn handle_message(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        trace!("Connection: handle_message >>> {:?}",  secret!(message));
        self.step(message)
    }

    pub fn send_message(&self, message: &A2AMessage) -> VcxResult<()> {
        trace!("Connection::send_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Sending message", self.source_id());

        let did_doc = self.connection_sm.did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection DIDDoc is not set"))?;

        self.agent_info().send_message(message, &did_doc)
    }

    pub fn send_message_to_self_endpoint(message: &A2AMessage, did_doc: &DidDoc) -> VcxResult<()> {
        trace!("Connection::send_message_to_self_endpoint >>> message: {:?}, did_doc: {:?}", secret!(message), secret!(did_doc));

        AgentInfo::send_message_anonymously(message, did_doc)
    }

    fn parse_generic_message(message: &str, _message_options: &str) -> A2AMessage {
        match ::serde_json::from_str::<A2AMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => {
                BasicMessage::create()
                    .set_content(message.to_string())
                    .set_time()
                    .to_a2a_message()
            }
        }
    }

    pub fn send_generic_message(&self, message: &str, _message_options: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Sending generic message", self.source_id());

        let message = Connection::parse_generic_message(message, _message_options);
        self.send_message(&message).map(|_| String::new())
    }

    pub fn send_ping(&mut self, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", secret!(comment));
        debug!("Connection {}: Sending ping message", self.source_id());

        self.handle_message(DidExchangeMessages::SendPing(comment))
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.connection_sm.source_id());
        self.agent_info().delete()
    }

    fn step(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        self.connection_sm = self.connection_sm.clone().step(message)?;
        Ok(())
    }

    pub fn send_discovery_features(&mut self, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", secret!(query), secret!(comment));
        debug!("Connection {}: Sending discovery features message", self.source_id());

        self.handle_message(DidExchangeMessages::DiscoverFeatures((query, comment)))
    }

    pub fn send_reuse(&mut self, invitation: OutofbandInvitation) -> VcxResult<()> {
        trace!("Connection::send_reuse >>> invitation: {:?}", secret!(invitation));
        debug!("Connection {}: Sending reuse message", self.source_id());

        self.handle_message(DidExchangeMessages::SendHandshakeReuse(invitation))
    }

    pub fn send_answer(&mut self, question: String, response: String) -> VcxResult<()> {
        trace!("Connection::send_answer >>> question: {:?}, response: {:?}", secret!(question), secret!(response));
        debug!("Connection {}: Sending question answer message", self.source_id());

        let question: Question = ::serde_json::from_str(&question)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Could not parse Aries Question from message: {:?}. Err: {:?}", question, err)))?;

        let response: QuestionResponse = ::serde_json::from_str(&response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Could not parse Aries Valid Question Response from message: {:?}. Err: {:?}", response, err)))?;

        self.handle_message(DidExchangeMessages::SendAnswer((question, response)))
    }

    pub fn send_invite_action(&mut self, goal_code: String) -> VcxResult<()> {
        trace!("Connection::send_invite_action >>> goal_code: {:?}", secret!(goal_code));
        debug!("Connection {}: Sending invitation for taking an action", self.source_id());

        self.handle_message(DidExchangeMessages::SendInviteAction(goal_code))
    }

    pub fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");
        debug!("Connection {}: Getting information", self.source_id());

        let agent_info = self.agent_info().clone();

        let current = SideConnectionInfo {
            did: agent_info.pw_did.clone(),
            recipient_keys: agent_info.recipient_keys().clone(),
            routing_keys: agent_info.routing_keys()?,
            service_endpoint: agent_info.agency_endpoint()?,
            protocols: Some(self.connection_sm.get_protocols()),
        };

        let remote = match self.connection_sm.did_doc() {
            Some(did_doc) =>
                Some(SideConnectionInfo {
                    did: did_doc.id.clone(),
                    recipient_keys: did_doc.recipient_keys(),
                    routing_keys: did_doc.routing_keys(),
                    service_endpoint: did_doc.get_endpoint(),
                    protocols: self.connection_sm.get_remote_protocols(),
                }),
            None => None
        };

        let connection_info = PairwiseConnectionInfo {
            my: current,
            their: remote,
            invitation: self.get_invitation(),
        };

        return Ok(json!(connection_info).to_string());
    }

    pub fn get_completed_connection(&self) -> VcxResult<CompletedConnection> {
        self.connection_sm.completed_connection()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                      format!("Connection object {} in state {} not ready to send remote messages", self.connection_sm.source_id(), self.state())))
    }
}

#[cfg(test)]
mod tests {
    use v3::messages::a2a::A2AMessage;
    use v3::handlers::connection::connection::Connection;

    #[test]
    fn test_parse_generic_message_plain_string_should_be_parsed_as_basic_msg() -> Result<(), String> {
        let message = "Some plain text message";
        let result = Connection::parse_generic_message(message, "");
        match result {
            A2AMessage::BasicMessage(basic_msg) => {
                assert_eq!(basic_msg.content, message);
                Ok(())
            }
            other => Err(format!("Result is not BasicMessage, but: {:?}", other))
        }
    }

    #[test]
    fn test_parse_generic_message_json_msg_should_be_parsed_as_generic() -> Result<(), String> {
        let message = json!({
            "@id": "some id",
            "@type": "some type",
            "content": "some content"
        }).to_string();
        let result = Connection::parse_generic_message(&message, "");
        match result {
            A2AMessage::Generic(value) => {
                assert_eq!(value.to_string(), message);
                Ok(())
            }
            other => Err(format!("Result is not Generic, but: {:?}", other))
        }
    }
}