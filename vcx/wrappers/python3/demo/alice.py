import asyncio
import json
from time import sleep

from demo.demo_utils import download_message, update_message_as_read
from vcx.api.connection import Connection
from vcx.api.credential import Credential
from vcx.api.disclosed_proof import DisclosedProof
from vcx.api.utils import vcx_agent_provision
from vcx.api.vcx_init import vcx_init_with_config
from vcx.state import State
from vc_auth_oidc.alice_vc_auth import handle_challenge


# logging.basicConfig(level=logging.DEBUG) uncomment to get logs

provisionConfig = {
    'agency_url': 'https://agency-team1.pdev.evernym.com',
    'agency_did': 'TGLBMTcW9fHdkSqown9jD8',
    'agency_verkey': 'FKGV9jKvorzKPtPJPNLZkYPkLhiS1VbxdvBgd1RjcQHR',
    'wallet_name': 'alice_wallet',
    'wallet_key': '123',
    'payment_method': 'null',
    'enterprise_seed': '000000000000000000000000Trustee1',
    'protocol_type': '1.0',
}


async def main():
    await init()
    connection_to_faber = None
    while True:
        answer = input(
            "Would you like to do? \n "
            "0 - establish connection \n "
            "1 - check for credential offer \n "
            "2 - check for proof request \n "
            "3 - pass vc_auth_oidc-challenge \n "
            "4 - check for questions \n "
            "5 - establish out-of-band connection \n "
            "6 - check for messages\n "
            "else finish \n") \
            .lower().strip()
        if answer == '0':
            connection_to_faber = await connect()
        elif answer == '1':
            print("Check agency for a credential offer")
            offers = await Credential.get_offers(connection_to_faber)
            print("Offer: " + json.dumps(offers[0]))
            credential = await Credential.create('credential', offers[0])
            accept_offer_answer = input(
                "Would you like to accept offer? \n "
                "0 - accept \n "
                "1 - reject \n "
                "else finish \n") \
                .lower().strip()
            if accept_offer_answer == '0':
                await accept_offer(connection_to_faber, credential)
            elif accept_offer_answer == '1':
                await reject_offer(connection_to_faber, credential)
            else:
                break
        elif answer == '2':
            print("Check agency for a proof request")
            requests = await DisclosedProof.get_requests(connection_to_faber)
            print("#23 Create a Disclosed proof object from proof request")
            print("Proof Request: " + json.dumps(requests[0]))
            proof = await DisclosedProof.create('proof', requests[0])
            accept_proof_answer = input(
                "Would you like to accept proof? \n "
                "0 - accept \n "
                "1 - reject \n "
                "else finish \n") \
                .lower().strip()
            if accept_proof_answer == '0':
                await create_proof(connection_to_faber, proof)
            elif accept_proof_answer == '1':
                await reject_proof(connection_to_faber, proof)
            else:
                break
        elif answer == '3':
            request = await handle_challenge()
            print("#23 Create a Disclosed proof object from proof request")
            proof = await DisclosedProof.create('proof', request)
            await create_proof(None, proof)
        elif answer == '4':
            print("Check agency for a questions")
            pw_did = await connection_to_faber.get_my_pw_did()
            uid, question, _ = await download_message(pw_did, 'question')
            question = json.loads(question)
            answer = question['valid_responses'][0]
            await connection_to_faber.send_answer(json.dumps(question), json.dumps(answer))
            await update_message_as_read(pw_did, uid)
        elif answer == '5':
            connection_to_faber = await outofband_connect()
        elif answer == '6':
            pw_did = await connection_to_faber.get_my_pw_did()
            uid, question, _ = await download_message(pw_did, None)
        else:
            break

    print("Finished")


async def init():
    print("#7 Provision an agent and wallet, get back configuration details")
    config = await vcx_agent_provision(json.dumps(provisionConfig))
    config = json.loads(config)
    # Set some additional configuration options specific to alice
    config['institution_name'] = 'alice'
    config['institution_logo_url'] = 'http://robohash.org/456'
    config['genesis_path'] = 'docker.txn'
    config['payment_method'] = 'null'

    config = json.dumps(config)

    print("#8 Initialize libvcx with new configuration")
    await vcx_init_with_config(config)


async def connect():
    print("#9 Input faber.py invitation details")
    details = input('invite details: ')

    print("#10 Convert to valid json and string and create a connection to faber")
    jdetails = json.loads(details)
    connection_to_faber = await Connection.accept_connection_invite('faber', json.dumps(jdetails))
    connection_state = await connection_to_faber.update_state()
    while connection_state != State.Accepted:
        sleep(2)
        await connection_to_faber.update_state()
        connection_state = await connection_to_faber.get_state()

    print("Connection is established")
    return connection_to_faber


async def outofband_connect():
    print("#9 Input faber.py invitation details")
    details = input('invite details: ')

    print("#10 Convert to valid json and string and create a connection to faber")
    jdetails = json.loads(details)
    connection_to_faber = await Connection.create_with_outofband_invite('faber', json.dumps(jdetails))
    await connection_to_faber.connect('{"use_public_did": true}')
    connection_state = await connection_to_faber.update_state()
    while connection_state != State.Accepted:
        sleep(2)
        await connection_to_faber.update_state()
        connection_state = await connection_to_faber.get_state()

    print("Connection is established")
    return connection_to_faber


async def accept_offer(connection_to_faber, credential):
    print("#15 After receiving credential offer, send credential request")
    await credential.send_request(connection_to_faber, 0)

    print("#16 Poll agency and accept credential offer from faber")
    credential_state = await credential.get_state()
    while credential_state != State.Accepted:
        sleep(2)
        await credential.update_state()
        credential_state = await credential.get_state()


async def reject_offer(connection_to_faber, credential):
    print("#15 Reject credential offer")
    await credential.reject(connection_to_faber)

    print("#16 Check credential offer state")
    credential_state = await credential.get_state()
    assert credential_state == State.Rejected


async def create_proof(connection_to_faber, proof):
    print("#24 Query for credentials in the wallet that satisfy the proof request")
    credentials = await proof.get_creds()

    # Use the first available credentials to satisfy the proof request
    for attr in credentials['attrs']:
        credentials['attrs'][attr] = {
            'credential': credentials['attrs'][attr][0]
        }

    print("#25 Generate the proof")
    await proof.generate_proof(credentials, {})

    print("#26 Send the proof")
    await proof.send_proof(connection_to_faber)

    proof_state = await proof.get_state()
    while proof_state != State.Accepted:
        sleep(2)
        await proof.update_state()
        proof_state = await proof.get_state()

    print("proof is verified!!")


async def reject_proof(connection_to_faber, proof):
    print("#15 Reject proof request")
    await proof.reject_proof(connection_to_faber)

    print("#16 Check proof request state")
    proof_state = await proof.get_state()
    assert proof_state == State.Rejected


if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    sleep(1)
