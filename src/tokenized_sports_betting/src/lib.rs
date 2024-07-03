#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Bet {
    id: u64,
    user_id: u64,
    event_id: u64,
    amount: u64,
    odds: f64,
    status: BetStatus,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum BetStatus {
    Pending,
    Won,
    Lost,
    Cancelled,
}

impl Default for BetStatus {
    fn default() -> Self {
        BetStatus::Pending
    }
}

// Implement Storable for Bet
impl Storable for Bet {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable for Bet
impl BoundedStorable for Bet {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static BET_STORAGE: RefCell<StableBTreeMap<u64, Bet, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static EVENT_STORAGE: RefCell<StableBTreeMap<u64, Event, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    username: String,
    balance: u64,
    bet_history: Vec<u64>,
}

// Implement Storable for User
impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable for User
impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Event {
    id: u64,
    name: String,
    participants: Vec<String>,
    odds: Vec<f64>,
    status: EventStatus,
}

// Implement Storable for Event
impl Storable for Event {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable for Event
impl BoundedStorable for Event {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum EventStatus {
    Upcoming,
    Ongoing,
    Completed,
    Cancelled,
}

impl Default for EventStatus {
    fn default() -> Self {
        EventStatus::Upcoming
    }
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct BetPayload {
    user_id: u64,
    event_id: u64,
    amount: u64,
    odds: f64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct UserPayload {
    username: String,
    balance: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct EventPayload {
    name: String,
    participants: Vec<String>,
    odds: Vec<f64>,
    status: EventStatus,
}

#[ic_cdk::update]
fn add_bet(payload: BetPayload) -> Result<Bet, Error> {
    validate_bet_payload(&payload)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let bet = Bet {
        id,
        user_id: payload.user_id,
        event_id: payload.event_id,
        amount: payload.amount,
        odds: payload.odds,
        status: BetStatus::Pending,
        created_at: time(),
        updated_at: None,
    };

    do_insert_bet(&bet);

    let mut user_opt = USER_STORAGE.with(|storage| storage.borrow().get(&payload.user_id));

    if let Some(mut user) = user_opt {
        user.bet_history.push(bet.id);
        USER_STORAGE.with(|storage| storage.borrow_mut().insert(user.id, user));
    } else {
        return Err(Error::NotFound {
            msg: format!("User with id={} not found", payload.user_id),
        });
    }

    Ok(bet)
}

#[ic_cdk::query]
fn get_bet(id: u64) -> Result<Bet, Error> {
    match BET_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(bet) => Ok(bet),
        None => Err(Error::NotFound {
            msg: format!("A bet with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_bet_status(id: u64, status: BetStatus) -> Result<Bet, Error> {
    match BET_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut bet) => {
            bet.status = status;
            bet.updated_at = Some(time());
            do_insert_bet(&bet);
            Ok(bet)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't update a bet with id={}. Bet not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_bet(id: u64) -> Result<Bet, Error> {
    match BET_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(bet) => Ok(bet),
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't delete a bet with id={}. Bet not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn add_user(payload: UserPayload) -> Result<User, Error> {
    validate_user_payload(&payload)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let user = User {
        id,
        username: payload.username,
        balance: payload.balance,
        bet_history: Vec::new(),
    };

    USER_STORAGE.with(|service| service.borrow_mut().insert(user.id, user.clone()));
    Ok(user)
}

#[ic_cdk::query]
fn get_user(id: u64) -> Result<User, Error> {
    match USER_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!("A user with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_user(id: u64, payload: UserPayload) -> Result<User, Error> {
    validate_user_payload(&payload)?;

    match USER_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut user) => {
            user.username = payload.username;
            user.balance = payload.balance;
            USER_STORAGE.with(|service| service.borrow_mut().insert(user.id, user.clone()));
            Ok(user)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't update a user with id={}. User not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_user(id: u64) -> Result<User, Error> {
    match USER_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't delete a user with id={}. User not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn add_event(payload: EventPayload) -> Result<Event, Error> {
    validate_event_payload(&payload)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let event = Event {
        id,
        name: payload.name,
        participants: payload.participants,
        odds: payload.odds,
        status: payload.status,
    };

    EVENT_STORAGE.with(|service| service.borrow_mut().insert(event.id, event.clone()));
    Ok(event)
}

#[ic_cdk::query]
fn get_event(id: u64) -> Result<Event, Error> {
    match EVENT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(event) => Ok(event),
        None => Err(Error::NotFound {
            msg: format!("An event with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_event(id: u64, payload: EventPayload) -> Result<Event, Error> {
    validate_event_payload(&payload)?;

    match EVENT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut event) => {
            event.name = payload.name;
            event.participants = payload.participants;
            event.odds = payload.odds;
            event.status = payload.status;
            EVENT_STORAGE.with(|service| service.borrow_mut().insert(event.id, event.clone()));
            Ok(event)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't update an event with id={}. Event not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_event(id: u64) -> Result<Event, Error> {
    match EVENT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(event) => Ok(event),
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't delete an event with id={}. Event not found.",
                id
            ),
        }),
    }
}

// New Functions

// Function to deposit balance for a user
#[ic_cdk::update]
fn deposit_balance(user_id: u64, amount: u64) -> Result<User, Error> {
    if amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Deposit amount must be greater than zero".to_string(),
        });
    }
    USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let mut user = storage.get(&user_id).ok_or_else(|| Error::NotFound {
            msg: "User not found".to_string(),
        })?;
        user.balance += amount;
        storage.insert(user.id, user.clone());
        Ok(user)
    })
}

// Function to withdraw balance for a user
#[ic_cdk::update]
fn withdraw_balance(user_id: u64, amount: u64) -> Result<User, Error> {
    if amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Withdrawal amount must be greater than zero".to_string(),
        });
    }
    USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let mut user = storage.get(&user_id).ok_or_else(|| Error::NotFound {
            msg: "User not found".to_string(),
        })?;
        if user.balance < amount {
            return Err(Error::InvalidInput {
                msg: "Insufficient balance".to_string(),
            });
        }
        user.balance -= amount;
        storage.insert(user.id, user.clone());
        Ok(user)
    })
}

// Function to update event status
#[ic_cdk::update]
fn update_event_status(event_id: u64, status: EventStatus) -> Result<Event, Error> {
    EVENT_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let mut event = storage.get(&event_id).ok_or_else(|| Error::NotFound {
            msg: "Event not found".to_string(),
        })?;
        event.status = status;
        storage.insert(event.id, event.clone());
        Ok(event)
    })
}

// Function to get all bets by a user
#[ic_cdk::query]
fn get_user_bets(user_id: u64) -> Result<Vec<Bet>, Error> {
    USER_STORAGE.with(|storage| {
        let user = storage.borrow().get(&user_id).ok_or_else(|| Error::NotFound {
            msg: "User not found".to_string(),
        })?;
        let bets = BET_STORAGE.with(|bet_storage| {
            user.bet_history.iter()
                .filter_map(|bet_id| bet_storage.borrow().get(bet_id))
                .collect()
        });
        Ok(bets)
    })
}

// Helper method to insert bet
fn do_insert_bet(bet: &Bet) {
    BET_STORAGE.with(|service| service.borrow_mut().insert(bet.id, bet.clone()));
}

// Validate bet payload
fn validate_bet_payload(payload: &BetPayload) -> Result<(), Error> {
    if payload.amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Bet amount must be greater than zero".to_string(),
        });
    }
    if payload.odds <= 0.0 {
        return Err(Error::InvalidInput {
            msg: "Odds must be greater than zero".to_string(),
        });
    }
    Ok(())
}

// Validate user payload
fn validate_user_payload(payload: &UserPayload) -> Result<(), Error> {
    if payload.username.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Username cannot be empty".to_string(),
        });
    }
    Ok(())
}

// Validate event payload
fn validate_event_payload(payload: &EventPayload) -> Result<(), Error> {
    if payload.name.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Event name cannot be empty".to_string(),
        });
    }
    if payload.participants.is_empty() {
        return Err(Error::InvalidInput {
            msg: "Event must have at least one participant".to_string(),
        });
    }
    if payload.odds.is_empty() {
        return Err(Error::InvalidInput {
            msg: "Event must have odds defined".to_string(),
        });
    }
    Ok(())
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
    Unauthorized { msg: String },
}

ic_cdk::export_candid!();