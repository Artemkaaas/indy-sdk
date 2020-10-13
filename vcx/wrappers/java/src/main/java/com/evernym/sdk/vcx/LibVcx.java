package com.evernym.sdk.vcx;

import com.sun.jna.*;
import com.sun.jna.ptr.PointerByReference;
import static com.sun.jna.Native.detach;

import java.io.File;

public abstract class LibVcx {
    private static final String LIBRARY_NAME = "vcx";
    /*
     * Native library interface
     */

    private static final String LIBVCX_LOGGER_PREFIX = String.format("%s.native", LibVcx.class.getName());


    /**
     * JNA method signatures for calling SDK function.
     */
    public interface API extends Library {

        public int vcx_init_with_config(int command_handle, String config, Callback cb);
        public int vcx_init(int command_handle, String config_path, Callback cb);
        public int vcx_init_minimal(String config);
        public int vcx_init_pool(int command_handle, String pool_config, Callback cb);

        public String vcx_error_c_message(int error_code);
        public String vcx_version();
        public int vcx_shutdown(boolean delete);
        public int vcx_reset();

        /**
         * Init Sovtoken payment library
         *
         * @return  error code
         */
        public int sovtoken_init();
//        public int nullpay_init();

        /**
         * Helper API for testing purposes.
         *
         * @param msg   message to mock next Agency response in case testing mode is enabled.
         */
        public void vcx_set_next_agency_response(int msg);

        /*
        * Helper API to fetch last error details
        * */
        public void vcx_get_current_error(PointerByReference error);

        /**
         * The API represent Credential Schema that will be published on the Ledger and used for Issuance.
         */

        /**
         * Create a new Schema object and publish correspondent record on the ledger.
         */
        public int vcx_schema_create(int command_handle, String source_id, String schema_name, String version, String schema_data, int payment_handle, Callback cb);

         /**
         * Create a new Schema object that will be published by Endorser later.
         */
        public int vcx_schema_prepare_for_endorser(int command_handle, String source_id, String schema_name, String version, String schema_data, String endorser, Callback cb);

        /**
         * Takes the schema object and returns a json string of all its attributes.
         */
        public int vcx_schema_serialize(int command_handle, int schema_handle, Callback cb);

        /**
         * Takes a json string representing a schema object and recreates an object matching the json.
         */
        public int vcx_schema_deserialize(int command_handle, String serialized_schema, Callback cb);

        /**
         * Retrieves all of the data associated with a schema on the ledger.
         */
        public int vcx_schema_get_attributes(int command_handle, String source_id, String schema_id, Callback cb);

        /**
         * Retrieves schema's id.
         */
        public int vcx_schema_get_schema_id(int command_handle, int schema_handle, Callback cb);

        /**
         * Releases the schema object by de-allocating memory.
         */
        public int vcx_schema_release(int handle);

        /**
         * Checks if schema is published on the Ledger and updates the state.
         */
        public int vcx_schema_update_state(int command_handle, int schema_handle, Callback cb);

        /**
         * Get the current state of the schema object.
         */
        public int vcx_schema_get_state(int command_handle, int schema_handle, Callback cb);




        /**
         * Tha API represents a pairwise connection with another identity owner.
         * Once the connection, is established communication can happen securely and privately.
         * Credentials and Presentations are exchanged using this object.
         *
         * For creating a connection with an identity owner for interactions such as exchanging
         * claims and proofs.
         */

        /**
         * Create a Connection object that provides a pairwise connection for an institution's user.
         */
        public int vcx_connection_create(int command_handle, String source_id, Callback cb);

        /**
         * Create a Connection object that provides an Out-of-Band Connection for an institution's user.
         */
        public int vcx_connection_create_outofband(int command_handle, String source_id, String goal_code, String goal, boolean handshake, String requestAttach, Callback cb);

        /**
         * Establishes connection between institution and its user.
         */
        public int vcx_connection_connect(int command_handle, int connection_handle, String connection_type, Callback cb);

        /**
         * Accept connection for the given invitation.
         */
        public int vcx_connection_accept_connection_invite(int command_handle, String source_id, String invite_details, String connection_type, Callback cb);

        /**
         * Asynchronously request a connection to be redirected to old one.
         */
        public int vcx_connection_redirect(int command_handle, int connection_handle, int redirect_connection_handle, Callback cb);

        /**
         * Get the redirect details for the connection.
         */
        public int vcx_connection_get_redirect_details(int command_handle, int connection_handle, Callback cb);

        /**
         * Takes the Connection object and returns a json string of all its attributes.
         */
        public int vcx_connection_serialize(int command_handle, int connection_handle, Callback cb);

        /**
         * Takes a json string representing a connection object and recreates an object matching the json.
         */
        public int vcx_connection_deserialize(int command_handle, String serialized_claim, Callback cb);

        /**
         * Query the agency for the received messages.
         * Checks for any messages changing state in the connection and updates the state attribute.
         */
        public int vcx_connection_update_state(int command_handle, int connection_handle, Callback cb);

        /**
         * Update the state of the Connection object based on the given message.
         */
        public int vcx_connection_update_state_with_message(int command_handle, int connection_handle, String message, Callback cb);

        /**
         * Returns the current state of the Connection object.
         */
        public int vcx_connection_get_state(int command_handle, int connection_handle, Callback cb);

        /**
         * Releases the connection object by de-allocating memory.
         */
        public int vcx_connection_release(int connection_handle);

        /**
         * Get the invite details that were sent or can be sent to the remote side.
         */
        public int vcx_connection_invite_details(int command_handle, int connection_handle, int abbreviated, Callback cb);

        /**
         * Create a Connection object from the given invite_details that provides a pairwise connection.
         */
        public int vcx_connection_create_with_invite(int command_handle, String source_id, String invite_details, Callback cb);

        /**
         * Delete a Connection object from the agency and release its handle.
         */
        public int vcx_connection_delete_connection(int command_handle, int connection_handle, Callback cb);

        /**
         * Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
         */
        public int vcx_connection_send_ping(int command_handle, int connection_handle, String comment, Callback cb);

        /**
         * Send discovery features message to the specified connection to discover which features it supports, and to what extent.
         */
        public int vcx_connection_send_discovery_features(int command_handle, int connection_handle, String query, String comment, Callback cb);

        /**
         * Get the information about the connection state.
         */
        public int vcx_connection_info(int command_handle, int connection_handle, Callback cb);

        /**
         * Retrieves pw_did from Connection object.
         * */
        public int vcx_connection_get_pw_did(int command_handle, int connection_handle, Callback cb);

        /**
         * Get their pairwise did from connection
         * */
        public int vcx_connection_get_their_pw_did(int command_handle, int connection_handle, Callback cb);

        /**
         * Send a message to the specified connection
         */
        public int vcx_connection_send_message(int command_handle, int connection_handle, String msg, String send_message_options, Callback cb);

        /**
         * Generate a signature for the specified data
         */
        public int vcx_connection_sign_data(int command_handle, int connection_handle, byte[] data_raw, int data_len, Callback cb);

        /**
         * Verify the signature is valid for the specified data
         */
        public int vcx_connection_verify_signature(int command_handle, int connection_handle, byte[] data_raw, int data_len, byte[] signature_raw, int signature_len, Callback cb);

        /**
         * Create a Connection object from the given Out-of-Band Invitation.
         */
        public int vcx_connection_create_with_outofband_invitation(int command_handle, String source_id, String invite, Callback cb);

        /**
         * Send a message to reuse existing Connection instead of setting up a new one as response on received Out-of-Band Invitation.
         */
        public int vcx_connection_send_reuse(int command_handle, int connection_handle, String invite, Callback cb);

        /**
         * Send answer on received question message according to Aries question-answer protocol.
         */
        public int vcx_connection_send_answer(int command_handle, int connection_handle, String question, String answer, Callback cb);

        /**
         * Send a message to invite another side to take a particular action.
         */
        public int vcx_connection_send_invite_action(int command_handle, int connection_handle, String goal_code, Callback cb);

        /**
         * The API represents an Issuer side in credential issuance process.
         * Assumes that pairwise connection between Issuer and Holder is already established.
         */

        /** Send a credential offer to user showing what will be included in the actual credential. */
        public int vcx_issuer_create_credential(int command_handle, String source_id, int cred_def_handle, String issuer_did, String credential_data, String credential_name, String price, Callback cb);

        /** Send a credential offer to user showing what will be included in the actual credential. */
        public int vcx_issuer_send_credential_offer(int command_handle, int credential_handle, int connection_handle, Callback cb);

        /** Gets the offer message that can be sent to the specified connection */
        public int vcx_issuer_get_credential_offer_msg(int command_handle, int credential_handle, Callback cb);

        /**
         * Query the agency for the received messages.
         * Checks for any messages changing state in the object and updates the state attribute.
         * */
        public int vcx_issuer_credential_update_state(int command_handle, int credential_handle, Callback cb);

        /** Update the state of the credential based on the given message. */
        public int vcx_issuer_credential_update_state_with_message(int command_handle, int credential_handle, String message, Callback cb);

        /** Get the current state of the issuer credential object. */
        public int vcx_issuer_credential_get_state(int command_handle, int credential_handle, Callback cb);

        /** Sends the credential to the end user (holder). */
        public int vcx_issuer_send_credential(int command_handle, int credential_handle, int connection_handle, Callback cb);

        /** Gets the credential message that can be sent to the user */
        public int vcx_issuer_get_credential_msg(int command_handle, int credential_handle, String my_pw_did, Callback cb);

        /** Takes the credential object and returns a json string of all its attributes. */
        public int vcx_issuer_credential_serialize(int command_handle, int credential_handle, Callback cb);

        /** Takes a json string representing an issuer credential object and recreates an object matching the json. */
        public int vcx_issuer_credential_deserialize(int command_handle, String serialized_credential, Callback cb);

        /** Terminates a credential for the specified reason. */
        public int vcx_issuer_terminate_credential(int command_handle, int credential_handle, int state_type, String msg);

        /** Releases the issuer credential object by deallocating memory. */
        public int vcx_issuer_credential_release(int credential_handle);

        /** Populates credential_request with the latest credential request received. (not in MVP) */
        public int vcx_issuer_get_credential_request(int credential_handle, String credential_request);

        /** Sets the credential request in an accepted state. (not in MVP) */
        public int vcx_issuer_accept_credential(int credential_handle);


        /**
         * APIs in this module are called by a verifier throughout the request-proof-and-verify process.
         */

        /**
         * Create a new Proof object that requests a proof for an enterprise
         */
        public int vcx_proof_create(int command_handle, String source_id, String requested_attrs, String requested_predicates, String revocationInterval, String name, Callback cb);


        /**
         * Create a new Proof object based on the given Presentation Proposal message
         */
        public int vcx_proof_create_with_proposal(int command_handle, String source_id, String presentation_proposal, String name, Callback cb);

        /**
         * Sends a proof request to pairwise connection.
         */
        public int vcx_proof_send_request(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Get the proof request message that can be sent to the specified connection.
         */
        public int vcx_proof_get_request_msg(int command_handle, int proof_handle, Callback cb);

        /**
         * Populate response_data with the latest proof offer received.
         * Todo: This should be depricated, use vcx_get_proof_msg
         */
        public int vcx_get_proof(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Get Proof message.
        */
        public int vcx_get_proof_msg(int command_handle, int proof_handle, Callback cb);

        /**
         * Set proof offer as accepted.
         */
        public int vcx_proof_accepted(int proof_handle, String response_data);

        /**
         * Query the agency for the received messages.
         * Checks for any messages changing state in the object and updates the state attribute.
         */
        public int vcx_proof_update_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Update the state of the proof based on the given message.
         */
        public int vcx_proof_update_state_with_message(int command_handle, int proof_handle, String message, Callback cb);

        /**
         * Get the current state of the proof object.
         */
        public int vcx_proof_get_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Takes the proof object and returns a json string of all its attributes.
         */
        public int vcx_proof_serialize(int command_handle, int proof_handle, Callback cb);

        /**
         * Takes a json string representing a proof object and recreates an object matching the json
         */
        public int vcx_proof_deserialize(int command_handle, String serialized_proof, Callback cb);

        /**
         * Releases the proof object by de-allocating memory
         */
        public int vcx_proof_release(int proof_handle);

        /**
         * APIs in this module are called by a prover throughout the request-proof-and-verify process.
         */

        /**
         * Create a Proof object for fulfilling a corresponding proof request
         */
        public int vcx_disclosed_proof_create_with_request(int command_handle, String source_id, String proof_req, Callback cb);

        /**
         * Send a proof to the connection, called after having received a proof request
         */
        public int vcx_disclosed_proof_send_proof(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Send a proof rejection to the connection, called after having received a proof request
         */
        public int vcx_disclosed_proof_reject_proof(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Get the proof message for sending.
         */
        public int vcx_disclosed_proof_get_proof_msg(int command_handle, int proof_handle, Callback cb);

        /**
         * Get the reject proof message for sending.
         */
        public int vcx_disclosed_proof_get_reject_msg(int command_handle, int proof_handle, Callback cb);

        /**
         * Checks for any state change in the disclosed proof and updates the state attribute
         */
        public int vcx_disclosed_proof_update_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Checks for any state change from the given message and updates the state attribute.
         */
        public int vcx_disclosed_proof_update_state_with_message(int command_handle, int proof_handle, String message, Callback cb);

        /**
         * Check for any proof requests from the connection.
         */
        public int vcx_disclosed_proof_get_requests(int command_handle, int connection_handle, Callback cb);

        /**
         * Get the current state of the disclosed proof object.
         */
        public int vcx_disclosed_proof_get_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Takes the disclosed proof object and returns a json string of all its attributes.
         */
        public int vcx_disclosed_proof_serialize(int command_handle, int proof_handle, Callback cb);

        /**
         * Takes a json string representing an disclosed proof object and recreates an object matching the json.
         */
        public int vcx_disclosed_proof_deserialize(int command_handle, String serialized_proof, Callback cb);

        /**
         * Releases the disclosed proof object by de-allocating memory.
         */
        public int vcx_disclosed_proof_release(int proof_handle);

        /**
         * Create a proof based off of a known message id for a given connection.
         */
        public int vcx_disclosed_proof_create_with_msgid(int command_handle, String source_id, int connection_handle, String msd_id, Callback cb);

        /**
         * Get credentials from wallet matching to the proof request associated with proof object.
         */
        public int vcx_disclosed_proof_retrieve_credentials(int command_handle, int proof_handle, Callback cb);

        /**
         * Accept proof request associated with proof object and generates a proof from the selected credentials and self attested attributes.
         */
        public int vcx_disclosed_proof_generate_proof(int command_handle, int proof_handle, String selected_credentials, String self_attested_attributes, Callback cb);


        /**
         * Declines presentation request..
         */
        public int vcx_disclosed_proof_decline_presentation_request(int command_handle, int proof_handle, int connection_handle, String reason, String proposal, Callback cb);


        /**
         * UtilsApi object
         *
         */

        /*
        * Provision an agent in the agency, populate configuration and wallet for this agent.
        * */
        public String vcx_provision_agent(String json);

        /*
        * Provision an agent in the agency, populate configuration and wallet for this agent.
        * */
        public int vcx_agent_provision_async(int command_handle, String json,Callback cb);

        /*
        * Provision an agent in the agency, populate configuration and wallet for this agent.
        * */
        public String vcx_provision_agent_with_token(String config, String token);

        /*
        * Update information on the agent (ie, comm method and type)
        * */
        public int vcx_get_provision_token(int command_handle, String config, Callback cb);

        /*
        * Update information on the agent (ie, comm method and type)
        * */
        public int vcx_agent_update_info(int command_handle,String json,Callback cb);

        /*
        * Get ledger fees from the network
        * */
        public int vcx_ledger_get_fees(int command_handle, Callback cb);

        /*
        * Retrieve author agreement and acceptance mechanisms set on the Ledger
        * */
        public int vcx_get_ledger_author_agreement(int command_handle, Callback cb);

        /*
        * Set some accepted agreement as active.
        * */
        public int vcx_set_active_txn_author_agreement_meta(String text, String version, String hash, String accMechType, long timeOfAcceptance);

        /// Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
        ///
        /// EXPERIMENTAL
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// submitter_did: DID of the request sender.
        /// text: a content of the TTA.
        /// version: a version of the TTA (unique UTF-8 string).
        /// cb: Callback that takes command result as parameter.
        ///
        /// #Returns
        /// Request result as json.
        ///
        /// #Errors
        /// Common*
        // void           (*cb)(indy_handle_t command_handle_,
        //                        indy_error_t  err,
        //                        const char*   request_json)
        public int indy_build_txn_author_agreement_request(int command_handle, String submitter_did, String text, String version, Callback cb);



        /// Builds a GET_TXN_AUTHR_AGRMT request. Request to get a specific Transaction Author Agreement from the ledger.
        ///
        /// EXPERIMENTAL
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// submitter_did: (Optional) DID of the request sender.
        /// data: (Optional) specifies a condition for getting specific TAA.
        /// Contains 3 mutually exclusive optional fields:
        /// {
        ///     hash: Optional<str> - hash of requested TAA,
        ///     version: Optional<str> - version of requested TAA.
        ///     timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
        /// }
        /// Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.
        ///
        /// cb: Callback that takes command result as parameter.
        ///
        /// #Returns
        /// Request result as json.
        ///
        /// #Errors
        /// Common*
        // void           (*cb)(indy_handle_t command_handle_,
        // indy_error_t  err,
        // const char*   request_json)
        public int indy_build_get_txn_author_agreement_request(int command_handle, String submitter_did, String data, Callback cb);


        /// Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
        /// Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
        ///
        /// EXPERIMENTAL
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// submitter_did: DID of the request sender.
        /// aml: a set of new acceptance mechanisms:
        /// {
        ///     "<acceptance mechanism label 1>": { acceptance mechanism description 1},
        ///     "<acceptance mechanism label 2>": { acceptance mechanism description 2},
        ///     ...
        /// }
        /// version: a version of new acceptance mechanisms. (Note: unique on the Ledger)
        /// aml_context: (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
        /// cb: Callback that takes command result as parameter.
        ///
        /// #Returns
        /// Request result as json.
        ///
        /// #Errors
        /// Common*
        // void           (*cb)(indy_handle_t command_handle_,
        // indy_error_t  err,
        // const char*   request_json)
        public int indy_build_acceptance_mechanisms_request(int command_handle, String submitter_did, String aml, String version, String aml_context, Callback cb);

        /// Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
        /// valid for specified time or the latest one.
        ///
        /// EXPERIMENTAL
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// submitter_did: (Optional) DID of the request sender.
        /// timestamp: i64 - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
        /// version: (Optional) version of acceptance mechanisms.
        /// cb: Callback that takes command result as parameter.
        ///
        /// NOTE: timestamp and version cannot be specified together.
        ///
        /// #Returns
        /// Request result as json.
        ///
        /// #Errors
        /// Common*
        // void           (*cb)(indy_handle_t command_handle_,
        // indy_error_t  err,
        // const char*   request_json)
        public int indy_build_get_acceptance_mechanisms_request(int command_handle, String submitter_did, Long  timestamp, String version, Callback cb);

        /// Append transaction author agreement acceptance data to a request.
        /// This function should be called before signing and sending a request
        /// if there is any transaction author agreement set on the Ledger.
        ///
        /// EXPERIMENTAL
        ///
        /// This function may calculate hash by itself or consume it as a parameter.
        /// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// request_json: original request data json.
        /// text and version - (optional) raw data about TAA from ledger.
        ///     These parameters should be passed together.
        ///     These parameters are required if taa_digest parameter is omitted.
        /// taa_digest - (optional) hash on text and version. This parameter is required if text and version parameters are omitted.
        /// mechanism - mechanism how user has accepted the TAA
        /// time - UTC timestamp when user has accepted the TAA
        /// cb: Callback that takes command result as parameter.
        ///
        /// #Returns
        /// Updated request result as json.
        ///
        /// #Errors
        /// Common*
        // void           (*cb)(indy_handle_t command_handle_,
        // indy_error_t  err,
        // const char*   request_with_meta_json)
        public int  indy_append_txn_author_agreement_acceptance_to_request(int command_handle, String request_json, String text, String version, String taa_digest, String mechanism, Long time, Callback cb);


        /*
        * Set the pool handle before calling vcx_init_minimal
        * */
        public int vcx_pool_set_handle(int handle);

        /*
        * Gets minimal request price for performing an action in case the requester can perform this action.
        * */
        public int vcx_get_request_price(int command_handle, String action_json, String requester_info_json, Callback cb);

        /*
        * Endorse transaction to the ledger preserving an original author
        * */
        public int vcx_endorse_transaction(int command_handle, String transaction, Callback cb);

        /**
         * The API represents a Holder side in credential issuance process.
         */

        /** Create a Credential object that requests and receives a credential for an institution. */
        public int vcx_credential_create_with_offer(int command_handle, String source_id, String credential_offer,Callback cb);

        /** Create a Credential object based off of a known message id for a given connection. */
        public int vcx_credential_create_with_msgid(int command_handle, String source_id, int connection, String msg_id,Callback cb);

        /** Accept credential for the given offer. */
        public int vcx_credential_accept_credential_offer(int command_handle, String source_id, String offer, int connection_handle, Callback cb);

        /** Asynchronously sends the credential request to the connection. */
        public int vcx_credential_send_request(int command_handle, int credential_handle, int connection_handle,int payment_handle, Callback cb);

        /** Approves the credential offer and gets the credential request message that can be sent to the specified connection */
        public int vcx_credential_get_request_msg(int command_handle, int credential_handle, String myPwDid, String theirPwDid, int payment_handle, Callback cb);

        /** Queries agency for credential offers from the given connection. */
        public int vcx_credential_get_offers(int command_handle, int connection_handle,Callback cb);

        /**
         * Query the agency for the received messages.
         * Checks for any messages changing state in the credential object and updates the state attribute.
         * */
        public int vcx_credential_update_state(int command_handle, int credential_handle,Callback cb);

        /** Update the state of the credential based on the given message. */
        public int vcx_credential_update_state_with_message(int command_handle, int credential_handle, String message, Callback cb);

        /** Get the current state of the credential object. */
        public int vcx_credential_get_state(int command_handle, int credential_handle, Callback cb);

        /** Takes the credential object and returns a json string of all its attributes. */
        public int vcx_credential_serialize(int command_handle, int credential_handle, Callback cb);

        /** Takes a json string representing an credential object and recreates an object matching the json. */
        public int vcx_credential_deserialize(int command_handle, String serialized_credential, Callback cb);

        /** Releases the credential object by de-allocating memory. */
        public int vcx_credential_release(int credential_handle);

        /** Retrieve information about a stored credential in user's wallet, including credential id and the credential itself. */
        public int vcx_get_credential(int command_handle, int credential_handle, Callback cb);

        /** Send a Credential rejection to the connection. */
        public int vcx_credential_reject(int command_handle, int credential_handle, int connection_handle, String comment, Callback cb);

        /** Build Presentation Proposal message for revealing Credential data. */
        public int vcx_credential_get_presentation_proposal_msg(int command_handle, int credential_handle, Callback cb);

        /** Delete a credential from the wallet and release it from memory. */
        public int vcx_delete_credential(int command_handle, int credential_handle, Callback cb);

        /**
         * wallet object
         *
         * Used for exporting and importing and managing the wallet.
         */

        /** Export the wallet as an encrypted file */
        public int vcx_wallet_export(int command_handle, String path, String backup_key, Callback cb);

        /** Import an encrypted file back into the wallet */
        public int vcx_wallet_import(int command_handle, String config, Callback cb);

        /** Add a record into wallet */
        public int vcx_wallet_add_record(int command_handle, String recordType, String recordId, String recordValue, String recordTag, Callback cb);

        /** Delete a record from wallet */
        public int vcx_wallet_delete_record(int command_handle, String recordType, String recordId, Callback cb);

        /** Get a record from wallet */
        public int vcx_wallet_get_record(int command_handle, String recordType, String recordId, String optionsJson, Callback cb);

        /** Update a record in wallet */
        public int vcx_wallet_update_record_value(int command_handle, String recordType, String recordId, String recordValue, Callback cb);

        /** Set wallet handle manually */
        public int vcx_wallet_set_handle(int handle);

        /** Create a Wallet Backup object that provides a Cloud wallet backup and provision's backup protocol with Agent */
        public int vcx_wallet_backup_create(int command_handle, String sourceID, String backupKey, Callback cb);

        /** Wallet Backup to the Cloud */
        public int vcx_wallet_backup_backup(int command_handle, int walletBackupHandle, String path, Callback cb);

        /** Checks for any state change and updates the the state attribute */
        public int vcx_wallet_backup_update_state(int commandHandle, int walletBackupHandle, Callback cb);

        /* Checks the message any state change and updates the the state attribute */
        public int vcx_wallet_backup_update_state_with_message(int commandHandle, int walletBackupHandle,  String message, Callback cb);

        /** Takes the wallet backup object and returns a json string of all its attributes */
        public int vcx_wallet_backup_serialize(int commandHandle, int walletBackupHandle, Callback cb);

        /* Takes a json string representing an wallet backup object and recreates an object matching the json */
        public int vcx_wallet_backup_deserialize(int commandHandle, String walletBackupStr, Callback cb);

        /** Retrieve Backup from the cloud and Import the encrypted file back into the wallet */
        public int vcx_wallet_backup_restore(int command_handle, String config, Callback cb);


        /** Sign with payment address **/
        public int vcx_wallet_sign_with_address(int command_handle, String address, byte[] message_raw, int message_len, Callback cb);

        /** Verify with payment address **/
        public int vcx_wallet_verify_with_address(int command_handle, String address, byte[] message_raw, int message_len, byte[] signature_raw, int signature_len, Callback cb);

        /**
         * token object
         *
         * Used for sending tokens and getting token related info
         */

        /** Gets token Balance and payment addresses info */
        public int vcx_wallet_get_token_info(int command_handle, int payment_handle, Callback cb);

        /** Sends token to recipient */
        public int vcx_wallet_send_tokens(int command_handle, int payment_handle, String tokens, String recipient, Callback cb);

        /** Create a payment address and returns it */
        public int vcx_wallet_create_payment_address(int command_handle, String seed, Callback cb);

        /**
         * message object
         *
         * Used for getting and updating messages
         */

        /** Retrieve messages from the agent. */
        public int vcx_messages_download(int command_handle, String messageStatus, String uids, String pwdids, Callback cb);

        /** Retrieve single message from the agency by the given uid. */
        public int vcx_download_message(int command_handle, String uid, Callback cb);

        /** Get messages for given uids from Cloud Agent */
        public int vcx_download_agent_messages(int command_handle, String messageStatus, String uids, Callback cb);

        /** Update the status of messages from the specified connection */
        public int vcx_messages_update_status(int command_handle, String messageStatus, String msgJson, Callback cb);

        /** Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials. */
        public int vcx_fetch_public_entities(int command_handle, Callback cb);

        /**
         * Object representing Credential Definition publishing on the Ledger and used for the Issuance.
         */

        /** Create a new CredentialDef object and publish correspondent record on the ledger. */
        int vcx_credentialdef_create(int command_handle, String source_id, String credentialdef_name, String schema_id, String issuer_did, String tag,  String config, int payment_handle, Callback cb);

        /** Create a new CredentialDef object that will be published by Endorser later. */
        int vcx_credentialdef_prepare_for_endorser(int command_handle, String source_id, String credentialdef_name, String schema_id, String issuer_did, String tag,  String config, String endorser, Callback cb);

        /** Create a new CredentialDef object from a cred_def_id. */
        int vcx_credentialdef_create_with_id(int command_handle, String source_id, String credentialdef_id, String issuer_did, String revocation_config, Callback cb);

        /** Takes the credentialdef object and returns a json string of all its attributes. */
        int vcx_credentialdef_serialize(int command_handle, int credentialdef_handle, Callback cb);

        /** Takes a json string representing a credentialdef object and recreates an object matching the json. */
        int vcx_credentialdef_deserialize(int command_handle, String serialized_credentialdef, Callback cb);

        /** Releases the credentialdef object by de-allocating memory. */
        int vcx_credentialdef_release(int handle);

        /** Retrieves credential definition's id. */
        int vcx_credentialdef_get_cred_def_id(int command_handle, int cred_def_handle, Callback cb);

        /** Checks if credential definition is published on the Ledger and updates the state if it is. */
        public int vcx_credentialdef_update_state(int command_handle, int credentialdef_handle,Callback cb);

        /** Get the current state of the credential definition object */
        public int vcx_credentialdef_get_state(int command_handle, int credentialdef_handle, Callback cb);

        /**
         * logger
         *
         */

        /** Set custom logger implementation. */
        int vcx_set_logger(Pointer context, Callback enabled, Callback log, Callback flush);

        /** Set custom logger implementation with max lvl. */
        int vcx_set_logger_with_max_lvl(Pointer context, Callback enabled, Callback log, Callback flush, int max_lvl);

        /** Set max lvl for current logger implementation. */
        int vcx_set_log_max_lvl(int max_lvl);

        /** Set stdout logger implementation. */
        int vcx_set_default_logger(String log_level);

        /**
         * Evernym extensions
         */

        int indy_crypto_anon_crypt(int command_handle, String their_vk, byte[] message_raw, int message_len, Callback cb);
        int indy_crypto_anon_decrypt(int command_handle, int wallet_handle, String my_vk, byte[] encrypted_msg_raw, int encrypted_msg_len, Callback cb);
    }

    /*
     * Initialization
     */

    public static API api = null;

    static {

        try {
            init();
        } catch (UnsatisfiedLinkError ex) {
            // Library could not be found in standard OS locations.
            // Call init(File file) explicitly with absolute library path.
            ex.printStackTrace();
        }
    }

    /**
     * Initializes the API with the path to the C-Callable library.
     *
     * @param searchPath  The path to the directory containing the C-Callable library file.
     * @param libraryName The name of the library to initialize.
     */
    public static void init(String searchPath, String libraryName) {

        NativeLibrary.addSearchPath(libraryName, searchPath);
        api = Native.loadLibrary(libraryName, API.class);
        initLogger();
    }

    /**
     * Initializes the API with the path to the C-Callable library.
     * Warning: This is not platform-independent.
     *
     * @param file The absolute path to the C-Callable library file.
     */
    public static void init(File file) {

        api = Native.loadLibrary(file.getAbsolutePath(), API.class);
        initLogger();
    }

    /**
     * Initializes the API with the default library.
     */
    public static void init() {

        api = Native.loadLibrary(LIBRARY_NAME, API.class);
        initLogger();
    }

    public static void initByLibraryName(String libraryName) {

        System.loadLibrary(libraryName);
        api = Native.loadLibrary(libraryName, API.class);
        initLogger();
    }

    /**
     * Indicates whether or not the API has been initialized.
     *
     * @return true if the API is initialize, otherwise false.
     */
    public static boolean isInitialized() {

        return api != null;
    }

    public static void logMessage(String loggerName, int level, String message) {
        org.slf4j.Logger logger = org.slf4j.LoggerFactory.getLogger(loggerName);
        switch (level) {
            case 1:
                logger.error(message);
                break;
            case 2:
                logger.warn(message);
                break;
            case 3:
                logger.info(message);
                break;
            case 4:
                logger.debug(message);
                break;
            case 5:
                logger.trace(message);
                break;
            default:
                break;
        }
    }

    private static class Logger {
        private static Callback enabled = null;

        private static Callback log = new Callback() {

            @SuppressWarnings({"unused", "unchecked"})
            public void callback(Pointer context, int level, String target, String message, String module_path, String file, int line) {

                detach(false);

                // NOTE: We must restrict the size of the message because the message could be the whole
                // contents of a file, like a 10 MB log file and we do not want all of that content logged
                // into the log file itself... This is what the log statement would look like
                // 2019-02-19 04:34:12.813-0700 ConnectMe[9216:8454774] Debug indy::commands::crypto | src/commands/crypto.rs:286 | anonymous_encrypt <<< res:
                if (message.length() > 102400) {
                    // if message is more than 100K then log only 10K of the message
                    message = message.substring(0, 10240);
                }
                String msg = String.format("%s:%d | %s", file, line, message);
                logMessage(LIBVCX_LOGGER_PREFIX + target.replace("::", "."), level, msg);
            }
        };

        private static Callback flush = null;
    }

    private static void initLogger() {
        org.slf4j.Logger logger = org.slf4j.LoggerFactory.getLogger(LIBVCX_LOGGER_PREFIX);
        int logLevel;
        if (logger.isTraceEnabled()) {
            logLevel = 5;
        } else if (logger.isDebugEnabled()) {
            logLevel = 4;
        } else if (logger.isInfoEnabled()) {
            logLevel = 3;
        } else if (logger.isWarnEnabled()) {
            logLevel = 2;
        } else if (logger.isErrorEnabled()) {
            logLevel = 1;
        } else { // Off
            logLevel = 0;
        }
        api.vcx_set_logger_with_max_lvl(null, Logger.enabled, Logger.log, Logger.flush, logLevel);
    }
}
