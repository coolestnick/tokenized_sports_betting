# Tokenized Sports Betting

The Tokenized Sports Betting platform is a blockchain-based system that enables users to place bets on various sports events using tokenized assets. This platform ensures transparency, security, and fairness by leveraging smart contracts on the blockchain. Users can place bets, view bet history, manage their accounts, and claim rewards, all within a decentralized and trustless environment.

## Features

### 1. Bet Management

- Create, read, update, and delete bets.
- Each bet includes fields for event details, betting odds, amount, user ID, and bet status.

### 2. User Management

- Create, read, update, and delete user information.
- Each user includes fields for username, balance, and bet history.

### 3. Event Management

- Create, read, update, and delete sports events.
- Each event includes fields for event details, participants, odds, and event status.

### 4. Bet History

- Maintain a history of bets placed by each user, including timestamps and bet details.

### 5. Input Validation

- Validate input data to ensure required fields are provided and follow the expected format.
- Prevent invalid data from being entered into the system, improving data integrity and accuracy.

### 6. Error Handling

- Provide error handling mechanisms to gracefully handle unexpected errors or invalid requests.
- Return appropriate error messages and status codes to the client to aid in debugging and troubleshooting.

### Requirements

- rustc 1.64 or higher

```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```

- rust wasm32-unknown-unknown target

```bash
$ rustup target add wasm32-unknown-unknown
```

- candid-extractor

```bash
$ cargo install candid-extractor
```

- install `dfx`

```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background --clean
```

## API Functions

### Handling Users

1. dfx canister call tokenized_sports_betting add_user '(record{username="John";balance=10;})'
2. dfx canister call tokenized_sports_betting get_user '(0)'
3. dfx canister call tokenized_sports_betting update_user '(0,record{username="John";balance=1000;})'

### Handling Events

1. dfx canister call tokenized_sports_betting add_event '(record{name="Event 1";participants=vec{"John"};odds=vec{1.76};status=variant{Upcoming};})'
2. dfx canister call tokenized_sports_betting get_event '(1)'
3. dfx canister call tokenized_sports_betting update_event '(1,record{name="John Event";participants=vec{"John"};odds=vec{2.43};status=variant{Ongoing};})'

### Handling Bets

1. dfx canister call tokenized_sports_betting add_bet '(record {user_id=0;event_id=1;amount=10000;odds=1.76;})'
2. dfx canister call tokenized_sports_betting get_bet '(2)'
3. dfx canister call tokenized_sports_betting update_bet_status '(2,variant{Cancelled})'

### Deleting everything

1. dfx canister call tokenized_sports_betting delete_bet '(2)'
2. dfx canister call tokenized_sports_betting delete_event '(1)'
3. dfx canister call tokenized_sports_betting delete_user '(0)'
