package com.evernym.sdk.vcx.connection;

/**
 * Created by abdussami on 05/06/18.
 */


import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;
import com.sun.jna.Pointer;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java9.util.concurrent.CompletableFuture;

/**
 * Created by abdussami on 03/06/18.
 */

/**
 *   The object of the VCX API representing a pairwise relationship with another identity owner.
 *   Once the relationship, or connection, is established communication can happen securely and privately.
 *   Credentials and Proofs are exchanged using this object.
 *
 *   # States
 *
 *   The set of object states and transitions depends on communication method is used.
 *   The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.
 *
 *   proprietary:
 *       Inviter:
 *           VcxStateType::VcxStateInitialized - once `vcx_connection_create` (create Connection object) is called.
 *
 *           VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (send Connection invite) is called.
 *
 *           VcxStateType::VcxStateAccepted - once `connReqAnswer` messages is received.
 *                                            use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
 *           VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.
 *
 *       Invitee:
 *           VcxStateType::VcxStateRequestReceived - once `vcx_connection_create_with_invite` (create Connection object with invite) is called.
 *
 *           VcxStateType::VcxStateAccepted - once `vcx_connection_connect` (accept Connection invite) is called.
 *
 *           VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.
 *
 *   aries:
 *       Inviter:
 *           VcxStateType::VcxStateInitialized - 1) once `vcx_connection_create` (create Connection object) is called.
 *                                               2) once `vcx_connection_create_with_outofband_invitation` (create OutofbandConnection object) is called with `handshake:true`.
 *
 *           VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (prepared Connection invite) is called.
 *
 *           VcxStateType::VcxStateRequestReceived - once `ConnectionRequest` messages is received.
 *                                                   accept `ConnectionRequest` and send `ConnectionResponse` message.
 *                                                   use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
 *
 *           VcxStateType::VcxStateAccepted - 1) once `Ack` messages is received.
 *                                               use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
 *                                            2) once `vcx_connection_connect` is called for Outoband Connection created with `handshake:false`.
 *
 *           VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
 *                                           OR
 *                                       `ConnectionProblemReport` messages is received on state updates.
 *
 *       Invitee:
 *           VcxStateType::VcxStateOfferSent - 1) once `vcx_connection_create_with_invite` (create Connection object with invite) is called.
 *                                             2) once `vcx_connection_create_with_outofband_invitation`
 *                                                (create Connection object with Out-of-Band Invitation containing `handshake_protocols`) is called.
 *
 *           VcxStateType::VcxStateRequestReceived - once `vcx_connection_connect` (accept `ConnectionInvite` and send `ConnectionRequest` message) is called.
 *
 *           VcxStateType::VcxStateAccepted - 1) once `ConnectionResponse` messages is received.
 *                                               send `Ack` message if requested.
 *                                               use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
 *           VcxStateType::VcxStateAccepted - 2) once `vcx_connection_create_with_outofband_invitation`
 *                                               (create one-time Connection object with Out-of-Band Invitation does not containing `handshake_protocols`) is called.
 *
 *           VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
 *                                           OR
 *                                       `ConnectionProblemReport` messages is received on state updates.
 *
 *   # Transitions
 *
 *   proprietary:
 *       Inviter:
 *           VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized
 *           VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
 *           VcxStateType::VcxStateOfferSent - received `connReqAnswer` - VcxStateType::VcxStateAccepted
 *           any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`
 *
 *       Invitee:
 *           VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateRequestReceived
 *           VcxStateType::VcxStateRequestReceived - `vcx_connection_connect` - VcxStateType::VcxStateAccepted
 *           any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`
 *
 *   aries - RFC: https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
 *       Inviter:
 *           VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized
 *           VcxStateType::None - `vcx_connection_create_with_outofband_invitation` - VcxStateType::VcxStateInitialized
 *
 *           VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
 *           VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateAccepted (Out-ob-Band Connection created with `handshake:false`)
 *
 *           VcxStateType::VcxStateOfferSent - received `ConnectionRequest` - VcxStateType::VcxStateRequestReceived
 *           VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone
 *
 *           VcxStateType::VcxStateRequestReceived - received `Ack` - VcxStateType::VcxStateAccepted
 *           VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone
 *
 *           VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted
 *
 *           any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone
 *
 *       Invitee:
 *           VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateOfferSent
 *           VcxStateType::None - `vcx_connection_create_with_outofband_invitation` (invite contains `handshake_protocols`) - VcxStateType::VcxStateOfferSent
 *           VcxStateType::None - `vcx_connection_create_with_outofband_invitation` (no `handshake_protocols`) - VcxStateType::VcxStateAccepted
 *
 *           VcxStateType::VcxStateOfferSent - `vcx_connection_connect` - VcxStateType::VcxStateRequestReceived
 *           VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone
 *
 *           VcxStateType::VcxStateRequestReceived - received `ConnectionResponse` - VcxStateType::VcxStateAccepted
 *           VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone
 *
 *           VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted
 *
 *           any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone
 *
 *   # Messages
 *
 *   proprietary:
 *       ConnectionRequest (`connReq`)
 *       ConnectionRequestAnswer (`connReqAnswer`)
 *
 *   aries:
 *       Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
 *       ConnectionRequest - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#1-connection-request
 *       ConnectionResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#2-connection-response
 *       ConnectionProblemReport - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#error-message-example
 *       Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
 *       Ping - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
 *       PingResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
 *       Query - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#query-message-type
 *       Disclose - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#disclose-message-type
 *       Out-of-Band Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0434-outofband#message-type-httpsdidcommorgout-of-bandverinvitation
 */

public class ConnectionApi extends VcxJava.API {

	private static final Logger logger = LoggerFactory.getLogger("ConnectionApi");


	private static Callback vcxConnectionCreateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = connectionHandle;
			future.complete(result);
		}
	};

	/**
	 * Create a Connection object that provides a pairwise connection for an institution's user.
	 *
	 * @param  sourceId     user personal identification for the connection, should be unique.
	 *
	 * @return              handle that should be used to perform actions with the Connection object.
	 *
	 * @throws VcxException If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxConnectionCreate(String sourceId) throws VcxException {
		ParamGuard.notNull(sourceId, "sourceId");
		logger.debug("vcxConnectionCreate() called with: sourceId = [ {} ]", sourceId);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create(
				commandHandle,
				sourceId,
				vcxConnectionCreateCB
		);
		checkResult(result);
		return future;
	}

	/**
	 * Create a Connection object that provides an Out-of-Band Connection for an institution's user.
	 * NOTE: this method can be used when `aries` protocol is set.
	 * NOTE: this method is EXPERIMENTAL
	 * WARN: `requestAttach` field is not fully supported in the current library state.
	 *        You can use simple messages like Question but it cannot be used
	 *        for Credential Issuance and Credential Presentation.
	 *
	 * @param  sourceId     user personal identification for the connection, should be unique.
	 * @param  goalCode     a self-attested code the receiver may want to display to
	 *                      the user or use in automatically deciding what to do with the out-of-band message.
	 * @param  goal         a self-attested string that the receiver may want to display to the user about
	 *                      the context-specific goal of the out-of-band message.
	 * @param  handshake    whether Inviter wants to establish regular connection using `connections` handshake protocol.
	 *                      if false, one-time connection channel will be created.
	 * @param  requestAttach  An additional message as JSON that will be put into attachment decorator
	 *                        that the receiver can using in responding to the message (for example Question message).
	 *
	 * @return              handle that should be used to perform actions with the Connection object.
	 *
	 * @throws VcxException If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxConnectionCreateOutofband(String sourceId, String goalCode,
	                                                                      String goal, boolean handshake,
	                                                                      String requestAttach) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
		logger.debug("vcxConnectionCreateOutofband() called with: sourceId = [ {} ], goalCode = [****], " +
				"goal = [****], handshake = [****], requestAttach = [****]", sourceId);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create_outofband(
				commandHandle,
				sourceId,
				goalCode,
				goal,
				handshake,
				requestAttach,
				vcxConnectionCreateCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxUpdateStateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int state) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = state;
			future.complete(result);
		}
	};

	/**
	 * Query the agency for the received messages.
	 * Checks for any messages changing state in the Connection and updates the state attribute.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 *
	 * @return                   the most current state of the Connection object.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxConnectionUpdateState(int connectionHandle) throws VcxException {
		logger.debug("vcxConnectionUpdateState() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_update_state(
				commandHandle,
				connectionHandle,
				vcxUpdateStateCB
		);
		checkResult(result);
		return future;
	}

	/**
	 * Update the state of the Connection object based on the given message.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 * @param  message           message to process for any Connection state transitions.
	 *
	 * @return                   the most current state of the Connection object.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxConnectionUpdateStateWithMessage(int connectionHandle, String message) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNull(message, "message");

		logger.debug("vcxConnectionUpdateStateWithMessage() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_update_state_with_message(
				commandHandle,
				connectionHandle,
				message,
				vcxUpdateStateCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxCreateConnectionWithInviteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = connectionHandle;
			future.complete(result);
		}
	};

	/**
	 * Create a Connection object from the given Invitation that provides a pairwise connection.
	 *
	 * @param  invitationId  institution's personal identification for the connection.
	 *                       It'll be used as a connection response label.
	 * @param  inviteDetails A string representing a json object which is provided by an entity that wishes to make a connection.
	 *                       The format depends on used communication protocol:
	 *                          proprietary:
	 *                              "{"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}"
	 *                          aries:
	 *                              "{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation","label":"Alice","recipientKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],"serviceEndpoint":"https://example.com/endpoint","routingKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]}"
	 *
	 * @return               handle that should be used to perform actions with the Connection object.
	 *
	 * @throws VcxException  If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxCreateConnectionWithInvite(String invitationId, String inviteDetails) throws VcxException {
		ParamGuard.notNull(invitationId, "invitationId");
		ParamGuard.notNull(inviteDetails, "inviteDetails");
		logger.debug("vcxCreateConnectionWithInvite() called with: invitationId = [" + invitationId + "], inviteDetails = [****]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create_with_invite(
				commandHandle,
				invitationId,
				inviteDetails,
				vcxCreateConnectionWithInviteCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxCreateConnectionWithOutofbandInviteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			// TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
			Integer result = connectionHandle;
			future.complete(result);
		}
	};

	/**
	 * Create a Connection object from the given Out-of-Band Invitation.
	 * Depending on the format of Invitation there are two way of follow interaction:
	 *     * Invitation contains `handshake_protocols`: regular Connection process will be ran.
	 *         Follow steps as for regular Connection establishment.
	 *     * Invitation does not contain `handshake_protocols`: one-time completed Connection object will be created.
	 *         You can use `vcx_connection_send_message` or specific function to send a response message.
	 *         Note that on repeated message sending an error will be thrown.
	 *
	 * NOTE: this method can be used when `aries` protocol is set.
	 *
	 * WARN: The user has to analyze the value of "request~attach" field yourself and
	 *       create/handle the correspondent state object or send a reply once the connection is established.
	 *
	 * @param  sourceId  user personal identification for the connection, should be unique.
	 * @param  invite    A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
	 *                  {
	 *                      "@type": "https://didcomm.org/out-of-band/%VER/invitation",
	 *                      "@id": "<id used for context as pthid>", -  the unique ID of the message.
	 *                      "label": Optional<string>, - a string that the receiver may want to display to the user,
	 *                                                  likely about who sent the out-of-band message.
	 *                      "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
	 *                                                      the user or use in automatically deciding what to do with the out-of-band message.
	 *                      "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
	 *                                                  about the context-specific goal of the out-of-band message.
	 *                      "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
	 *                                                                  that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
	 *                                                                  One or both of handshake_protocols and request~attach MUST be included in the message.
	 *                      "request~attach": Optional<[
	 *                          {
	 *                              "@id": "request-0",
	 *                              "mime-type": "application/json",
	 *                              "data": {
	 *                                  "json": "<json of protocol message>"
	 *                              }
	 *                          }
	 *                      ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
	 *                            One or both of handshake_protocols and request~attach MUST be included in the message.
	 *                      "service": [
	 *                          {
	 *                              "id": string
	 *                              "type": string,
	 *                              "recipientKeys": [string],
	 *                              "routingKeys": [string],
	 *                              "serviceEndpoint": string
	 *                          }
	 *                      ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
	 *                  }
	 *
	 * @return               handle that should be used to perform actions with the Connection object.
	 *
	 * @throws VcxException  If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> vcxCreateConnectionWithOutofbandInvite(String sourceId, String invite) throws VcxException {
		ParamGuard.notNull(sourceId, "sourceId");
		ParamGuard.notNull(invite, "invite");
		logger.debug("vcxCreateConnectionWithOutofbandInvite() called with: sourceId = [" + sourceId + "], invite = [****]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create_with_outofband_invitation(
				commandHandle,
				sourceId,
				invite,
				vcxCreateConnectionWithOutofbandInviteCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionConnectCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String inviteDetails) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], inviteDetails = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			String result = inviteDetails;
			future.complete(result);
		}
	};

	/**
	 * Establishes connection between institution and its user.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 * @param  connectionType    details indicating if the connection will be established by text or QR Code.
	 *                           "{"connection_type":"SMS","phone":"123","use_public_did":true}"
	 *
	 * @return                   Connection Invite as JSON string.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	@Deprecated
	public static CompletableFuture<String> vcxAcceptInvitation(int connectionHandle, String connectionType) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNullOrWhiteSpace(connectionType, "connectionType");
		return vcxConnectionConnect(connectionHandle, connectionType);
	}

	/**
	 * Establishes connection between institution and its user.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 * @param  connectionType    details indicating if the connection will be established by text or QR Code.
	 *                           "{"connection_type":"SMS","phone":"123","use_public_did":true}"
	 *
	 * @return                   Connection Invite as JSON string.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<String> vcxConnectionConnect(int connectionHandle, String connectionType) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNullOrWhiteSpace(connectionType, "connectionType");
		logger.debug("vcxAcceptInvitation() called with: connectionHandle = [" + connectionHandle + "], connectionType = [****]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_connect(
				commandHandle,
				connectionHandle,
				connectionType,
				vcxConnectionConnectCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionAcceptConnectionInviteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle, String connectionSerialized) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], " +
					"connectionHandle = [" + connectionHandle + "], connectionSerialized = [****]");
			CompletableFuture<AcceptConnectionResult> future = (CompletableFuture<AcceptConnectionResult>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			AcceptConnectionResult result = new AcceptConnectionResult(connectionHandle, connectionSerialized);
			future.complete(result);
		}
	};

	/**
	 * Accept connection for the given invitation.
	 *
	 * This function performs the following actions:
	 *  1. Creates Connection state object from the given invitation (vcxCreateConnectionWithInvite)
	 *  2. Replies to the inviting side (vcxConnectionConnect)
	 *
	 * @param  invitationId  institution's personal identification for the connection.
	 *                       It'll be used as a connection response label.
	 * @param  inviteDetails A string representing a json object which is provided by an entity that wishes to make a connection.
	 *                       The format depends on used communication protocol:
	 *                          proprietary:
	 *                              "{"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}"
	 *                          aries:
	 *                              "{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation","label":"Alice","recipientKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],"serviceEndpoint":"https://example.com/endpoint","routingKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]}"
	 * @param  connectionType    details indicating if the connection will be established by text or QR Code.
	 *                           "{"connection_type":"SMS","phone":"123","use_public_did":true}"
	 *
	 * @return               AcceptConnectionResult object containing:
	 *                          - handle that should be used to perform actions with the Connection object.
	 *                          - Connection object as JSON string
	 *
	 * @throws VcxException  If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<AcceptConnectionResult> vcxConnectionAcceptConnectionInvite(String invitationId,
	                                                                                            String inviteDetails,
	                                                                                            String connectionType) throws VcxException {
		ParamGuard.notNull(invitationId, "invitationId");
		ParamGuard.notNull(inviteDetails, "inviteDetails");
		logger.debug("vcxConnectionAcceptConnectionInvite() called with: invitationId = [" + invitationId + "], " +
				"inviteDetails = [****], connectionType = [****]");
		CompletableFuture<AcceptConnectionResult> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_accept_connection_invite(
				commandHandle,
				invitationId,
				inviteDetails,
				connectionType,
				vcxConnectionAcceptConnectionInviteCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionRedirectCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			future.complete(null);
		}
	};

	/**
	 * Redirect Connection
	 *
	 * @param  connectionHandle            handle pointing to a Connection object.
	 * @param  redirectConnectionHandle    handle pointing to a new Connection object.
	 *
	 * @return                   void
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> vcxConnectionRedirect(int connectionHandle, int redirectConnectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNull(redirectConnectionHandle, "redirectConnectionHandle");
		logger.debug("vcxConnectionRedirect() called with: connectionHandle = [" + connectionHandle + "], redirectConnectionHandle = [" + redirectConnectionHandle + "]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_redirect(
				commandHandle,
				connectionHandle,
				redirectConnectionHandle,
				vcxConnectionRedirectCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionGetRedirectDetailsCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String redirectDetails) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], redirectDetails = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			String result = redirectDetails;
			future.complete(result);
		}
	};

	/**
	 * Get Connection redirect details.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 *
	 * @return                   Connection redirect details as JSON string.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<String> vcxConnectionGetRedirectDetails(int connectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		logger.debug("vcxConnectionGetRedirectDetails() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_get_redirect_details(
				commandHandle,
				connectionHandle,
				vcxConnectionGetRedirectDetailsCB
		);
		checkResult(result);
		return future;
	}


	private static Callback vcxConnectionSerializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String serializedData) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], serializedData = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(serializedData);
		}
	};

	/**
	 * Get JSON string representation of Connection object.
	 *
	 * @param  connectionHandle  handle pointing to a Connection object.
	 *
	 * @return                   Connection object as JSON string.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<String> connectionSerialize(int connectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		logger.debug("connectionSerialize() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_serialize(
				commandHandle,
				connectionHandle,
				vcxConnectionSerializeCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionDeserializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(connectionHandle);
		}
	};

	/**
	 * Takes a json string representing a Connection object and recreates an object matching the JSON.
	 *
	 * @param  connectionData  JSON string representing a Connection object.
	 *
	 * @return                 handle that should be used to perform actions with the Connection object.
	 *
	 * @throws VcxException    If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> connectionDeserialize(String connectionData) throws VcxException {
		ParamGuard.notNull(connectionData, "connectionData");
		logger.debug("connectionDeserialize() called with: connectionData = [****]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_deserialize(
				commandHandle,
				connectionData,
				vcxConnectionDeserializeCB
		);
		checkResult(result);
		return future;
	}


	private static Callback vcxConnectionDeleteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(null);
		}
	};

	/**
	 * Delete a Connection object from the agency and release its handle.
	 * <p>
	 * NOTE: This eliminates the connection and any ability to use it for any communication.
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 *
	 * @return                  void
	 *
	 * @throws VcxException    If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> deleteConnection(int connectionHandle) throws VcxException {
		logger.debug("deleteConnection() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_delete_connection(commandHandle, connectionHandle, vcxConnectionDeleteCB);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionInviteDetailsCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String details) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], details = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			future.complete(details);
		}
	};

	/**
	 * Get the invite details that were sent or can be sent to the remote side.
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 * @param  abbreviated      abbreviated connection details for QR codes or not (applicable for `proprietary` communication method only)
	 *
	 * @return                  Connection Invitation as JSON string
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<String> connectionInviteDetails(int connectionHandle, int abbreviated) throws VcxException {
		logger.debug("connectionInviteDetails() called with: connectionHandle = [" + connectionHandle + "], abbreviated = [****]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_invite_details(commandHandle, connectionHandle, abbreviated, vcxConnectionInviteDetailsCB);
		checkResult(result);
		return future;
	}

	/**
	 * Releases the Connection object by de-allocating memory.
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static int connectionRelease(int connectionHandle) throws VcxException {
		logger.debug("connectionRelease() called with: handle = [" + connectionHandle + "]");
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		int result = LibVcx.api.vcx_connection_release(connectionHandle);
		checkResult(result);

		return result;
	}

	private static Callback vcxConnectionGetStateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int state) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(state);
		}
	};

	/**
	 * Returns the current internal state of the Connection object.
	 * Possible states:
	 *         1 - Initialized
	 *         2 - Connection Request Sent
	 *         3 - Connection Response Received
	 *         4 - Connection Accepted
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 *
	 * @return                  the most current state of the Connection object.
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> connectionGetState(int connectionHandle) throws VcxException {
		logger.debug("connectionGetState() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_get_state(commandHandle, connectionHandle, vcxConnectionGetStateCB);
		checkResult(result);
		return future;
	}

	private static Callback voidCb = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
	 * <p>
	 * Note that this function works in case `aries` communication method is used.
	 * In other cases it returns ActionNotSupported error.
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 * @param  comment          (Optional) human-friendly description of the ping.
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> connectionSendPing(
			int connectionHandle,
			String comment
	) throws VcxException {
		logger.debug("sendPing() called with: connectionHandle = [" + connectionHandle + "], comment = [****]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_ping(commandHandle, connectionHandle, comment, voidCb);
		checkResult(result);

		return future;
	}

	/**
	 * Send discovery features message to the specified connection to discover which features it supports, and to what extent.
	 * <p>
	 * Note that this function works in case `aries` communication method is used.
	 * In other cases it returns ActionNotSupported error.
	 *
	 * @param  connectionHandle handle pointing to a Connection object.
	 * @param  query            (Optional) query string to match against supported message types.
	 * @param  comment          (Optional) human-friendly description of the query.
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> connectionSendDiscoveryFeatures(
			int connectionHandle,
			String query,
			String comment
	) throws VcxException {
		logger.debug("connectionSendDiscoveryFeatures() called with: connectionHandle = [" + connectionHandle + "], query = [****], comment = [****]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_discovery_features(commandHandle, connectionHandle, query, comment, voidCb);
		checkResult(result);

		return future;
	}

    private static Callback vcxConnectionSendMessageCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String msgId) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], msgId = [" + msgId + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(msgId);
        }
    };

	/**
	 * Send a generic message to the pairwise connection.
	 *
	 * @param  connectionHandle     handle pointing to a Connection object.
	 * @param  message              actual message to send
	 * @param  sendMessageOptions   message details
	 *     {
	 *         msg_type: String,            // type of message to send. can be any string.
	 *         msg_title: String,           // message title (user notification)
	 *         ref_msg_id: Option(String),  // If responding to a message, id of the message
	 *     }                             
	 *
	 * @return                      id of sent message
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<String> connectionSendMessage(int connectionHandle, String message, String sendMessageOptions) throws VcxException {
        logger.debug("connectionSendMessage() called with: connectionHandle = [" + connectionHandle + "], message = [****], sendMessageOptions = [****]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_send_message(commandHandle, connectionHandle, message, sendMessageOptions, vcxConnectionSendMessageCB);
        checkResult(result);
        return future;
    }


    private static Callback vcxConnectionSignDataCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, Pointer signature_raw, int signature_len) {

            CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
            if (! checkResult(future, err)) return;

            byte[] encryptedMsg = new byte[signature_len];
            signature_raw.read(0, encryptedMsg, 0, signature_len);

            future.complete(encryptedMsg);
        }
    };

	/**
	 * Generate a signature for the specified data using Connection pairwise keys.
	 *
	 * @param  connectionHandle     handle pointing to a Connection object.
	 * @param  data                 raw data buffer for signature
	 * @param  dataLength           length of data buffer
	 *                                 
	 * @return                      generated signature bytes
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<byte[]> connectionSignData(int connectionHandle, byte[] data, int dataLength) throws VcxException {

        ParamGuard.notNull(data, "data");

        CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_sign_data(commandHandle, connectionHandle, data, dataLength, vcxConnectionSignDataCB);
        checkResult(future, result);

        return future;
    }

    private static Callback vcxConnectionVerifySignatureCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, boolean valid) {

            CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(xcommand_handle);
            if (! checkResult(future, err)) return;

            future.complete(valid);
        }
    };

	/**
	 * Verify the signature is valid for the specified data using Connection pairwise keys.
	 *
	 * @param  connectionHandle     handle pointing to a Connection object.
	 * @param  data                 raw data buffer for signature
	 * @param  dataLength           length of data buffer
	 * @param  signature            signature generate for raw data
	 * @param  signatureLength      length of signature buffer
	 *
	 * @return                      bool whether the signature was valid or not
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<Boolean> connectionVerifySignature(int connectionHandle, byte[] data, int dataLength, byte[] signature, int signatureLength) throws VcxException {

        ParamGuard.notNull(data, "data");
        ParamGuard.notNull(signature, "signature");

        CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_verify_signature(commandHandle, connectionHandle, data, dataLength, signature, signatureLength, vcxConnectionVerifySignatureCB);
        checkResult(future, result);

        return future;
    }

    private static Callback vcxConnectionGetPwDidCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, String pwDid) {

            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            future.complete(pwDid);
        }
    };

	/**
	 * Retrieves pairwise DID used for Connection.
	 *
	 * @param  connectionHandle     handle pointing to a Connection object.
	 *
	 * @return                      pairwise DID used for Connection.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<String> connectionGetPwDid(int connectionHandle) throws VcxException {

        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_get_pw_did(commandHandle, connectionHandle, vcxConnectionGetPwDidCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxConnectionGetTheirPwDidCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, String theirPwDid) {

            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            future.complete(theirPwDid);
        }
    };

	/**
	 * Retrieves pairwise DID of the remote side used for Connection.
	 *
	 * @param  connectionHandle     handle pointing to a Connection object.
	 *
	 * @return                      pairwise DID of the remote side used for Connection.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<String> connectionGetTheirPwDid(int connectionHandle) throws VcxException {

        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_get_their_pw_did(commandHandle, connectionHandle, vcxConnectionGetTheirPwDidCB);
        checkResult(result);

        return future;
    }

	private static Callback vcxConnectionInfoCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String info) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], info = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(info);
		}
	};

	/**
	 * Get the information about the established Connection.
	 * <p>
	 * Note: This method can be used for `aries` communication method only.
	 * For other communication method it returns ActionNotSupported error.
	 * 
	 * @param  connectionHandle     handle pointing to a Connection object.
	 *
	 * @return                      Connection Information as JSON string.
	 *                              {
	 *                                  "current": {
	 *                                      "did": string, - DID of current connection side
	 *                                      "recipientKeys": array[string], - Recipient keys
	 *                                      "routingKeys": array[string], - Routing keys
	 *                                      "serviceEndpoint": string, - Endpoint
	 *                                      "protocols": array[string], - The set of protocol supported by current side.
	 *                                  },
	 *                                  "remote: Optional({ - details about remote connection side
	 *                                      "did": string - DID of remote side
	 *                                      "recipientKeys": array[string] - Recipient keys of remote side
	 *                                      "routingKeys": array[string] - Routing keys of remote side
	 *                                      "serviceEndpoint": string - Endpoint of remote side
	 *                                      "protocols": array[string] - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
	 *                                  })
	 *                              }
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<String> connectionInfo(int connectionHandle) throws VcxException {
		logger.debug("connectionInfo() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_info(commandHandle, connectionHandle, vcxConnectionInfoCB);
		checkResult(result);
		return future;
	}

	/**
	 * Send a message to reuse existing Connection instead of setting up a new one as response on received Out-of-Band Invitation.
	 * <p>
	 * Note that this function works in case `aries` communication method is used.
	 * In other cases it returns ActionNotSupported error.
	 *
	 * @param  connectionHandle handle pointing to a Connection object to awaken and send reuse message.
	 * @param  invite           A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
	 *                          {
	 *                              "@type": "https://didcomm.org/out-of-band/%VER/invitation",
	 *                              "@id": "<id used for context as pthid>", -  the unique ID of the message.
	 *                              "label": Optional<string>, - a string that the receiver may want to display to the user,
	 *                                                          likely about who sent the out-of-band message.
	 *                              "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
	 *                                                              the user or use in automatically deciding what to do with the out-of-band message.
	 *                              "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
	 *                                                          about the context-specific goal of the out-of-band message.
	 *                              "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
	 *                                                                          that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
	 *                                                                          One or both of handshake_protocols and request~attach MUST be included in the message.
	 *                              "request~attach": Optional<[
	 *                                  {
	 *                                      "@id": "request-0",
	 *                                      "mime-type": "application/json",
	 *                                      "data": {
	 *                                          "json": "<json of protocol message>"
	 *                                      }
	 *                                  }
	 *                              ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
	 *                                  One or both of handshake_protocols and request~attach MUST be included in the message.
	 *                              "service": [
	 *                                  {
	 *                                      "id": string
	 *                                      "type": string,
	 *                                      "recipientKeys": [string],
	 *                                      "routingKeys": [string],
	 *                                      "serviceEndpoint": string
	 *                                  }
	 *                              ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
	 *                          }
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> connectionSendReuse(
			int connectionHandle,
			String invite
	) throws VcxException {
		ParamGuard.notNull(invite, "invite");

		logger.debug("connectionSendReuse() called with: connectionHandle = [" + connectionHandle + "], invite = [*****]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_reuse(commandHandle, connectionHandle, invite, voidCb);
		checkResult(result);

		return future;
	}

	/**
	 * Send answer on received question message according to Aries question-answer protocol.
	 * <p>
	 * Note that this function works in case `aries` communication method is used.
	 * In other cases it returns ActionNotSupported error.
	 *
	 * @param  connectionHandle handle pointing to a Connection object to send answer message.
	 * @param  question         A JSON string representing Question received via pairwise connection.
	 *                          {
	 *                              "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question",
	 *                              "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
	 *                              "question_text": "Alice, are you on the phone with Bob from Faber Bank right now?",
	 *                              "question_detail": "This is optional fine-print giving context to the question and its various answers.",
	 *                              "nonce": "<valid_nonce>",
	 *                              "signature_required": true,
	 *                              "valid_responses" : [
	 *                                  {"text": "Yes, it's me"},
	 *                                  {"text": "No, that's not me!"}],
	 *                              "~timing": {
	 *                                  "expires_time": "2018-12-13T17:29:06+0000"
	 *                              }
	 *                          }
	 * @param  answer           An answer to use which is a JSON string representing chosen `valid_response` option from Question message.
	 *                          {
	 *                              "text": "Yes, it's me"
	 *                          }
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> connectionSendAnswer(
			int connectionHandle,
			String question,
			String answer
	) throws VcxException {
		ParamGuard.notNull(question, "question");
		ParamGuard.notNull(answer, "answer");

		logger.debug("connectionSendAnswer() called with: connectionHandle = [" + connectionHandle + "],  question = [****],  answer = [****]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_answer(commandHandle, connectionHandle, question, answer, voidCb);
		checkResult(result);

		return future;
	}

	/**
	 * Send a message to invite another side to take a particular action.
	 * The action is represented as a `goal_code` and should be described in a way that can be automated.
	 * <p>
	 * The related protocol can be found here:
	 *     https://github.com/hyperledger/aries-rfcs/blob/ecf4090b591b1d424813b6468f5fc391bf7f495b/features/0547-invite-action-protocol
	 *
	 * @param  connectionHandle handle pointing to a Connection object to send answer message.
	 * @param  goalCode         A code the receiver may want to display to the user or use in automatically
	 *                          deciding what to do after receiving the message.
	 *
	 * @return                  void
	 *
	 * @throws VcxException     If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Void> connectionSendInviteAction(
			int connectionHandle,
			String goalCode
	) throws VcxException {
		ParamGuard.notNull(goalCode, "goalCode");

		logger.debug("connectionSendInviteAction() called with: connectionHandle = [" + connectionHandle + "],  goalCode = [****]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_invite_action(commandHandle, connectionHandle, goalCode, voidCb);
		checkResult(result);

		return future;
	}
}
