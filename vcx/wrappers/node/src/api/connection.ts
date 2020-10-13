import * as ffi from 'ffi'
import * as ref from 'ref'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { ISerializedData, StateType } from './common'
import { VCXBaseWithState } from './vcx-base-with-state'

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
 *                                            2) once `vcx_connection_create_with_outofband_invitation`
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
 *
 *           VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
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

/**
 * @description Interface that represents the attributes of a Connection object.
 * This data is expected as the type for deserialize's parameter and serialize's return value
 * @interface
 */
export interface IConnectionData {
  source_id: string
  invite_detail: string,
  handle: number,
  pw_did: string,
  pw_verkey: string,
  did_endpoint: string,
  endpoint: string,
  uuid: string,
  wallet: string,
  state: StateType
}

/**
 * @description Interface that represents the parameters for `Connection.create` function.
 * @interface
 */
export interface IConnectionCreateData {
  // Institution's personal identification for the connection
  id: string
}

/**
 * @description Interface that represents the parameters for `Connection.createOutofband` function.
 * WARN: `requestAttach` field is not fully supported in the current library state.
 *        You can use simple messages like Question but it cannot be used
 *        for Credential Issuance and Credential Presentation.
 * @interface
 */
export interface IConnectionCreateOutofbandData {
  // Institution's personal identification for the connection
  id: string,
  // a self-attested code the receiver may want to display to
  // the user or use in automatically deciding what to do with the out-of-band message.
  goalCode?: string,
  // a self-attested string that the receiver may want to display to the user about
  // the context-specific goal of the out-of-band message.
  goal?: string,
  // whether Inviter wants to establish regular connection using `connections` handshake protocol.
  // if false, one-time connection channel will be created.
  handshake: boolean,
  // An additional message as JSON that will be put into attachment decorator
  // that the receiver can using in responding to the message (for example Question message).
  requestAttach?: string,
}

// A string representing a invitation json object.
export type IConnectionInvite = string

/**
 * A string representing a out-of-band invitation json object.
 *     {
 *         "@type": "https://didcomm.org/out-of-band/%VER/invitation",
 *         "@id": "<id used for context as pthid>", -  the unique ID of the message.
 *         "label": Optional<string>, - a string that the receiver may want to display to the user,
 *                                      likely about who sent the out-of-band message.
 *         "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
 *                                          the user or use in automatically deciding what 
 *                                          to do with the out-of-band message.
 *         "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
 *                                     about the context-specific goal of the out-of-band message.
 *         "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
 *                                                     that the receiver can use in responding to the message
 *                                                     in order to create or reuse a connection with the sender.
 *                                                     One or both of handshake_protocols and request~attach
 *                                                     MUST be included in the message.
 *         "request~attach": Optional<[
 *             {
 *                 "@id": "request-0",
 *                 "mime-type": "application/json",
 *                 "data": {
 *                     "json": "<json of protocol message>"
 *                 }
 *             }
 *         ]>, - an attachment decorator containing an array of request messages in order of preference
 *               that the receiver can using in responding to the message.
 *               One or both of handshake_protocols and request~attach MUST be included in the message.
 *         "service": [
 *             {
 *                 "id": string
 *                 "type": string,
 *                 "recipientKeys": [string],
 *                 "routingKeys": [string],
 *                 "serviceEndpoint": string
 *             }
 *         ] - an item that is the equivalent of the service block of a DIDDoc 
 *             that the receiver is to use in responding to the message.
 *     }
 */
export type IConnectionOutofbandInvite = string

/**
 * @description Interface that represents the parameters for `Connection.createWithInvite` function.
 * @interface
 */
export interface IRecipientInviteInfo extends IConnectionCreateData {
  // Invitation provided by an entity that wishes to make a connection.
  invite: IConnectionInvite
}

/**
 * @description Interface that represents the parameters for `Connection.createWithOutofbandInvite` function.
 * @interface
 */
export interface IRecipientOutofbandInviteInfo extends IConnectionCreateData {
  // Out-of-Band Invitation provided by an entity that wishes interaction.
  invite: IConnectionOutofbandInvite
}

/**
 * @description Interface that represents the parameters for `Connection.connect` function.
 * @interface
 */
export interface IConnectOptions {
  /**
  * Provides details about establishing connection
  *      {
  *         "connection_type": Option<"string"> - one of "SMS", "QR",
  *        "phone": "string": Option<"string"> - phone number in case "connection_type" is set into "SMS",
  *        "update_agent_info": Option<bool> - whether agent information needs to be updated.
  *                                             default value for `update_agent_info`=true
  *                                             if agent info does not need to be updated, set `update_agent_info`=false
  *        "use_public_did": Option<bool> - whether to use public DID for an establishing connection
  *                                         default value for `use_public_did`=false
  *    }
  */
  data: string
}

/**
 * @description Interface that represents the parameters for `Connection.acceptConnectionInvite` function.
 * @interface
 */
export interface IAcceptInviteInfo extends IRecipientInviteInfo, IConnectOptions {
}

/**
 * @description Interface that represents the parameters for `Connection.sendMessage` function.
 * @interface
 */
export interface IMessageData {
  // Actual message to send
  msg: string,
  // Type of message to send. Can be any string
  type: string,
  // Message title (user notification)
  title: string,
  // If responding to a message, id of the message
  refMsgId?: string,
}

/**
 * @description Interface that represents the parameters for `Connection.verifySignature` function.
 * @interface
 */
export interface ISignatureData {
  // Message was signed
  data: Buffer,
  // Generated signature
  signature: Buffer
}

/**
 * @description A string representing a connection info json object.
 *      {
 *         "current": {
 *             "did": <str>
 *             "recipientKeys": array<str>
 *             "routingKeys": array<str>
 *             "serviceEndpoint": <str>,
 *             "protocols": array<str> -  The set of protocol supported by current side.
 *         },
 *         "remote: { <Option> - details about remote connection side
 *             "did": <str> - DID of remote side
 *             "recipientKeys": array<str> - Recipient keys
 *             "routingKeys": array<str> - Routing keys
 *             "serviceEndpoint": <str> - Endpoint
 *             "protocols": array<str> - The set of protocol supported by side.
 *                                       Is filled after DiscoveryFeatures process was completed.
 *          }
 *    }
 */
export type IConnectionInfo = string

/**
 * @description Interface that represents the parameters for `Connection.sendAnswer` function.
 * @interface
 */
export interface IConnectionAnswerData {
  // A JSON string representing Question received via pairwise connection.
  question: object,
  // An answer to use which is a JSON string representing chosen `valid_response` option from Question message.
  answer: object,
}

/**
 * @description Interface that represents the parameters for `Connection.sendInviteAction` function.
 * @interface
 */
export interface IConnectionInviteActionData {
  // A code the receiver may want to display to the user or use in automatically deciding what to do after receiving the message.
  goalCode: object,
}

export function voidPtrToUint8Array (origPtr: any, length: number): Buffer {
  /**
   * Read the contents of the pointer and copy it into a new Buffer
   */
  const ptrType = ref.refType('uint8 *')
  const pointerBuf = ref.alloc(ptrType, origPtr)
  const newPtr = ref.readPointer(pointerBuf, 0, length)
  const newBuffer = Buffer.from(newPtr)
  return newBuffer
}
/**
 * @class Class representing a Connection
 */
export class Connection extends VCXBaseWithState<IConnectionData> {
  /**
   * Create a connection object, represents a single endpoint and can be used for sending and receiving
   * credentials and proofs
   *
   * Example:
   * ```
   * source_id = 'foobar123'
   * connection = await Connection.create(source_id)
   * ```
   */
  public static async create ({ id }: IConnectionCreateData): Promise<Connection> {
    try {
      const connection = new Connection(id)
      const commandHandle = 0
      await connection._create((cb) => rustAPI().vcx_connection_create(commandHandle, id, cb))
      return connection
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Create a Connection object that provides an Out-of-Band Connection for an institution's user.
   *
   * NOTE: this method can be used when `aries` protocol is set.
   *
   * NOTE: this method is EXPERIMENTAL
   *
   * Example:
   * ```
   * const data = {
   *  id: 'foobar123',
   *  goal: 'Foo Goal',
   *  handshake: true,
   * }
   * connection = await Connection.createOutofband(data)
   * ```
   */
  public static async createOutofband ({ id, goalCode, goal, handshake, requestAttach }:
    IConnectionCreateOutofbandData): Promise<Connection> {
    try {
      const connection = new Connection(id)
      const commandHandle = 0
      await connection._create((cb) => rustAPI().vcx_connection_create_outofband(
        commandHandle, id, goalCode, goal, handshake, requestAttach, cb))
      return connection
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Create a connection object with a provided invite, represents a single endpoint and can be used for
   * sending and receiving credentials and proofs.
   * Invite details are provided by the entity offering a connection and generally pulled from a provided QRCode.
   *
   * Example:
   * ```
   * sourceId = 'foobar123'
   * connection_handle = await Connection.createWithInvite({sourceId, inviteDetails})
   * ```
   */
  public static async createWithInvite ({ id, invite }: IRecipientInviteInfo): Promise<Connection> {
    const connection = new Connection(id)
    const commandHandle = 0
    try {
      await connection._create((cb) => rustAPI().vcx_connection_create_with_invite(commandHandle,
                                                 id, invite, cb))

      return connection
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

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
   * Example:
   * ```
   * sourceId = 'foobar123'
   * connection_handle = await Connection.createWithOutofbandInvite({sourceId, invite})
   * ```
   */
  public static async createWithOutofbandInvite ({ id, invite }: IRecipientOutofbandInviteInfo): Promise<Connection> {
    const connection = new Connection(id)
    const commandHandle = 0
    try {
      await connection._create((cb) => rustAPI().vcx_connection_create_with_outofband_invitation(commandHandle,
                                                 id, invite, cb))
      return connection
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Accept connection for the given invitation.
   *
   * This function performs the following actions:
   * 1. Creates Connection state object from the given invitation
   *     (equal to `Connection.createWithInvite` function).
   * 2. Replies to the inviting side
   *     (equal to `Connection.connect` function).
   * Example:
   * id = 'foobar123'
   * data = '{"connection_type":"SMS","phone":"5555555555"}'
   * connection2 = await Connection.acceptConnectionInvite({id, invite, data})
   */
  public static async acceptConnectionInvite ({ id, invite, data }: IAcceptInviteInfo): Promise<Connection> {
    try {
      return await createFFICallbackPromise<Connection>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_accept_connection_invite(0, id, invite, data, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32', 'string'],
          (handle: number, err: any, connectionHandle: number, connectionSerialized: string) => {
            if (err) {
              reject(err)
              return
            }
            const connection = new Connection(id)
            connection._setHandle(connectionHandle)
            connection._serialized = connectionSerialized
            resolve(connection)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Create the object from a previously serialized object.
   * Example:
   * data = await connection1.serialize()
   * connection2 = await Connection.deserialize(data)
   */
  public static async deserialize (connectionData: ISerializedData<IConnectionData>) {
    const connection = await super._deserialize(Connection, connectionData)
    return connection
  }

  protected _releaseFn = rustAPI().vcx_connection_release
  protected _updateStFn = rustAPI().vcx_connection_update_state
  protected _updateStWithMessageFn = rustAPI().vcx_connection_update_state_with_message
  protected _getStFn = rustAPI().vcx_connection_get_state
  protected _serializeFn = rustAPI().vcx_connection_serialize
  protected _deserializeFn = rustAPI().vcx_connection_deserialize
  protected _inviteDetailFn = rustAPI().vcx_connection_invite_details
  protected _infoFn = rustAPI().vcx_connection_info
  protected _serialized: string = ''

  get serialized (): string {
    return this._serialized
  }

  /**
   *
   * Updates the state of the connection from the given message.
   *
   * Example:
   * ```
   * await object.updateStateWithMessage(message)
   * ```
   * @returns {Promise<void>}
   */
  public async updateStateWithMessage (message: string): Promise<void> {
    try {
      const commandHandle = 0
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_update_state_with_message(commandHandle, this.handle, message, cb)
          if (rc) {
            resolve(StateType.None)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (handle: number, err: any, state: StateType) => {
            if (err) {
              reject(err)
            }
            resolve(state)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Delete the object from the agency and release any memory associated with it
   * NOTE: This eliminates the connection and any ability to use it for any communication.
   *
   * Example:
   * ```
   * def connection = await Connection.create(source_id)
   * await connection.delete()
   * ```
   */
  public async delete (): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_delete_connection(0, this.handle, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32'],
          (xcommandHandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Creates a connection between enterprise and end user.
   *
   * Example:
   * ```
   * connection = await Connection.create('foobar123')
   * inviteDetails = await connection.connect(
   *     {data: '{"connection_type":"SMS","phone":"5555555555", "use_public_did":true}'})
   * ```
   * @returns {Promise<string}
   */
  public async connect (connectionData: IConnectOptions): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_connect(0, this.handle, connectionData.data, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, details: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!details) {
                reject(`Connection ${this.sourceId} connect returned empty string`)
                return
              }
              resolve(details)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Sends a message to the connection.
   *
   * Example:
   * ```
   * msg_id = await connection.send_message(
   *     {msg:"are you there?",type:"question","title":"Sending you a question"})
   * ```
   * @returns {Promise<string>} Promise of String representing UID of created message in 1.0 VCX protocol. When using
   * 2.0 / 3.0 / Aries protocol, return empty string.
   */
  public async sendMessage (msgData: IMessageData): Promise<string> {
    const sendMsgOptions = {
      msg_title: msgData.title,
      msg_type: msgData.type,
      ref_msg_id: msgData.refMsgId
    }
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_send_message(0, this.handle,
              msgData.msg, JSON.stringify(sendMsgOptions), cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, details: string) => {
              if (err) {
                reject(err)
                return
              }
              resolve(details)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Sign data using connection pairwise key.
   *
   * Example:
   * ```
   * signature = await connection.signData(bufferOfBits)
   * ```
   * @returns {Promise<string}
   */
  public async signData (data: Buffer): Promise<Buffer> {
    try {
      return await createFFICallbackPromise<Buffer>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_sign_data(0, this.handle,
              ref.address(data), data.length, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'pointer', 'uint32'],
            (xHandle: number, err: number, details: any, length: number) => {
              if (err) {
                reject(err)
                return
              }
              if (!details) {
                reject(`Connection ${this.sourceId}  returned empty buffer`)
                return
              }
              const newBuffer = voidPtrToUint8Array(details, length)
              resolve(newBuffer)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Verify the signature of the data using connection pairwise key.
   *
   * Example:
   * ```
   * valid = await connection.verifySignature({data: bufferOfBits, signature: signatureBits})
   * ```
   * @returns {Promise<string}
   */
  public async verifySignature (signatureData: ISignatureData): Promise<boolean> {
    try {
      return await createFFICallbackPromise<boolean>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_verify_signature(0, this.handle,
              ref.address(signatureData.data), signatureData.data.length,
              ref.address(signatureData.signature), signatureData.signature.length, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'bool'],
            (xHandle: number, err: number, valid: boolean) => {
              if (err) {
                reject(err)
                return
              }
              resolve(valid)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Get the invite details that were sent or can be sent to the remote side.
   *
   * Example:
   * ```
   * phoneNumber = '8019119191'
   * connection = await Connection.create('foobar123')
   * inviteDetails = await connection.connect({phone: phoneNumber})
   * inviteDetailsAgain = await connection.inviteDetails()
   * ```
   */
  public async inviteDetails (abbr: boolean = false): Promise<IConnectionInvite> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = this._inviteDetailFn(0, this.handle, abbr, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'string'],
          (handle: number, err: number, details: string) => {
            if (err) {
              reject(err)
              return
            }
            if (!details) {
              reject('no details returned')
              return
            }
            resolve(details)
          })
      )
      return data
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
   *
   * Note that this function is useful in case `aries` communication method is used.
   * In other cases it returns ActionNotSupported error.
   *
   */
  public async sendPing (comment: string | null | undefined): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_send_ping(0, this.handle, comment, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Send discovery features message to the specified connection to discover 
   * which features it supports, and to what extent.
   *
   * Note that this function is useful in case `aries` communication method is used.
   * In other cases it returns ActionNotSupported error.
   *
   */
  public async sendDiscoveryFeatures (query: string | null | undefined,
                                      comment: string | null | undefined): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_send_discovery_features(0, this.handle, query, comment, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Send a message to reuse existing Connection instead of setting up a new one
   * as response on received Out-of-Band Invitation.
   *
   * Note that this function works in case `aries` communication method is used.
   *     In other cases it returns ActionNotSupported error.
   *
   * Example:
   * ```
   * await connection.sendReuse(invite)
   * ```
   */
  public async sendReuse (invite: IConnectionOutofbandInvite): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_send_reuse(0, this.handle, invite, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Send answer on received question message according to Aries question-answer protocol.
   *
   * Note that this function works in case `aries` communication method is used.
   *     In other cases it returns ActionNotSupported error.
   *
   * Example:
   * ```
   * const data = {
   *   question: {
   *     "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question",
   *     "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
   *     "question_text": "Alice, are you on the phone with Bob from Faber Bank right now?",
   *     "valid_responses" : [
   *             {"text": "Yes, it's me"},
   *             {"text": "No, that's not me!"}
   *     ],
   *     "~timing": {
   *             "expires_time": "2018-12-13T17:29:06+0000"
   *     }
   *   },
   *   answer: {
   *    "text": "Yes, it's me"
   *   }
   * }
   * await connection.sendAnswer(invite)
   * ```
   */
  public async sendAnswer (data: IConnectionAnswerData): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_send_answer(0, this.handle,
            JSON.stringify(data.question), JSON.stringify(data.answer), cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Send a message to invite another side to take a particular action.
   * The action is represented as a `goal_code` and should be described in a way that can be automated.
   *
   * The related protocol can be found here:
   *     https://github.com/hyperledger/aries-rfcs/blob/ecf4090b591b1d424813b6468f5fc391bf7f495b/features/0547-invite-action-protocol
   *
   * Example:
   * ```
   * await connection.sendInviteAction('automotive.inspect.tire')
   * ```
   */
  public async sendInviteAction (data: IConnectionInviteActionData): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_send_invite_action(0, this.handle, data.goalCode, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Retrieves pw_did from Connection object
   *
   */
  public async getPwDid (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_get_pw_did(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, details: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!details) {
                reject(`Connection ${this.sourceId} connect returned empty string`)
                return
              }
              resolve(details)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Retrieves their_pw_did from Connection object
   *
   */
  public async getTheirDid (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_get_their_pw_did(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, details: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!details) {
                reject(`Connection ${this.sourceId} connect returned empty string`)
                return
              }
              resolve(details)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Redirects to an existing connection if one already present.
   *
   * Example:
   * ```
   * const oldConnectionToAcme = searchConnectionsByPublicDID({
   *  public_did: inviteDetails.publicDID
   * })
   * const redirectConnectionToAcme = await Connection.createWithInvite({
   *  id: 'faber-redirect',
   *  invite: JSON.stringify(inviteDetails)
   * })
   * await redirectConnectionToAcme.redirect({
   *  redirectToConnection: oldConnectionToAcme
   * })
   * ```
   */
  public async connectionRedirect (existingConnection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_connection_redirect(0, this.handle, existingConnection.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32'],
            (xcommandHandle: number, err: number) => {
              if (err) {
                reject(err)
                return
              }
              resolve()
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

/**
 * Gets the redirection details if the connection already exists.
 *
 * Example:
 * ```
 * await connectionToAlice.updateState()
 * connectionState = await connectionToAlice.getState()
 *
 * if (connectionState == StateType.Redirected) {
 * redirectDetails = await connectionToAlice.getRedirectDetails()
 * serializedOldConnection = searchConnectionsByTheirDid({
 *   theirDid: redirectDetails.theirDID
 * })
 * oldConnection = await Connection.deserialize({
 *   connectionData: serializedOldConnection
 * })
 *}
 * ```
 */
  public async getRedirectDetails (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_connection_get_redirect_details(
            0,
            this.handle,
            cb
          )
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, details: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!details) {
                reject(`proof ${this.sourceId} returned empty string`)
                return
              }
              resolve(details)
            }
          )
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Get the information about the connection state.
   *
   * Note: This method can be used for `aries` communication method only.
   *     For other communication method it returns ActionNotSupported error.
   *
   */
  public async info (): Promise<IConnectionInfo> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = this._infoFn(0, this.handle, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'string'],
          (handle: number, err: number, info: string) => {
            if (err) {
              reject(err)
              return
            }
            if (!info) {
              reject('no info returned')
              return
            }
            resolve(info)
          })
      )
      return data
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
}
