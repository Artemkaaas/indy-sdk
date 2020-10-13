use std::collections::HashMap;

use rmp_serde;
use serde_json;
use serde_json::Value;

use api::VcxStateType;
use error::prelude::*;
use messages;
use messages::{GeneralMessage, MessageStatusCode, RemoteMessageType, SerializableObjectWithState};
use messages::invite::{InviteDetail, SenderDetail, Payload as ConnectionPayload, AcceptanceDetails, RedirectDetail, RedirectionDetails};
use messages::payload::{Payloads, PayloadKinds};
use messages::thread::Thread;
use messages::send_message::SendMessageOptions;
use messages::get_message::{Message, MessagePayload};
use object_cache::ObjectCache;
use settings;
use utils::error;
use utils::libindy::signus::create_and_store_my_did;
use utils::libindy::crypto;
use utils::json::mapped_key_rewrite;
use utils::json::KeyMatch;

use v3::handlers::connection::connection::Connection as ConnectionV3;
use v3::handlers::connection::states::ActorDidExchangeState;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::connection::invite::Invitation as InvitationV3;
use v3::messages::a2a::A2AMessage;
use v3::handlers::connection::types::CompletedConnection;
use v3::messages::invite_action::invite::Invite as InviteForAction;

use settings::ProtocolTypes;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connections> = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum Connections {
    #[serde(rename = "1.0")]
    V1(Connection),
    #[serde(rename = "2.0")]
    V3(ConnectionV3),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionOptions {
    #[serde(default)]
    pub connection_type: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    pub use_public_did: Option<bool>,
    pub update_agent_info: Option<bool>,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        ConnectionOptions {
            connection_type: None,
            phone: None,
            use_public_did: None,
            update_agent_info: Some(true),
        }
    }
}

impl ConnectionOptions {
    pub fn from_opt_str(options: Option<&String>) -> VcxResult<ConnectionOptions> {
        Ok(
            match options.as_ref().map(|opt| opt.trim()) {
                None => ConnectionOptions::default(),
                Some(opt) if opt.is_empty() => ConnectionOptions::default(),
                Some(opt) => {
                    serde_json::from_str(opt)
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize ConnectionOptions: {}", err)))?
                }
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Connection {
    source_id: String,
    pw_did: String,
    pw_verkey: String,
    state: VcxStateType,
    uuid: String,
    endpoint: String,
    // For QR code invitation
    invite_detail: Option<InviteDetail>,
    redirect_detail: Option<RedirectDetail>,
    invite_url: Option<String>,
    agent_did: String,
    agent_vk: String,
    their_pw_did: String,
    their_pw_verkey: String,
    // used by proofs/credentials when sending to edge device
    public_did: Option<String>,
    their_public_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<settings::ProtocolTypes>,
}

impl Connection {
    fn _connect_send_invite(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        trace!("Connection::_connect_send_invite >>> options: {:?}", secret!(options));
        debug!("Connection {}: Sending invite", self.source_id);

        let (invite, url) =
            messages::send_invite()
                .to(&self.pw_did)?
                .to_vk(&self.pw_verkey)?
                .phone_number(options.phone.as_ref().map(String::as_str))?
                .agent_did(&self.agent_did)?
                .agent_vk(&self.agent_vk)?
                .public_did(self.public_did.as_ref().map(String::as_str))?
                .thread(&Thread::new())?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.extend("Cannot send Invite"))?;

        self.state = VcxStateType::VcxStateOfferSent;
        self.invite_detail = Some(invite);
        self.invite_url = Some(url);

        trace!("Connection::_connect_send_invite <<<");

        Ok(error::SUCCESS.code_num)
    }

    pub fn delete_connection(&mut self) -> VcxResult<u32> {
        trace!("Connection::delete_connection >>>");
        debug!("Connection {}: deleting connection", self.source_id);

        messages::delete_connection()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .version(&self.version)?
            .send_secure()
            .map_err(|err| err.extend("Cannot delete Connection"))?;

        self.state = VcxStateType::VcxStateNone;

        trace!("Connection::delete_connection <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn _connect_accept_invite(&mut self) -> VcxResult<u32> {
        trace!("Connection::_connect_accept_invite >>>");
        debug!("Connection {}: Accepting invite", self.source_id);

        let details: &InviteDetail = self.invite_detail.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Connection object state: `invite_detail` not found", self.source_id)))?;

        trace!("Connection::_connect_accept_invite: invite: {:?}", secret!(details));

        messages::accept_invite()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .sender_details(&details.sender_detail)?
            .sender_agency_details(&details.sender_agency_detail)?
            .answer_status_code(&MessageStatusCode::Accepted)?
            .reply_to(&details.conn_req_id)?
            .thread(
                &Thread::new()
                    .set_thid(details.thread_id.clone().unwrap_or_default())
                    .update_received_order(&details.sender_detail.did)
            )?
            .version(self.version.clone())?
            .send_secure()
            .map_err(|err| err.extend("Cannot accept Invite"))?;

        self.state = VcxStateType::VcxStateAccepted;

        trace!("Connection::_connect_accept_invite <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn connect(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        trace!("Connection::connect >>> options: {:?}", secret!(options));
        debug!("Connection {}: Connecting", self.source_id);

        match self.state {
            VcxStateType::VcxStateInitialized
            | VcxStateType::VcxStateOfferSent => self._connect_send_invite(options),
            VcxStateType::VcxStateRequestReceived => self._connect_accept_invite(),
            _ => {
                warn!("connection {} in state {} not ready to connect", self.source_id, self.state as u32);
                Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                       format!("Connection {} in state {} not ready to connect", self.source_id, self.state as u32)))
            }
        }
    }

    fn redirect(&mut self, redirect_to: &Connection) -> VcxResult<u32> {
        trace!("Connection::redirect >>> redirect_to: {:?}", redirect_to);
        debug!("Connection {}: Redirecting", self.source_id);

        let details: &InviteDetail = self.invite_detail.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot get `invite_detail` on Connection object {:?}", self.source_id)))?;

        trace!("Connection::redirect: redirection details: {:?}", secret!(details));

        match self.state {
            VcxStateType::VcxStateRequestReceived => {
                messages::redirect_connection()
                    .to(&self.pw_did)?
                    .to_vk(&self.pw_verkey)?
                    .agent_did(&self.agent_did)?
                    .agent_vk(&self.agent_vk)?
                    .sender_details(&details.sender_detail)?
                    .sender_agency_details(&details.sender_agency_detail)?
                    .redirect_details(&redirect_to.generate_redirect_details()?)?
                    .answer_status_code(&MessageStatusCode::Redirected)?
                    .reply_to(&details.conn_req_id)?
                    .thread(
                        &Thread::new()
                            .set_thid(details.thread_id.clone().unwrap_or_default())
                            .update_received_order(&details.sender_detail.did)
                    )?
                    .version(self.version.clone())?
                    .send_secure()
                    .map_err(|err| err.extend("Cannot send redirect"))?;

                self.state = VcxStateType::VcxStateRedirected;

                Ok(error::SUCCESS.code_num)
            }
            _ => {
                warn!("connection {} in state {} not ready to redirect", self.source_id, self.state as u32);
                Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                       format!("Connection {} in state {} not ready to redirect", self.source_id, self.state as u32)))
            }
        }?;

        trace!("Connection::redirect <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn generate_redirect_details(&self) -> VcxResult<RedirectDetail> {
        trace!("Connection::generate_redirect_details >>>");
        debug!("Connection {}: Generating redirection details", self.source_id);

        let signature = format!("{}{}", self.pw_did, self.pw_verkey);
        let signature = ::utils::libindy::crypto::sign(&self.pw_verkey, signature.as_bytes())?;
        let signature = base64::encode(&signature);

        let details = RedirectDetail {
            their_did: self.pw_did.clone(),
            their_verkey: self.pw_verkey.clone(),
            their_public_did: self.public_did.clone(),
            did: self.their_pw_did.clone(),
            verkey: self.their_pw_verkey.clone(),
            public_did: self.their_public_did.clone(),
            signature,
        };

        trace!("Connection::redirect <<< details: {:?}", secret!(details));

        Ok(details)
    }

    fn get_state(&self) -> u32 {
        trace!("Connection::get_state >>>");

        let state = self.state as u32;

        debug!("Connection {} is in state {}", self.source_id, self.state as u32);
        trace!("Connection::get_state <<< state: {:?}", state);
        state
    }

    fn set_state(&mut self, state: VcxStateType) {
        trace!("Connection::set_state >>> state: {:?}", state);
        self.state = state;
    }

    fn get_pw_did(&self) -> &String { &self.pw_did }
    fn set_pw_did(&mut self, did: &str) { self.pw_did = did.to_string(); }

    fn get_their_pw_did(&self) -> &String { &self.their_pw_did }
    fn set_their_pw_did(&mut self, did: &str) { self.their_pw_did = did.to_string(); }

    fn set_their_public_did(&mut self, did: &str) { self.their_public_did = Some(did.to_string()); }
    fn get_their_public_did(&self) -> Option<String> { self.their_public_did.clone() }

    fn get_agent_did(&self) -> &String { &self.agent_did }
    fn set_agent_did(&mut self, did: &str) { self.agent_did = did.to_string(); }

    fn get_pw_verkey(&self) -> &String { &self.pw_verkey }
    fn set_pw_verkey(&mut self, verkey: &str) { self.pw_verkey = verkey.to_string(); }

    fn get_their_pw_verkey(&self) -> &String { &self.their_pw_verkey }
    fn set_their_pw_verkey(&mut self, verkey: &str) { self.their_pw_verkey = verkey.to_string(); }

    fn get_agent_verkey(&self) -> &String { &self.agent_vk }
    fn set_agent_verkey(&mut self, verkey: &str) { self.agent_vk = verkey.to_string(); }

    fn get_invite_detail(&self) -> &Option<InviteDetail> { &self.invite_detail }
    fn set_invite_detail(&mut self, id: InviteDetail) {
        self.version = match id.version.is_some() {
            true => Some(settings::ProtocolTypes::from(id.version.clone().unwrap())),
            false => Some(settings::get_connecting_protocol_version()),
        };
        self.invite_detail = Some(id);
    }

    #[allow(dead_code)]
    fn get_redirect_detail(&self) -> &Option<RedirectDetail> { &self.redirect_detail }
    fn set_redirect_detail(&mut self, rd: RedirectDetail) { self.redirect_detail = Some(rd); }

    fn get_version(&self) -> Option<settings::ProtocolTypes> {
        self.version.clone()
    }

    fn get_source_id(&self) -> &String { &self.source_id }

    fn create_agent_pairwise(&mut self) -> VcxResult<u32> {
        trace!("Connection::create_agent_pairwise >>>");
        debug!("Connection {}: Creating pairwise agent", self.source_id);

        let (for_did, for_verkey) = messages::create_keys()
            .for_did(&self.pw_did)?
            .for_verkey(&self.pw_verkey)?
            .version(&self.version)?
            .send_secure()
            .map_err(|err| err.extend("Cannot create pairwise agent"))?;

        debug!("Connection {}: Created pairwise agent with did {:?}, vk: {:?}", self.source_id, secret!(for_did), secret!(for_verkey));
        self.set_agent_did(&for_did);
        self.set_agent_verkey(&for_verkey);

        trace!("Connection::create_agent_pairwise <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn update_agent_profile(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        trace!("Connection::create_agent_pairwise >>> options: {:?}", secret!(options));
        debug!("Connection {}: Updating agent config", self.source_id);

        if options.use_public_did.unwrap_or(false) {
            self.public_did = Some(settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?);
        };

        ::messages::agent_utils::update_agent_profile(&self.pw_did,
                                                      &self.public_did,
                                                      ProtocolTypes::V1)
    }

    pub fn update_state(&mut self, message: Option<String>) -> VcxResult<u32> {
        trace!("Connection::update_state >>> message: {:?}", secret!(message));
        debug!("Connection {}: Updating state", self.source_id);

        if self.state == VcxStateType::VcxStateInitialized ||
            self.state == VcxStateType::VcxStateAccepted ||
            self.state == VcxStateType::VcxStateRedirected {
            return Ok(self.get_state());
        }

        let response =
            messages::get_messages()
                .to(&self.pw_did)?
                .to_vk(&self.pw_verkey)?
                .agent_did(&self.agent_did)?
                .agent_vk(&self.agent_vk)?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.extend("Cannot get connection messages"))?;

        debug!("Connection {}: Received messages: {:?}", self.source_id, secret!(response));

        if self.state == VcxStateType::VcxStateOfferSent || self.state == VcxStateType::VcxStateInitialized {
            for message in response {
                if message.status_code == MessageStatusCode::Accepted && message.msg_type == RemoteMessageType::ConnReqAnswer {
                    debug!("Connection {}: Received connection request answer", self.source_id);

                    let rc = self.process_acceptance_message(&message);
                    if rc.is_err() {
                        self.force_v2_parse_acceptance_details(&message)?;
                    }
                } else if message.status_code == MessageStatusCode::Redirected && message.msg_type == RemoteMessageType::ConnReqRedirect {
                    debug!("Connection {}: Received connection redirect message", self.source_id);

                    let rc = self.process_redirect_message(&message);
                    if rc.is_err() {
                        self.force_v2_parse_redirection_details(&message)?;
                    }
                } else {
                    warn!("Connection {}: Received unexpected message: {:?}", self.source_id, message);
                }
            }
        };

        let state = self.get_state();

        trace!("Connection::update_state <<< state: {:?}", state);

        Ok(state)
    }

    pub fn process_acceptance_message(&mut self, message: &Message) -> VcxResult<()> {
        trace!("Connection::process_acceptance_message >>> message: {:?}", secret!(message));

        let details = parse_acceptance_details(message)
            .map_err(|err| err.extend("Cannot parse acceptance details"))?;

        self.set_their_pw_did(&details.did);
        self.set_their_pw_verkey(&details.verkey);
        self.set_state(VcxStateType::VcxStateAccepted);

        trace!("Connection::process_acceptance_message <<<");

        Ok(())
    }


    pub fn send_generic_message(&self, message: &str, msg_options: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Sending generic message", self.source_id);

        if self.state != VcxStateType::VcxStateAccepted {
            return Err(VcxError::from_msg(VcxErrorKind::NotReady, format!("Connection {} is not in Accepted state. Not ready to send message", self.source_id)));
        }

        let msg_options: SendMessageOptions = serde_json::from_str(msg_options)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                          format!("Cannot parse SendMessageOptions from `msg_options` JSON string. Err: {:?}", err)))?;

        let response =
            ::messages::send_message()
                .to(&self.get_pw_did())?
                .to_vk(&self.get_pw_verkey())?
                .msg_type(&RemoteMessageType::Other(msg_options.msg_type.clone()))?
                .version(self.version.clone())?
                .edge_agent_payload(
                    &self.get_pw_verkey(),
                    &self.get_their_pw_verkey(),
                    &message,
                    PayloadKinds::Other(msg_options.msg_type.clone()),
                    None)?
                .agent_did(&self.get_agent_did())?
                .agent_vk(&self.get_agent_verkey())?
                .set_title(&msg_options.msg_title)?
                .set_detail(&msg_options.msg_title)?
                .ref_msg_id(msg_options.ref_msg_id.clone())?
                .status_code(&MessageStatusCode::Accepted)?
                .send_secure()
                .map_err(|err| err.extend("Cannot send generic message"))?;

        let msg_uid = response.get_msg_uid()?;

        debug!("Connection {}: Sent generic message", self.source_id);
        trace!("Connection::send_generic_message <<< msg_uid: {:?}", secret!(msg_uid));
        return Ok(msg_uid);
    }


    pub fn send_invite_action(&self, goal_code: String) -> VcxResult<()> {
        trace!("Connection::send_invite_action >>> goal_code: {:?}", secret!(goal_code));
        debug!("Connection {}: Sending invitation for action", self.source_id);

        if self.state != VcxStateType::VcxStateAccepted {
            return Err(VcxError::from_msg(VcxErrorKind::NotReady, format!("Connection {} is not in Accepted state. Not ready to send message", self.source_id)));
        }

        let invite = InviteForAction::create()
            .set_goal_code(goal_code)
            .to_a2a_message();

        let response =
            ::messages::send_message()
                .to(&self.get_pw_did())?
                .to_vk(&self.get_pw_verkey())?
                .msg_type(&RemoteMessageType::Other(String::from("invite-action")))?
                .version(self.version.clone())?
                .edge_agent_payload(
                    &self.get_pw_verkey(),
                    &self.get_their_pw_verkey(),
                    &json!(invite).to_string(),
                    PayloadKinds::Other(String::from("invite-action")),
                    None)?
                .agent_did(&self.get_agent_did())?
                .agent_vk(&self.get_agent_verkey())?
                .set_title("Take the action")?
                .set_detail("Take the action")?
                .status_code(&MessageStatusCode::Accepted)?
                .send_secure()
                .map_err(|err| err.extend("Cannot send generic message"))?;

        debug!("Connection {}: Sent send invite action", self.source_id);
        trace!("Connection::send_invite_action <<<");
        return Ok(());
    }
}

fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidConnectionHandle)
    } else {
        err
    }
}

pub fn create_agent_keys(source_id: &str, pw_did: &str, pw_verkey: &str) -> VcxResult<(String, String)> {
    /*
        Create User Pairwise Agent in old way.
        Send Messages corresponding to V2 Protocol version to avoid code changes on Agency side.
    */
    trace!("Connection::create_agent_keys >>> pw_did: {:?}, pw_verkey: {:?}", secret!(pw_did), secret!(pw_verkey));
    debug!("creating pairwise keys on agent for connection {}", source_id);

    let (agent_did, agent_verkey) = messages::create_keys()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .version(&Some(settings::get_protocol_type()))?
        .send_secure()
        .map_err(|err| err.extend("Cannot create pairwise agent"))?;

    debug!("created pairwise agent for connection: {} with did {:?}, vk: {:?}", source_id, secret!(agent_did), secret!(agent_verkey));
    trace!("Connection::create_agent_keys <<<");

    Ok((agent_did, agent_verkey))
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn set_agent_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_agent_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported))
        }
    }).map_err(handle_err)
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_agent_did().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_did.to_string())
        }
    }).map_err(handle_err)
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_pw_did().to_string()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_did.to_string())
        }
    }).map_err(handle_err)
}

pub fn set_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_pw_did`"))
        }
    }).map_err(handle_err)
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_did().to_string()),
            Connections::V3(ref connection) => connection.remote_did()
        }
    }).map_err(handle_err)
}

pub fn set_their_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_their_pw_did`"))
        }
    }).map_err(handle_err)
}

pub fn set_their_public_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_public_did(did)),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_their_public_did`"))
        }
    }).map_err(handle_err)
}

pub fn get_their_public_did(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_public_did()),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `get_their_public_did`"))
        }
    }).map_err(handle_err)
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_verkey().to_string()),
            Connections::V3(ref connection) => connection.remote_vk()
        }
    }).map_err(handle_err)
}

pub fn set_their_pw_verkey(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_verkey(did)),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_their_pw_verkey`"))
        }
    }).map_err(handle_err)
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_agent_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_vk.clone())
        }
    }).map_err(handle_err)
}

pub fn get_version(handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_version()),
            Connections::V3(_) => Ok(Some(settings::get_protocol_type()))
        }
    }).map_err(handle_err)
}

pub fn set_agent_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_agent_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_agent_verkey`"))
        }
    }).map_err(handle_err)
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_pw_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_vk.clone())
        }
    }).map_err(handle_err)
}

pub fn set_pw_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_pw_verkey`"))
        }
    }).map_err(handle_err)
}

pub fn get_state(handle: u32) -> u32 {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_state()),
            Connections::V3(ref connection) => Ok(connection.state())
        }
    }).unwrap_or(0)
}


pub fn set_state(handle: u32, state: VcxStateType) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_state(state)),
            Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `set_state`"))
        }
    }).map_err(handle_err)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_source_id().clone()),
            Connections::V3(ref connection) => Ok(connection.get_source_id())
        }
    }).map_err(handle_err)
}

fn store_connection(connection: Connections) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

fn create_connection_v1(source_id: &str) -> VcxResult<Connection> {
    trace!("create_connection_v1 >>> source_id: {:?}", source_id);

    let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();

    let (pw_did, pw_verkey) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

    let connection = Connection {
        source_id: source_id.to_string(),
        pw_did,
        pw_verkey,
        state: VcxStateType::VcxStateInitialized,
        uuid: String::new(),
        endpoint: String::new(),
        invite_detail: None,
        redirect_detail: None,
        invite_url: None,
        agent_did: String::new(),
        agent_vk: String::new(),
        their_pw_did: String::new(),
        their_pw_verkey: String::new(),
        public_did: None,
        their_public_did: None,
        version: Some(settings::get_connecting_protocol_version()),
    };

    trace!("create_connection_v1 <<<");
    Ok(connection)
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    debug!("create_connection with source_id: {}", source_id);

    // Initiate connection of new format -- redirect to v3 folder
    if settings::is_aries_protocol_set() {
        let connection = Connections::V3(ConnectionV3::create(source_id));
        let handle = store_connection(connection);
        debug!("create_connection >>> created connection V3, handle: {:?}", handle);
        return handle;
    }

    let connection = create_connection_v1(source_id)?;

    let handle = store_connection(Connections::V1(connection));

    debug!("create_connection >>> created connection V1, handle: {:?}", handle);
    handle
}

pub fn create_outofband_connection(source_id: &str, goal_code: Option<String>, goal: Option<String>,
                                   handshake: bool, request_attach: Option<String>) -> VcxResult<u32> {
    debug!("create_outofband_connection with source_id: {}, goal_code: {:?}, goal: {:?}, handshake: {}, request_attach: {:?}",
           source_id, secret!(goal_code), secret!(goal), secret!(handshake), secret!(request_attach));

    if !settings::is_aries_protocol_set() {
        return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported,
                                      "Library must be initialized with `Aries` related `protocol_type` (`3.0`) to create Out-of-Band connection"));
    }

    let connection = Connections::V3(ConnectionV3::create_outofband(source_id, goal_code, goal, handshake, request_attach));
    let handle = store_connection(connection);

    debug!("create_connection >>> created out-of-band connection V3, handle: {:?}", handle);
    return handle;
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, secret!(details));

    // Invitation of new format -- redirect to v3 folder
    if let Ok(invitation) = serde_json::from_str::<InvitationV3>(details) {
        let connection = ConnectionV3::create_with_invite(source_id, invitation)?;
        let handle = store_connection(Connections::V3(connection));
        debug!("create_connection_with_invite: created connection v3, handle: {:?}", handle);
        return handle;
    }

    let details: Value = serde_json::from_str(&details)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidInviteDetail,
                                          format!("Cannot parse ConnectionInvitationDetails from `invite_details` JSON string. Err: {:?}", err)))?;

    let invite_details: InviteDetail = match serde_json::from_value(details.clone()) {
        Ok(x) => x,
        Err(_) => {
            // Try converting to abbreviated
            let details = unabbrv_event_detail(details)?;
            serde_json::from_value(details)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidInviteDetail,
                                                  format!("Cannot parse ConnectionInvitationDetails from `invite_details` JSON string. Err: {:?}", err)))?
        }
    };

    let mut connection = create_connection_v1(source_id)?;

    connection.set_their_pw_did(invite_details.sender_detail.did.as_str());
    connection.set_their_pw_verkey(invite_details.sender_detail.verkey.as_str());

    if let Some(did) = invite_details.sender_detail.public_did.as_ref() {
        connection.set_their_public_did(did);
    }

    connection.set_invite_detail(invite_details);
    connection.set_state(VcxStateType::VcxStateRequestReceived);

    let handle = store_connection(Connections::V1(connection))?;

    debug!("create_connection_with_invite >>> created out-of-band connection V1, handle: {:?}", handle);
    return Ok(handle);
}

pub fn create_connection_with_outofband_invite(source_id: &str, invitation: &str) -> VcxResult<u32> {
    debug!("create connection {} with outofband invite {}", source_id, secret!(invitation));

    let invitation = ::serde_json::from_str(invitation)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidRedirectDetail,
                                          format!("Could not parse Aries Out-of-Band Invitation  from `invite` JSON string, err: {:?}", err)))?;

    let connection = Connections::V3(ConnectionV3::create_with_outofband_invite(source_id, invitation)?);
    let handle = store_connection(connection);
    debug!("create_connection_with_outofband_invite: created connection v3, handle: {:?}", handle);
    return handle;
}

pub fn accept_connection_invite(source_id: &str,
                                details: &str,
                                options: Option<String>) -> VcxResult<(u32, String)> {
    debug!("create connection {} with invite {}", source_id, secret!(details));

    let connection_handle = create_connection_with_invite(source_id, details)?;
    connect(connection_handle, options)?;
    let connection_serialized = to_string(connection_handle)?;

    debug!("accept_connection_invite: created connection v3, handle: {:?}", connection_handle);

    Ok((connection_handle, connection_serialized))
}

pub fn parse_acceptance_details(message: &Message) -> VcxResult<SenderDetail> {
    trace!("Connection::parse_acceptance_details >>> message: {:?}", secret!(message));
    debug!("Connection: Parsing acceptance details message");

    let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let payload = message.payload
        .as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Received Message does not contain `payload`"))?;

    let acceptance_details = match payload {
        MessagePayload::V1(payload) => {
            trace!("Connection::parse_acceptance_details >>> MessagePayload::V1 payload");

            // TODO: check returned verkey
            let (_, payload) = crypto::parse_msg(&my_vk, &messages::to_u8(&payload))
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot decrypt Message payload. Err: {}", err)))?;

            let response: ConnectionPayload = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot decrypt Message payload. Err: {}", err)))?;

            let payload = messages::to_u8(&response.msg);

            let response: AcceptanceDetails = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

            response.sender_detail
        }
        MessagePayload::V2(payload) => {
            trace!("Connection::parse_acceptance_details >>> MessagePayload::V2 payload");

            let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
            let response: AcceptanceDetails = serde_json::from_str(&payload.msg)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

            response.sender_detail
        }
    };

    trace!("Connection::parse_acceptance_details <<< acceptance_details: {:?}", secret!(acceptance_details));
    Ok(acceptance_details)
}

impl Connection {
    pub fn parse_redirection_details(&self, message: &Message) -> VcxResult<RedirectDetail> {
        trace!("Connection::parse_redirection_details >>> message: {:?}", secret!(message));
        debug!("Connection {}: Parsing redirection details message", self.source_id);

        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Received Message does not contain `payload`"))?;

        let redirection_details = match payload {
            MessagePayload::V1(payload) => {
                trace!("Connection::parse_redirection_details >>> MessagePayload::V1 payload");

                // TODO: check returned verkey
                let (_, payload) = crypto::parse_msg(&my_vk, &messages::to_u8(&payload))
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot decrypt Message payload. Err: {}", err)))?;

                let response: ConnectionPayload = rmp_serde::from_slice(&payload[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot decrypt Message payload. Err: {}", err)))?;

                let payload = messages::to_u8(&response.msg);

                let response: RedirectionDetails = rmp_serde::from_slice(&payload[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse RedirectionDetails from Message payload. Err: {}", err)))?;

                response.redirect_detail
            }
            MessagePayload::V2(payload) => {
                trace!("Connection::parse_redirection_details >>> MessagePayload::V2 payload");

                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: RedirectionDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse RedirectionDetails from Message payload. Err: {}", err)))?;

                response.redirect_detail
            }
        };

        trace!("Connection::parse_redirection_details <<< redirection_details: {:?}", secret!(redirection_details));
        Ok(redirection_details)
    }

    pub fn force_v2_parse_acceptance_details(&mut self, message: &Message) -> VcxResult<SenderDetail> {
        trace!("Connection::force_v2_parse_acceptance_details >>> message: {:?}", secret!(message));
        debug!("Connection {}: Forcing parsing acceptance details", self.source_id);

        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Received Message does not contain `payload`"))?;

        let acceptance_details = match payload {
            MessagePayload::V1(payload) => {
                trace!("Connection::force_v2_parse_acceptance_details >>> MessagePayload::V1 payload");

                let vec = messages::to_u8(payload);
                let json: Value = serde_json::from_slice(&vec[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

                let payload = Payloads::decrypt_payload_v12(&my_vk, &json)?;
                let response: AcceptanceDetails = serde_json::from_value(payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

                self.set_their_pw_did(&response.sender_detail.did);
                self.set_their_pw_verkey(&response.sender_detail.verkey);
                self.set_state(VcxStateType::VcxStateAccepted);

                response.sender_detail
            }
            MessagePayload::V2(payload) => {
                trace!("Connection::force_v2_parse_acceptance_details >>> MessagePayload::V2 payload");

                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: AcceptanceDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

                response.sender_detail
            }
        };

        trace!("Connection::parse_redirection_details <<< force_v2_parse_acceptance_details: {:?}", secret!(acceptance_details));
        Ok(acceptance_details)
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => connection.send_generic_message(&msg, &msg_options),
            Connections::V3(ref connection) => connection.send_generic_message(msg, msg_options)
        }
    }).map_err(handle_err)
}

pub fn update_state_with_message(handle: u32, message: Message) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                if message.status_code == MessageStatusCode::Redirected && message.msg_type == RemoteMessageType::ConnReqRedirect {
                    connection.process_redirect_message(&message)?;
                    Ok(connection.get_state())
                } else {
                    connection.process_acceptance_message(&message)?;
                    Ok(connection.get_state())
                }
            }
            Connections::V3(ref mut connection) => {
                connection.update_state(Some(&json!(message).to_string()))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    }).map_err(handle_err)
}

impl Connection {
    pub fn force_v2_parse_redirection_details(&mut self, message: &Message) -> VcxResult<RedirectDetail> {
        trace!("Connection::force_v2_parse_redirection_details >>> message: {:?}", secret!(message));
        debug!("Connection {}: Forcing parsing redirection details", self.source_id);

        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Received Message does not contain `payload`"))?;

        let redirection_details = match payload {
            MessagePayload::V1(payload) => {
                trace!("Connection::force_v2_parse_redirection_details >>> MessagePayload::V1 payload");

                let vec = messages::to_u8(payload);
                let json: Value = serde_json::from_slice(&vec[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse RedirectionDetails from Message payload. Err: {}", err)))?;

                let payload = Payloads::decrypt_payload_v12(&my_vk, &json)?;
                let response: RedirectionDetails = serde_json::from_value(payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

                self.set_redirect_detail(response.redirect_detail.clone());
                self.set_state(VcxStateType::VcxStateRedirected);

                response.redirect_detail
            }
            MessagePayload::V2(payload) => {
                trace!("Connection::force_v2_parse_redirection_details >>> MessagePayload::V2 payload");

                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: RedirectionDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse AcceptanceDetails from Message payload. Err: {}", err)))?;

                response.redirect_detail
            }
        };

        trace!("Connection::force_v2_parse_redirection_details <<< redirection_details: {:?}", secret!(redirection_details));
        Ok(redirection_details)
    }
}

pub fn update_state(handle: u32, message: Option<String>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.update_state(message.clone())
            }
            Connections::V3(ref mut connection) => {
                connection.update_state(message.as_ref().map(String::as_str))
            }
        }
    }).map_err(handle_err)
}

impl Connection {
    pub fn process_redirect_message(&mut self, message: &Message) -> VcxResult<()> {
        trace!("Connection::process_redirect_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Processing redirection details", self.source_id);

        let details = self.parse_redirection_details(&message)
            .map_err(|err| err.extend("Cannot parse redirection details"))?;

        self.set_redirect_detail(details);
        self.set_state(VcxStateType::VcxStateRedirected);

        trace!("Connection::process_redirect_message <<<");
        Ok(())
    }
}

pub fn delete_connection(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.delete_connection()
            }
            Connections::V3(ref mut connection) => {
                connection.delete()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .map(|_| error::SUCCESS.code_num)
        .map_err(handle_err)
        .and(release(handle))
        .and_then(|_| Ok(error::SUCCESS.code_num))
}

pub fn connect(handle: u32, options: Option<String>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                debug!("establish connection {}", connection.source_id);
                let options_obj: ConnectionOptions = ConnectionOptions::from_opt_str(options.as_ref())?;
                if options_obj.update_agent_info.unwrap_or(true) {
                    connection.update_agent_profile(&options_obj)?;
                }

                connection.create_agent_pairwise()?;
                connection.connect(&options_obj)
            }
            Connections::V3(ref mut connection) => {
                connection.connect()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    }).map_err(handle_err)
}

pub fn redirect(handle: u32, redirect_handle: u32) -> VcxResult<u32> {
    let rc = CONNECTION_MAP.get(redirect_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                Ok(connection.clone())
            }
            Connections::V3(_) => {
                Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `redirect`."))
            }
        }
    }).map_err(handle_err)?;

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                debug!("redirecting connection {}", connection.get_source_id());
                connection.update_agent_profile(&ConnectionOptions::default())?;
                connection.create_agent_pairwise()?;
                connection.redirect(&rc)
            }
            Connections::V3(_) => {
                Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `redirect`."))
            }
        }
    }).map_err(handle_err)
}


pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                let object: SerializableObjectWithState<Connection, ConnectionV3> = SerializableObjectWithState::V1 { data: connection.to_owned() };
                Ok(json!(object).to_string())
            }
            Connections::V3(ref connection) => {
                let (data, state) = connection.to_owned().into();
                let object = SerializableObjectWithState::V2 { data, state };
                Ok(json!(object).to_string())
            }
        }
    }).map_err(handle_err)
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<Connection, ::v3::handlers::connection::states::ActorDidExchangeState> = ::serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                          format!("Cannot parse Connection state object from JSON string. Err: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V1 { data, .. } => {
            CONNECTION_MAP.add(Connections::V1(data))?
        }
        SerializableObjectWithState::V2 { data, state } => {
            CONNECTION_MAP.add(Connections::V3((data, state).into()))?
        }
    };

    Ok(handle)
}

pub fn release(handle: u32) -> VcxResult<()> {
    CONNECTION_MAP.release(handle).map_err(handle_err)
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32, abbreviated: bool) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                match abbreviated {
                    false => {
                        serde_json::to_string(&connection.get_invite_detail())
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize InviteDetail. Err: {}", err)))
                    }
                    true => {
                        let details = serde_json::to_value(&connection.get_invite_detail())
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize InviteDetail. Err: {}", err)))?;
                        let abbr = abbrv_event_detail(details)?;
                        serde_json::to_string(&abbr)
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize abbreviated InviteDetail. Err: {}", err)))
                    }
                }
            }
            Connections::V3(ref connection) => {
                connection.get_invite_details()
            }
        }
    }).map_err(handle_err)
}

pub fn get_redirect_details(handle: u32) -> VcxResult<String> {
    debug!("Get redirect details for connection {}", get_source_id(handle).unwrap_or_default());

    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                serde_json::to_string(&connection.redirect_detail)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize RedirectDetail. Err: {}", err)))
            }
            Connections::V3(_) => {
                Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries Connection type doesn't support this action: `get_redirect_details`."))
            }
        }
    }).map_err(handle_err)
}

pub fn set_redirect_details(handle: u32, redirect_detail: &RedirectDetail) -> VcxResult<()> {
    debug!("Set redirect details for connection {}", get_source_id(handle).unwrap_or_default());

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.set_redirect_detail(redirect_detail.clone());
                Ok(())
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    }).map_err(handle_err)
}


//**********
// Code to convert InviteDetails to Abbreviated String
//**********

impl KeyMatch for (String, Option<String>) {
    fn matches(&self, key: &String, context: &Vec<String>) -> bool {
        if key.eq(&self.0) {
            match context.last() {
                Some(parent) => {
                    if let Some(ref expected_parent) = self.1 {
                        return parent.eq(expected_parent);
                    }
                }
                None => {
                    return self.1.is_none();
                }
            }
        }
        false
    }
}


lazy_static! {
    static ref ABBREVIATIONS: Vec<(String, String)> = {
        vec![
        ("statusCode".to_string(),          "sc".to_string()),
        ("connReqId".to_string(),           "id".to_string()),
        ("senderDetail".to_string(),        "s".to_string()),
        ("name".to_string(),                "n".to_string()),
        ("agentKeyDlgProof".to_string(),    "dp".to_string()),
        ("agentDID".to_string(),            "d".to_string()),
        ("agentDelegatedKey".to_string(),   "k".to_string()),
        ("signature".to_string(),           "s".to_string()),
        ("DID".to_string(), "d".to_string()),
        ("logoUrl".to_string(), "l".to_string()),
        ("verKey".to_string(), "v".to_string()),
        ("senderAgencyDetail".to_string(), "sa".to_string()),
        ("endpoint".to_string(), "e".to_string()),
        ("targetName".to_string(), "t".to_string()),
        ("statusMsg".to_string(), "sm".to_string()),
        ]
    };
}

lazy_static! {
    static ref UNABBREVIATIONS: Vec<((String, Option<String>), String)> = {
        vec![
        (("sc".to_string(), None),                                  "statusCode".to_string()),
        (("id".to_string(), None),                                  "connReqId".to_string()),
        (("s".to_string(), None),                                   "senderDetail".to_string()),
        (("n".to_string(), Some("senderDetail".to_string())),       "name".to_string()),
        (("dp".to_string(), Some("senderDetail".to_string())),      "agentKeyDlgProof".to_string()),
        (("d".to_string(), Some("agentKeyDlgProof".to_string())),   "agentDID".to_string()),
        (("k".to_string(), Some("agentKeyDlgProof".to_string())),   "agentDelegatedKey".to_string()),
        (("s".to_string(), Some("agentKeyDlgProof".to_string())),   "signature".to_string()),
        (("d".to_string(), Some("senderDetail".to_string())),       "DID".to_string()),
        (("l".to_string(), Some("senderDetail".to_string())),       "logoUrl".to_string()),
        (("v".to_string(), Some("senderDetail".to_string())),       "verKey".to_string()),
        (("sa".to_string(), None),                                  "senderAgencyDetail".to_string()),
        (("d".to_string(), Some("senderAgencyDetail".to_string())), "DID".to_string()),
        (("v".to_string(), Some("senderAgencyDetail".to_string())), "verKey".to_string()),
        (("e".to_string(), Some("senderAgencyDetail".to_string())), "endpoint".to_string()),
        (("t".to_string(), None),                                   "targetName".to_string()),
        (("sm".to_string(), None),                                  "statusMsg".to_string()),
        ]
    };
}

fn abbrv_event_detail(val: Value) -> VcxResult<Value> {
    mapped_key_rewrite(val, &ABBREVIATIONS)
}

fn unabbrv_event_detail(val: Value) -> VcxResult<Value> {
    mapped_key_rewrite(val, &UNABBREVIATIONS)
        .map_err(|err| err.extend("Cannot unabbreviate event detail"))
}

impl Into<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn into(self) -> (Connection, ActorDidExchangeState) {
        let invitation = self.get_invitation();
        let data = Connection {
            source_id: self.source_id().clone(),
            pw_did: self.agent_info().pw_did.clone(),
            pw_verkey: self.agent_info().pw_vk.clone(),
            state: VcxStateType::from_u32(self.state()),
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: Some(InviteDetail{
                sender_detail: SenderDetail {
                    verkey: invitation.and_then(|invitation_| invitation_.recipient_key()).unwrap_or_default(),
                    ..SenderDetail::default()
                },
                ..InviteDetail::default()
            }),
            redirect_detail: None,
            invite_url: None,
            agent_did: self.agent_info().agent_did.clone(),
            agent_vk: self.agent_info().agent_vk.clone(),
            their_pw_did: self.remote_did().unwrap_or_default(),
            their_pw_verkey: self.remote_vk().unwrap_or_default(),
            public_did: None,
            their_public_did: None,
            version: Some(ProtocolTypes::V2), // TODO check correctness
        };

        (data, self.state_object().to_owned())
    }
}

impl From<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn from((connection, state): (Connection, ActorDidExchangeState)) -> ConnectionV3 {
        let agent_info = AgentInfo {
            pw_did: connection.get_pw_did().to_string(),
            pw_vk: connection.get_pw_verkey().to_string(),
            agent_did: connection.get_agent_did().to_string(),
            agent_vk: connection.get_agent_verkey().to_string(),
        };

        ConnectionV3::from_parts(connection.get_source_id().to_string(), agent_info, state)
    }
}

pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `get_messages`.")),
            Connections::V3(ref mut connection) => connection.get_messages(),
        }
    }).map_err(handle_err)
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `get_message_by_id`.")),
            Connections::V3(ref mut connection) => connection.get_message_by_id(&msg_id),
        }
    }).map_err(handle_err)
}

pub fn is_v3_connection(connection_handle: u32) -> VcxResult<bool> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Ok(false),
            Connections::V3(_) => Ok(true)
        }
    }).map_err(handle_err)
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `send_ping`.")),
            Connections::V3(ref mut connection) => connection.send_ping(comment.clone())
        }
    }).map_err(handle_err)
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `send_discovery_features`.")),
            Connections::V3(ref mut connection) => connection.send_discovery_features(query.clone(), comment.clone())
        }
    }).map_err(handle_err)
}

pub fn send_reuse(connection_handle: u32, invitation: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported,
                                                         "Proprietary Connection type doesn't support this action: `send_reuse`.")),
            Connections::V3(ref mut connection) => {
                let invitation = ::serde_json::from_str(&invitation)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                      format!("Could not parse Aries Out-of-Band Invitation from `invitation` JSON string. Err: {:?}", err)))?;

                connection.send_reuse(invitation)
            }
        }
    }).map_err(handle_err)
}

pub fn send_answer(connection_handle: u32, question: String, answer: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported,
                                                         "Proprietary Connection type doesn't support this action: `send_answer`.")),
            Connections::V3(ref mut connection) => {
                connection.send_answer(question.clone(), answer.clone())
            }
        }
    })
}

pub fn send_invite_action(connection_handle: u32, goal_code: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                connection.send_invite_action(goal_code.clone())
            },
            Connections::V3(ref mut connection) => {
                connection.send_invite_action(goal_code.clone())
            }
        }
    })
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `get_connection_info`.")),
            Connections::V3(ref connection) => connection.get_connection_info()
        }
    }).map_err(handle_err)
}

pub fn get_completed_connection(handle: u32) -> VcxResult<CompletedConnection> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Connection type doesn't support this action: `get_internal_connection_info`.")),
            Connections::V3(ref connection) => connection.get_completed_connection()
        }
    }).map_err(handle_err)
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use messages::get_message::*;
    use utils::constants::*;
    use utils::constants::INVITE_DETAIL_STRING;

    use super::*;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use utils::constants;

    pub fn build_test_connection() -> u32 {
        let handle = create_connection("alice").unwrap();
        connect(handle, Some("{}".to_string())).unwrap();
        handle
    }

    pub fn create_connected_connections() -> (u32, u32) {
        ::utils::devsetup::set_institution();

        let alice = create_connection("alice").unwrap();
        let my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let options = json!({"use_public_did": true}).to_string();

        connect(alice, Some(options)).unwrap();
        let details = get_invite_details(alice, false).unwrap();

        //BE CONSUMER AND ACCEPT INVITE FROM INSTITUTION
        ::utils::devsetup::set_consumer();

        let faber = create_connection_with_invite("faber", &details).unwrap();

        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber));

        connect(faber, Some("{}".to_string())).unwrap();
        let public_did = get_their_public_did(faber).unwrap().unwrap();
        assert_eq!(my_public_did, public_did);

        //BE INSTITUTION AND CHECK THAT INVITE WAS ACCEPTED
        ::utils::devsetup::set_institution();

        thread::sleep(Duration::from_secs(5));

        update_state(alice, None).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(alice));
        (faber, alice)
    }

    #[test]
    fn test_build_connection_failures_with_no_wallet() {
        let _setup = SetupDefaults::init();

        assert_eq!(create_connection("This Should Fail").unwrap_err().kind(), VcxErrorKind::InvalidWalletHandle);

        assert_eq!(create_connection_with_invite("This Should Fail", "BadDetailsFoobar").unwrap_err().kind(), VcxErrorKind::InvalidInviteDetail);
    }

    #[test]
    fn test_create_connection_agency_failure() {
        let _setup = SetupIndyMocks::init();

        let handle = create_connection("invalid").unwrap();
        let rc = connect(handle, None);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::PostMessageFailed);
    }

    #[test]
    fn test_create_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_connection").unwrap();

        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);
        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, Some("{}".to_string())).unwrap();

        AgencyMock::set_next_response(GET_MESSAGES_INVITE_ACCEPTED_RESPONSE.to_vec());
        update_state(handle, None).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);

        AgencyMock::set_next_response(DELETE_CONNECTION_RESPONSE.to_vec());
        assert_eq!(delete_connection(handle).unwrap(), 0);

        // This errors b/c we release handle in delete connection
        assert!(release(handle).is_err());
    }

    #[test]
    fn test_vcx_connection_connect_options() {
        let _setup = SetupMocks::init();
        let handle = create_connection("test_create_connection").unwrap();
        let mut connection_options = json!({
            "connection_type":"SMS",
            "phone":"123",
            "use_public_did":true,
        });
        assert!(connect(handle, Some(connection_options.to_string())).is_ok());
        connection_options["update_agent_info"] = json!(false);
        assert!(connect(handle, Some(connection_options.to_string())).is_ok());
        connection_options["update_agent_info"] = json!(true);
        assert!(connect(handle, Some(connection_options.to_string())).is_ok());
    }

    #[test]
    fn test_create_drop_create() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);
        let did1 = get_pw_did(handle).unwrap();

        release(handle).unwrap();

        let handle2 = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle2), VcxStateType::VcxStateInitialized as u32);
        let did2 = get_pw_did(handle2).unwrap();

        assert_ne!(handle, handle2);
        assert_eq!(did1, did2);

        release(handle2).unwrap();
    }

    #[test]
    fn test_connection_release_fails() {
        let _setup = SetupEmpty::init();

        let rc = release(1);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_get_state_fails() {
        let _setup = SetupEmpty::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[test]
    fn test_get_string_fails() {
        let _setup = SetupEmpty::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_get_qr_code_data() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_get_qr_code_data").unwrap();

        connect(handle, None).unwrap();

        let details = get_invite_details(handle, true).unwrap();
        assert!(details.contains("\"dp\":"));

        assert_eq!(get_invite_details(0, true).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());

        // Aries connection
        ::settings::set_config_value(::settings::COMMUNICATION_METHOD, "aries");

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());
    }

    #[test]
    fn test_deserialize_existing() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let first_string = to_string(handle).unwrap();

        let handle = from_string(&first_string).unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);
    }

    #[test]
    fn test_retry_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, None).unwrap();
        connect(handle, None).unwrap();
    }

    #[test]
    fn test_parse_redirect_details() {
        let _setup = SetupMocks::init();
        let test_name = "test_parse_acceptance_details";

        let response = Message {
            status_code: MessageStatusCode::Redirected,
            payload: Some(MessagePayload::V1(vec![-110, -109, -81, 99, 111, 110, 110, 82, 101, 113, 82, 101, 100, 105, 114, 101, 99, 116, -93, 49, 46, 48, -84, 105, 110, 100, 121, 46, 109, 115, 103, 112, 97, 99, 107, -36, 0, -24, -48, -111, -48, -105, -48, -74, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, -48, -39, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, -48, -64, -48, -74, 56, 88, 70, 104, 56, 121, 66, 122, 114, 112, 74, 81, 109, 78, 121, 90, 122, 103, 111, 84, 113, 66, -48, -39, 44, 69, 107, 86, 84, 97, 55, 83, 67, 74, 53, 83, 110, 116, 112, 89, 121, 88, 55, 67, 83, 98, 50, 112, 99, 66, 104, 105, 86, 71, 84, 57, 107, 87, 83, 97, 103, 65, 56, 97, 57, 84, 54, 57, 65, -48, -64, -48, -39, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61])),
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqRedirect,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let c = Connection {
            source_id: test_name.to_string(),
            pw_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            pw_verkey: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            state: VcxStateType::VcxStateOfferSent,
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            redirect_detail: None,
            invite_url: None,
            agent_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            agent_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            their_pw_did: String::new(),
            their_pw_verkey: String::new(),
            public_did: None,
            their_public_did: None,
            version: None,
        };

        c.parse_redirection_details(&response).unwrap();

        // test that it fails
        let bad_response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: None,
            // This will cause an error
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let e = c.parse_redirection_details(&bad_response).unwrap_err();
        assert_eq!(e.kind(), VcxErrorKind::InvalidAgencyResponse);
    }

    #[test]
    fn test_parse_acceptance_details() {
        let _setup = SetupMocks::init();

        let test_name = "test_parse_acceptance_details";

        let response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: Some(MessagePayload::V1(vec![-126, -91, 64, 116, 121, 112, 101, -125, -92, 110, 97, 109, 101, -83, 99, 111, 110, 110, 82, 101, 113, 65, 110, 115, 119, 101, 114, -93, 118, 101, 114, -93, 49, 46, 48, -93, 102, 109, 116, -84, 105, 110, 100, 121, 46, 109, 115, 103, 112, 97, 99, 107, -92, 64, 109, 115, 103, -36, 1, 53, -48, -127, -48, -84, 115, 101, 110, 100, 101, 114, 68, 101, 116, 97, 105, 108, -48, -125, -48, -93, 68, 73, 68, -48, -74, 67, 113, 85, 88, 113, 53, 114, 76, 105, 117, 82, 111, 100, 55, 68, 67, 52, 97, 86, 84, 97, 115, -48, -90, 118, 101, 114, 75, 101, 121, -48, -39, 44, 67, 70, 86, 87, 122, 118, 97, 103, 113, 65, 99, 117, 50, 115, 114, 68, 106, 117, 106, 85, 113, 74, 102, 111, 72, 65, 80, 74, 66, 111, 65, 99, 70, 78, 117, 49, 55, 113, 117, 67, 66, 57, 118, 71, -48, -80, 97, 103, 101, 110, 116, 75, 101, 121, 68, 108, 103, 80, 114, 111, 111, 102, -48, -125, -48, -88, 97, 103, 101, 110, 116, 68, 73, 68, -48, -74, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, -48, -79, 97, 103, 101, 110, 116, 68, 101, 108, 101, 103, 97, 116, 101, 100, 75, 101, 121, -48, -39, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, -48, -87, 115, 105, 103, 110, 97, 116, 117, 114, 101, -48, -39, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61])),
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let c = Connections::V1(Connection {
            source_id: test_name.to_string(),
            pw_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            pw_verkey: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            state: VcxStateType::VcxStateOfferSent,
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            redirect_detail: None,
            invite_url: None,
            agent_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            agent_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            their_pw_did: String::new(),
            their_pw_verkey: String::new(),
            public_did: None,
            their_public_did: None,
            version: None,
        });

        CONNECTION_MAP.add(c).unwrap();

        parse_acceptance_details(&response).unwrap();

        // test that it fails
        let bad_response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: None,
            // This will cause an error
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let e = parse_acceptance_details(&bad_response).unwrap_err();
        assert_eq!(e.kind(), VcxErrorKind::InvalidAgencyResponse);
    }

    #[test]
    fn test_invite_detail_abbr() {
        let _setup = SetupEmpty::init();

        let un_abbr = json!({
          "statusCode":"MS-102",
          "connReqId":"yta2odh",
          "senderDetail":{
            "name":"ent-name",
            "agentKeyDlgProof":{
              "agentDID":"N2Uyi6SVsHZq1VWXuA3EMg",
              "agentDelegatedKey":"CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
              "signature":"/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg=="
            },
            "DID":"F2axeahCaZfbUYUcKefc3j",
            "logoUrl":"ent-logo-url",
            "verKey":"74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx"
          },
          "senderAgencyDetail":{
            "DID":"BDSmVkzxRYGE4HKyMKxd1H",
            "verKey":"6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
            "endpoint":"52.38.32.107:80/agency/msg"
          },
          "targetName":"there",
          "statusMsg":"message sent"
        });

        let abbr = json!({
          "sc":"MS-102",
          "id": "yta2odh",
          "s": {
            "n": "ent-name",
            "dp": {
              "d": "N2Uyi6SVsHZq1VWXuA3EMg",
              "k": "CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
              "s":
                "/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg==",
            },
            "d": "F2axeahCaZfbUYUcKefc3j",
            "l": "ent-logo-url",
            "v": "74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx",
          },
          "sa": {
            "d": "BDSmVkzxRYGE4HKyMKxd1H",
            "v": "6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
            "e": "52.38.32.107:80/agency/msg",
          },
          "t": "there",
          "sm":"message sent"
        });
        let processed = abbrv_event_detail(un_abbr.clone()).unwrap();
        assert_eq!(processed, abbr);
        let unprocessed = unabbrv_event_detail(processed).unwrap();
        assert_eq!(unprocessed, un_abbr);
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_connection("rel1").unwrap();
        let h2 = create_connection("rel2").unwrap();
        let h3 = create_connection("rel3").unwrap();
        let h4 = create_connection("rel4").unwrap();
        let h5 = create_connection("rel5").unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_create_with_valid_invite_details() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();
        connect(handle, None).unwrap();

        let handle_2 = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();
        connect(handle_2, None).unwrap();
    }

    #[test]
    fn test_process_acceptance_message() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        let message = serde_json::from_str(INVITE_ACCEPTED_RESPONSE).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, update_state_with_message(handle, message).unwrap());
    }

    #[test]
    fn test_create_with_invalid_invite_details() {
        let _setup = SetupMocks::init();

        let bad_details = r#"{"id":"mtfjmda","s":{"d":"abc"},"l":"abc","n":"Evernym","v":"avc"},"sa":{"d":"abc","e":"abc","v":"abc"},"sc":"MS-101","sm":"message created","t":"there"}"#;
        let err = create_connection_with_invite("alice", &bad_details).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidInviteDetail);
    }

    #[test]
    fn test_void_functions_actually_have_results() {
        let _setup = SetupDefaults::init();

        assert_eq!(set_their_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_state(1, VcxStateType::VcxStateNone).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_their_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_agent_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        let _invite_details: InviteDetail = serde_json::from_str(INVITE_DETAIL_STRING).unwrap();

        let redirect_details: RedirectDetail = serde_json::from_str(REDIRECT_DETAIL_STRING).unwrap();
        assert_eq!(set_redirect_details(1, &redirect_details).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_different_protocol_version() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Ok(()),
                Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V1")),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_V3_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Connections::V3(_) => Ok(()),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_connection_redirection_real() {
        let _setup = SetupLibraryAgencyV1::init();

        //0. Create initial connection
        let (faber, alice) = ::connection::tests::create_connected_connections();

        //1. Faber sends another invite
        ::utils::devsetup::set_institution(); //Faber to Alice
        let alice2 = create_connection("alice2").unwrap();
        let my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let options = json!({"use_public_did": true}).to_string();
        connect(alice2, Some(options)).unwrap();
        let details_for_alice2 = get_invite_details(alice2, false).unwrap();
        println!("alice2 details: {}", details_for_alice2);

        //2. Alice receives (recognizes that there is already a connection), calls different api (redirect rather than regular connect)
        //BE CONSUMER AND REDIRECT INVITE FROM INSTITUTION
        ::utils::devsetup::set_consumer();
        let faber_duplicate = create_connection_with_invite("faber_duplicate", &details_for_alice2).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber_duplicate));
        redirect(faber_duplicate, faber).unwrap();
        let public_did = get_their_public_did(faber_duplicate).unwrap().unwrap();
        assert_eq!(my_public_did, public_did);

        //3. Faber waits for redirect state change
        //BE INSTITUTION AND CHECK THAT INVITE WAS ACCEPTED
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(2000));
        update_state(alice2, None).unwrap();
        assert_eq!(VcxStateType::VcxStateRedirected as u32, get_state(alice2));

        //4. Faber calls 'get_redirect_data' and based on data, finds old connection  (business logic of enterprise)
        let redirect_data = get_redirect_details(alice2).unwrap();
        println!("redirect_data: {}", redirect_data);

        let rd: RedirectDetail = serde_json::from_str(&redirect_data).unwrap();
        let alice_serialized = to_string(alice).unwrap();

        let to_alice_old: Connection = ::messages::ObjectWithVersion::deserialize(&alice_serialized)
            .map(|obj: ::messages::ObjectWithVersion<Connection>| obj.data).unwrap();


        // Assert redirected data match old connection to alice
        assert_eq!(rd.did, to_alice_old.pw_did);
        assert_eq!(rd.verkey, to_alice_old.pw_verkey);
        assert_eq!(rd.public_did, to_alice_old.public_did);
        assert_eq!(rd.their_did, to_alice_old.their_pw_did);
        assert_eq!(rd.their_verkey, to_alice_old.their_pw_verkey);
        assert_eq!(rd.their_public_did, to_alice_old.their_public_did);
    }

    #[test]
    fn test_accept_connection_invite() {
        let _setup = SetupMocks::init();

        let (connection_handle, connection_serialized) =
            accept_connection_invite("test", INVITE_DETAIL_STRING, None).unwrap();

        assert!(connection_handle > 0);
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(connection_handle));
        assert_eq!(connection_serialized, to_string(connection_handle).unwrap());
    }
}
