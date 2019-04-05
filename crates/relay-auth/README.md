# relay-auth

## Authorisation overview

In general, every connection attempt should start with an auth process to create an active session.

An active session is defined by a hash of (secret + timestamp + identifier).

A client can connect to the socket and then has two options:

- Recover an existing session by providing a session id
- Start a new session by providing a secret token

If a client connects and does not do one of these as the first operation, the server
will immediately disconnect it.

### session token

The session token is:

    expires:client_id:SHA256([expires]:[client_id]:[secret])

The only way to get a session token is by asking for one.

### secrets

Secrets are managed by the server instance, and statically distributed to the client.

Secrets can be revoked, but if so, clients can keep running until the session expires.

The server maintains a list of secrets, so you can roll-out and then retire old clients.

### workflow

- connection
- server starts a wait-idle disconnect workflow
- client issues either: recover session or start session

- start session 
  - check secret
  - reject client if invalid
  - generate session token
  - save session token
  - connect socket to a new isolate
  - return session token
  
- recover session
  - validate session token
  - reject client if invalid
  - look up session token
  - reject client if invalid
  - generate new session token
  - save new session token
  - discard old session token
  - connect socket to an existing isolate
  - return a new session token

Either way, the client ends up with an open socket to a valid isolate, or is disconnected.