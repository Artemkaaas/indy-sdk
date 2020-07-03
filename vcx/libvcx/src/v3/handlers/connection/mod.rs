pub mod agent;
pub mod states;
pub mod connection;
pub mod messages;
pub mod types;

#[cfg(test)]
pub mod tests {
    use v3::messages::connection::invite::tests::_invitation_json;
    use v3::messages::connection::invite::Invitation;
    use v3::messages::connection::response::Response;
    use v3::messages::connection::did_doc::tests::_service_endpoint;
    use v3::messages::connection::request::tests::_request;
    use v3::messages::ack::Ack;

    pub fn mock_connection() -> u32 {
        let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
        let invitation =
            Invitation::default()
                .set_recipient_keys(vec![key.clone()]);

        let connection_handle = ::connection::create_connection_with_invite("source_id", &json!(invitation).to_string()).unwrap();

        ::connection::connect(connection_handle, None).unwrap();

        let response =
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![key.to_string()], vec![])
                .set_thread_id(&_request().id.0)
                .encode(&key).unwrap();
        ::connection::update_state(connection_handle, Some(json!(response.to_a2a_message()).to_string())).unwrap();

        let ack = Ack::create()
            .set_thread(
                response.thread
                    .set_sender_order(1)
            );
        ::connection::update_state(connection_handle, Some(json!(ack.to_a2a_message()).to_string())).unwrap();

        connection_handle
    }

    fn _setup() {
        ::settings::set_config_value(::settings::COMMUNICATION_METHOD, "aries");
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    #[cfg(feature = "aries")]
    mod aries {
        use super::*;

        use v3::test::{Faber, Alice};
        use v3::messages::ack::tests::_ack;
        use v3::messages::a2a::A2AMessage;

        #[test]
        fn test_create_connection_works() {
            _setup();
            let connection_handle = ::connection::create_connection(_source_id()).unwrap();
            assert!(::connection::is_valid_handle(connection_handle));
            assert_eq!(1, ::connection::get_state(connection_handle));
        }

        #[cfg(feature = "aries")]
        #[test]
        fn test_create_connection_with_invite_works() {
            _setup();
            let connection_handle = ::connection::create_connection_with_invite(_source_id(), &_invitation_json()).unwrap();
            assert!(::connection::is_valid_handle(connection_handle));
            assert_eq!(2, ::connection::get_state(connection_handle));
        }

        #[cfg(feature = "aries")]
        #[test]
        fn test_get_connection_state_works() {
            _setup();
            let connection_handle = ::connection::create_connection(_source_id()).unwrap();
            assert_eq!(1, ::connection::get_state(connection_handle));
        }

        #[cfg(feature = "aries")]
        #[test]
        fn test_connection_send_works() {
            _setup();
            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            let uid: String;
            let message = _ack();

            // Send Message works
            {
                faber.send_message(&message.to_a2a_message());
            }

            {
                // Get Messages works
                alice.activate();

                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(1, messages.len());

                uid = messages.keys().next().unwrap().clone();
                let received_message = messages.values().next().unwrap().clone();

                match received_message {
                    A2AMessage::Ack(received_message) => assert_eq!(message, received_message.clone()),
                    _ => assert!(false)
                }
            }

            let _res = ::messages::get_message::download_messages(None, None, Some(vec![uid.clone()])).unwrap();

            // Get Message by id works
            {
                alice.activate();

                let message = ::connection::get_message_by_id(alice.connection_handle, uid.clone()).unwrap();

                match message {
                    A2AMessage::Ack(ack) => assert_eq!(_ack(), ack),
                    _ => assert!(false)
                }
            }

            // Update Message Status works
            {
                alice.activate();
                alice.update_message_status(uid);
                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(0, messages.len());
            }

            // Send Basic Message works
            {
                faber.activate();

                let basic_message = r#"Hi there"#;
                ::connection::send_generic_message(faber.connection_handle, basic_message, "").unwrap();

                alice.activate();

                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(1, messages.len());

                let uid = messages.keys().next().unwrap().clone();
                let message = messages.values().next().unwrap().clone();

                match message {
                    A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                    _ => assert!(false)
                }
                alice.update_message_status(uid);
            }

            // Download Messages
            {
                use messages::get_message::{download_messages, MessageByConnection, Message};

                let credential_offer = ::v3::messages::issuance::credential_offer::tests::_credential_offer();

                faber.send_message(&credential_offer.to_a2a_message());

                alice.activate();

                let messages: Vec<MessageByConnection> = download_messages(None, Some(vec!["MS-103".to_string()]), None).unwrap();
                let message: Message = messages[0].msgs[0].clone();
                assert_eq!(::messages::RemoteMessageType::Other("aries".to_string()), message.msg_type);
                let payload: ::messages::payload::PayloadV1 = ::serde_json::from_str(&message.decrypted_payload.unwrap()).unwrap();
                let _payload: Vec<::messages::issuance::credential_offer::CredentialOffer> = ::serde_json::from_str(&payload.msg).unwrap();

                alice.update_message_status(message.uid);

            }

            // Helpers
            {
                faber.activate();

                ::connection::get_pw_did(faber.connection_handle).unwrap();
                ::connection::get_pw_verkey(faber.connection_handle).unwrap();
                ::connection::get_their_pw_verkey(faber.connection_handle).unwrap();
                ::connection::get_source_id(faber.connection_handle).unwrap();
            }
        }
    }
}

