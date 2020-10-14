# Running the Alice/Faber Python demo

There are 2 versions of this demo, the original version (faber.py and alice.py), and a slightly modified version (faber-pg.py and alice-pg.py) that supports postgres wallet storage, and illustrated how the vcx object serializtion/deserialization can be used.

## Original Demo

This demo consists of 3 files:

faber*.py - scripts that acts as an institution/enterprise by initiating connection, writing a schema/cred_def to the ledger, sending a credential, and requesting proof.  

alice*.py - a script that acts as an individual by accepting a connection offer, requesting a credential and offering proof.

pool.txn - genesis files for connecting to an indy pool (existing file connects to libindy/sovtoken ledger)

To run these follow the next steps:

1.  Install `libindy` and `libnullpay` libraries:
     * Ubuntu - https://github.com/hyperledger/indy-sdk#ubuntu-based-distributions-ubuntu-1604-and-1804
     * Windows - https://github.com/hyperledger/indy-sdk#windows
     * MacOS - https://github.com/hyperledger/indy-sdk#macos 
        * Setup dependencies:
        ```
        sh "brew switch libsodium 1.0.12"
        sh "brew switch openssl 1.0.2q"
        sh "brew switch zeromq 4.2.3"
       ```
       * Instead of setting environment variable as described in the instruction above we can can copy 
       `libindy.dylib` and `libnullpay.dylib` libraries to `/usr/lib` or `usr/local/lib`.
       * If we can't install `libindy` dependencies of specified versions we need to build binaries ourselves.
       ```
       // from the top of repository
       cd libindy && cargo build && target/debug/libindy.so /usr/lib 
       // from the top of repository
       cd libnullpay && cargo build && target/debug/libnullpay.so /usr/lib 
       ```
2.  Install or build `libvcx`:
    * Prepared binaries are available for Ubuntu only:
        * bionic:      https://repo.corp.evernym.com/portal/dev/
        * xenial:      https://repo.corp.evernym.com/portal/dev/
    * Manual building:    
       ```
       // from the top of repository
       cd vcx/libvcx && cargo build && target/debug/libvcx.so /usr/lib 
      ```
      
3. Install the python requirements: 
    * `pip install -r requirements.txt`
    * Install `vcx` python wrapper from sources: `https://repo.corp.evernym.com/portal/dev/`
    **or**
    Set up environment variable to use sources from `vcx/wrappers/python3/vcx`.
4. `Devteam1` environment is used by default. 
In order to use different environment you need to change agency related fields in `provisionConfig` variable 
and change genesis transactions in `docker.txn` file.
Note: you need to delete `~/.indy_client` folder after changing `docker.txn` file.

5. Run scripts with `python3 faber.py` and `python3 alice.py`.

## Scripts

* Faber:
    * faber.py - base script representing Inviter / Issuer / Verifier sides.
    * faber-pg.py - same as faber.py but using Postgres as a wallet storage.
    * faber_commitedanswer_1.0.py - script to test commited-answer protocol for proprietary connection.
    * faber_commitedanswer_3.0.py - script to test commited-answer protocol for aries connection.
    * faber_question_3.0.py - script to test question-answer protocol for aries connection.
    * faber_credential_with_attachment.py - script to test credentials with attachments.
    * faber_redirect_1.0.py - script to test connection redirection for proprietary connection.
* Alice:
    * alice.py - base script representing Invitee / Holder / Prover sides.
    * alice_create_with_message_flow.py - similar to alice.py but used different functions for creating state objects.
    * alice_create_with_message_id_flow.py - similar to alice.py but used different functions for creating state objects.
    * alternate_wallet_path.py - similar to alice.py but used custom wallet storage.

