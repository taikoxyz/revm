use crate::{Address, Bytecode, HashMap, SpecId, StateChanges, TxEnv, B256, KECCAK_EMPTY, U256, I256};
use alloy_primitives::Bytes;
use bitflags::bitflags;
use core::hash::{Hash, Hasher};
use std::io::Read;

/// Chain specific address
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChainAddress(pub u64, pub Address);

/// EVM State is a mapping from addresses to accounts.
pub type EvmState = HashMap<ChainAddress, Account>;

/// Structure used for EIP-1153 transient storage.
pub type TransientStorage = HashMap<(ChainAddress, U256), U256>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<U256, EvmStorageSlot>;

/// Data needed for each xcall
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XCallData {
    /// The input
    pub input: XCallInput,
    /// The output
    pub output: XCallOutput,
}

/// XCallOutput data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XCallOutput {
    /// The result of the instruction execution.
    pub revert: bool,
    /// The output of the instruction execution.
    pub output: Bytes,
    /// The gas usage information.
    pub gas: u64,
}

/// XCallInput data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XCallInput {
    /// The call data of the call.
    pub input: Bytes,
    /// The return memory offset where the output of the call is written.
    ///
    /// In EOF, this range is invalid as EOF calls do not write output to memory.
    //pub return_memory_offset: Range<usize>,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The account address of bytecode that is going to be executed.
    ///
    /// Previously `context.code_address`.
    pub bytecode_address: ChainAddress,
    /// Target address, this account storage is going to be modified.
    ///
    /// Previously `context.address`.
    pub target_address: ChainAddress,
    /// This caller is invoking the call.
    ///
    /// Previously `context.caller`.
    pub caller: ChainAddress,
    /// Call value.
    ///
    /// NOTE: This value may not necessarily be transferred from caller to callee, see [`CallValue`].
    ///
    /// Previously `transfer.value` or `context.apparent_value`.
    pub value: U256,
    /// The call scheme.
    ///
    /// Previously `context.scheme`.
    //pub scheme: CallScheme,
    /// Whether the call is a static call, or is initiated inside a static call.
    pub is_static: bool,
    /// Whether the call is initiated from EOF bytecode.
    pub is_eof: bool,
}

/// Journal entries that are used to track changes to the state and are used to revert it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalEntry {
    /// Used to mark account that is warm inside EVM in regards to EIP-2929 AccessList.
    /// Action: We will add Account to state.
    /// Revert: we will remove account from state.
    AccountWarmed { address: ChainAddress },
    /// Mark account to be destroyed and journal balance to be reverted
    /// Action: Mark account and transfer the balance
    /// Revert: Unmark the account and transfer balance back
    AccountDestroyed {
        address: ChainAddress,
        target: ChainAddress,
        was_destroyed: bool, // if account had already been destroyed before this journal entry
        had_balance: U256,
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched { address: ChainAddress },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer {
        from: ChainAddress,
        to: ChainAddress,
        balance: U256,
    },
    /// Increment nonce
    /// Action: Increment nonce by one
    /// Revert: Decrement nonce by one
    NonceChange {
        address: ChainAddress, //geth has nonce value,
    },
    /// Create account:
    /// Actions: Mark account as created
    /// Revert: Unmart account as created and reset nonce to zero.
    AccountCreated { address: ChainAddress },
    /// Entry used to track storage changes
    /// Action: Storage change
    /// Revert: Revert to previous value
    StorageChanged {
        address: ChainAddress,
        key: U256,
        new: U256,
        had_value: U256,
    },
    /// Entry used to track storage warming introduced by EIP-2929.
    /// Action: Storage warmed
    /// Revert: Revert to cold state
    StorageWarmed { address: ChainAddress, key: U256 },
    /// It is used to track an EIP-1153 transient storage change.
    /// Action: Transient storage changed.
    /// Revert: Revert to previous value.
    TransientStorageChange {
        address: ChainAddress,
        key: U256,
        had_value: U256,
    },
    /// Code changed
    /// Action: Account code changed
    /// Revert: Revert to previous bytecode.
    CodeChange { address: ChainAddress },
    /// Call begin
    CallBegin { depth: usize, from_chain_id: u64, to_chain_id: u64, data: XCallData, delta: bool },
    /// Call end
    CallEnd { depth: usize },
    /// Tx begin
    TxBegin { tx: TxEnv },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDiffStorageSlot {
    key: U256,
    value: U256,
}

/// Journal entries that are used to track changes to the state and are used to revert it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateDiffEntry {
    /// Call end
    Diff { accounts: HashMap<ChainAddress, StateDiffAccount> },
    /// Call start
    XCall { call: XCallData },
}

/// Journal entries that are used to track changes to the state and are used to revert it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDiffAccount {
    /// storage changes
    storage: HashMap<U256, U256>,
    /// ETH balance change
    balance_delta: I256,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDiff {
    /// entries
    pub entries: Vec<StateDiffEntry>,
    pub outputs: Vec<XCallData>,
}


pub fn create_state_diff(state_changes: StateChanges, selected_chain_id: u64) -> StateDiff {
    let mut entries = Vec::new();
    let mut outputs = Vec::new();
    // (depth, chain_id, native)
    let mut call_stack: Vec<(usize, u64, bool)> = Vec::new();
    for state_change in state_changes.entries.iter() {
        match state_change {
            JournalEntry::StorageChanged {
                address,
                key,
                new,
                had_value,
            } => {
                // Only track the delta's when on the selected chain and we're not on the native chain
                if address.0 == selected_chain_id && !call_stack.last().unwrap().2 {
                    assert_eq!(call_stack.last().unwrap().1, selected_chain_id);
                    if entries.len() == 0 || !matches!(entries.last().unwrap(), StateDiffEntry::Diff { accounts: _ }) {
                        entries.push(StateDiffEntry::Diff { accounts: HashMap::new() });
                    }

                    if let StateDiffEntry::Diff { accounts } = entries.last_mut().unwrap() {
                        if !accounts.contains_key(address) {
                            accounts.insert(*address, StateDiffAccount::default());
                        }
                        let account = accounts.get_mut(address).unwrap();
                        account.storage.insert(*key, *new);
                    }
                }
            },
            JournalEntry::BalanceTransfer {
                from,
                to,
                balance,
            } => {
                // Track ETH balance changes when not on native chain
                if (from.0 == selected_chain_id || to.0 == selected_chain_id) && !call_stack.last().unwrap().2 {
                    assert_eq!(call_stack.last().unwrap().1, selected_chain_id);
                    if entries.len() == 0 || !matches!(entries.last().unwrap(), StateDiffEntry::Diff { accounts: _ }) {
                        entries.push(StateDiffEntry::Diff { accounts: HashMap::new() });
                    }

                    if let StateDiffEntry::Diff { accounts } = entries.last_mut().unwrap() {
                        if from.0 == selected_chain_id {
                            if !accounts.contains_key(from) {
                                accounts.insert(*from, StateDiffAccount::default());
                            }
                            let account = accounts.get_mut(from).unwrap();
                            account.balance_delta -= I256::from_limbs(*balance.as_limbs());
                        }
                        if to.0 == selected_chain_id {
                            if !accounts.contains_key(to) {
                                accounts.insert(*to, StateDiffAccount::default());
                            }
                            let account = accounts.get_mut(to).unwrap();
                            account.balance_delta += I256::from_limbs(*balance.as_limbs());
                        }
                    }
                }
            },
            JournalEntry::CallBegin {
                depth,
                from_chain_id,
                to_chain_id,
                data,
                delta,
            } => {
                // Only need to care when we do calls between chains
                if data.input.target_address.0 != data.input.caller.0 {
                    // L1 -> L2: only when we are on native L1
                    if data.input.caller.0 == selected_chain_id && call_stack.last().unwrap().2 {
                        outputs.push(data.clone());
                    }

                    // L2 -> L1: only when an actual call is requested in XCALLOPTIONS
                    if data.input.target_address.0 == selected_chain_id && false {
                        entries.push(StateDiffEntry::XCall { call: data.clone() });
                    }

                    // Add the call to the call stack
                    call_stack.push((*depth, data.input.target_address.0, *delta));
                }
            },
            JournalEntry::CallEnd {
                depth,
            } => {
                // Remove the call to the call stack
                if call_stack.last().unwrap().0 == *depth {
                    call_stack.pop();
                }
            },
            JournalEntry::TxBegin {
                tx,
            } => {
                // Start the call stack on the source chain
                assert!(call_stack.len() <= 1);
                call_stack.clear();

                call_stack.push((0, tx.caller.0, tx.caller.0 == selected_chain_id));

                if let Some(to) = tx.transact_to.to() {
                    if tx.transact_to.to().unwrap().0 == selected_chain_id {
                        entries.push(StateDiffEntry::XCall {
                            call: XCallData {
                                input: XCallInput {
                                    input: tx.data.clone(),
                                    gas_limit: tx.gas_limit,
                                    bytecode_address: *to,
                                    target_address: *to,
                                    caller: tx.caller,
                                    is_static: false,
                                    is_eof: false,
                                    value: tx.value,
                                },
                                output: XCallOutput {
                                    revert: false,
                                    output: Bytes::new(),
                                    gas: 0,
                                }
                            },
                        });
                    }
                }
            },
            _ => {}
        }
    }
    StateDiff {
        entries,
        outputs,
    }
}


impl Default for ChainAddress {
    fn default() -> Self {
        ChainAddress(1, Address::default())
    }
}

pub trait OnChain {
    fn on_chain(&self, chain_id: u64) -> ChainAddress;
}

impl OnChain for Address {
    fn on_chain(&self, chain_id: u64) -> ChainAddress {
        ChainAddress(chain_id, *self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Balance, nonce, and code.
    pub info: AccountInfo,
    /// Storage cache
    pub storage: EvmStorage,
    /// Account status flags.
    pub status: AccountStatus,
}

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct AccountStatus: u8 {
        /// When account is loaded but not touched or interacted with.
        /// This is the default state.
        const Loaded = 0b00000000;
        /// When account is newly created we will not access database
        /// to fetch storage values
        const Created = 0b00000001;
        /// If account is marked for self destruction.
        const SelfDestructed = 0b00000010;
        /// Only when account is marked as touched we will save it to database.
        const Touched = 0b00000100;
        /// used only for pre spurious dragon hardforks where existing and empty were two separate states.
        /// it became same state after EIP-161: State trie clearing
        const LoadedAsNotExisting = 0b0001000;
        /// used to mark account as cold
        const Cold = 0b0010000;
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Loaded
    }
}

impl Account {
    /// Create new account and mark it as non existing.
    pub fn new_not_existing() -> Self {
        Self {
            info: AccountInfo::default(),
            storage: HashMap::new(),
            status: AccountStatus::LoadedAsNotExisting,
        }
    }

    /// Check if account is empty and check if empty state before spurious dragon hardfork.
    #[inline]
    pub fn state_clear_aware_is_empty(&self, spec: SpecId) -> bool {
        if SpecId::enabled(spec, SpecId::SPURIOUS_DRAGON) {
            self.is_empty()
        } else {
            let loaded_not_existing = self.is_loaded_as_not_existing();
            let is_not_touched = !self.is_touched();
            loaded_not_existing && is_not_touched
        }
    }

    /// Mark account as self destructed.
    pub fn mark_selfdestruct(&mut self) {
        self.status |= AccountStatus::SelfDestructed;
    }

    /// Unmark account as self destructed.
    pub fn unmark_selfdestruct(&mut self) {
        self.status -= AccountStatus::SelfDestructed;
    }

    /// Is account marked for self destruct.
    pub fn is_selfdestructed(&self) -> bool {
        self.status.contains(AccountStatus::SelfDestructed)
    }

    /// Mark account as touched
    pub fn mark_touch(&mut self) {
        self.status |= AccountStatus::Touched;
    }

    /// Unmark the touch flag.
    pub fn unmark_touch(&mut self) {
        self.status -= AccountStatus::Touched;
    }

    /// If account status is marked as touched.
    pub fn is_touched(&self) -> bool {
        self.status.contains(AccountStatus::Touched)
    }

    /// Mark account as newly created.
    pub fn mark_created(&mut self) {
        self.status |= AccountStatus::Created;
    }

    /// Unmark created flag.
    pub fn unmark_created(&mut self) {
        self.status -= AccountStatus::Created;
    }

    /// Mark account as cold.
    pub fn mark_cold(&mut self) {
        self.status |= AccountStatus::Cold;
    }

    /// Mark account as warm and return true if it was previously cold.
    pub fn mark_warm(&mut self) -> bool {
        if self.status.contains(AccountStatus::Cold) {
            self.status -= AccountStatus::Cold;
            true
        } else {
            false
        }
    }

    /// Is account loaded as not existing from database
    /// This is needed for pre spurious dragon hardforks where
    /// existing and empty were two separate states.
    pub fn is_loaded_as_not_existing(&self) -> bool {
        self.status.contains(AccountStatus::LoadedAsNotExisting)
    }

    /// Is account newly created in this transaction.
    pub fn is_created(&self) -> bool {
        self.status.contains(AccountStatus::Created)
    }

    /// Is account empty, check if nonce and balance are zero and code is empty.
    pub fn is_empty(&self) -> bool {
        self.info.is_empty()
    }

    /// Returns an iterator over the storage slots that have been changed.
    ///
    /// See also [EvmStorageSlot::is_changed]
    pub fn changed_storage_slots(&self) -> impl Iterator<Item = (&U256, &EvmStorageSlot)> {
        self.storage.iter().filter(|(_, slot)| slot.is_changed())
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
            status: AccountStatus::Loaded,
        }
    }
}

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmStorageSlot {
    /// Original value of the storage slot.
    pub original_value: U256,
    /// Present value of the storage slot.
    pub present_value: U256,
    /// Represents if the storage slot is cold.
    pub is_cold: bool,
}

impl EvmStorageSlot {
    /// Creates a new _unchanged_ `EvmStorageSlot` for the given value.
    pub fn new(original: U256) -> Self {
        Self {
            original_value: original,
            present_value: original,
            is_cold: false,
        }
    }

    /// Creates a new _changed_ `EvmStorageSlot`.
    pub fn new_changed(original_value: U256, present_value: U256) -> Self {
        Self {
            original_value,
            present_value,
            is_cold: false,
        }
    }
    /// Returns true if the present value differs from the original value
    pub fn is_changed(&self) -> bool {
        self.original_value != self.present_value
    }

    /// Returns the original value of the storage slot.
    pub fn original_value(&self) -> U256 {
        self.original_value
    }

    /// Returns the current value of the storage slot.
    pub fn present_value(&self) -> U256 {
        self.present_value
    }

    /// Marks the storage slot as cold.
    pub fn mark_cold(&mut self) {
        self.is_cold = true;
    }

    /// Marks the storage slot as warm and returns a bool indicating if it was previously cold.
    pub fn mark_warm(&mut self) -> bool {
        core::mem::replace(&mut self.is_cold, false)
    }
}

/// AccountInfo account information.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: B256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside `revm`.
    pub code: Option<Bytecode>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::ZERO,
            code_hash: KECCAK_EMPTY,
            code: Some(Bytecode::default()),
            nonce: 0,
        }
    }
}

impl PartialEq for AccountInfo {
    fn eq(&self, other: &Self) -> bool {
        self.balance == other.balance
            && self.nonce == other.nonce
            && self.code_hash == other.code_hash
    }
}

impl Hash for AccountInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.balance.hash(state);
        self.nonce.hash(state);
        self.code_hash.hash(state);
    }
}

impl AccountInfo {
    pub fn new(balance: U256, nonce: u64, code_hash: B256, code: Bytecode) -> Self {
        Self {
            balance,
            nonce,
            code: Some(code),
            code_hash,
        }
    }

    /// Returns account info without the code.
    pub fn without_code(mut self) -> Self {
        self.take_bytecode();
        self
    }

    /// Returns if an account is empty.
    ///
    /// An account is empty if the following conditions are met.
    /// - code hash is zero or set to the Keccak256 hash of the empty string `""`
    /// - balance is zero
    /// - nonce is zero
    pub fn is_empty(&self) -> bool {
        let code_empty = self.is_empty_code_hash() || self.code_hash.is_zero();
        code_empty && self.balance.is_zero() && self.nonce == 0
    }

    /// Returns `true` if the account is not empty.
    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    /// Returns `true` if account has no nonce and code.
    pub fn has_no_code_and_nonce(&self) -> bool {
        self.is_empty_code_hash() && self.nonce == 0
    }

    /// Return bytecode hash associated with this account.
    /// If account does not have code, it returns `KECCAK_EMPTY` hash.
    pub fn code_hash(&self) -> B256 {
        self.code_hash
    }

    /// Returns true if the code hash is the Keccak256 hash of the empty string `""`.
    #[inline]
    pub fn is_empty_code_hash(&self) -> bool {
        self.code_hash == KECCAK_EMPTY
    }

    /// Take bytecode from account. Code will be set to None.
    pub fn take_bytecode(&mut self) -> Option<Bytecode> {
        self.code.take()
    }

    pub fn from_balance(balance: U256) -> Self {
        AccountInfo {
            balance,
            ..Default::default()
        }
    }

    pub fn from_bytecode(bytecode: Bytecode) -> Self {
        let hash = bytecode.hash_slow();

        AccountInfo {
            balance: U256::ZERO,
            nonce: 1,
            code: Some(bytecode),
            code_hash: hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Account, KECCAK_EMPTY, U256};

    #[test]
    fn account_is_empty_balance() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.balance = U256::from(1);
        assert!(!account.is_empty());

        account.info.balance = U256::ZERO;
        assert!(account.is_empty());
    }

    #[test]
    fn account_is_empty_nonce() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.nonce = 1;
        assert!(!account.is_empty());

        account.info.nonce = 0;
        assert!(account.is_empty());
    }

    #[test]
    fn account_is_empty_code_hash() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.code_hash = [1; 32].into();
        assert!(!account.is_empty());

        account.info.code_hash = [0; 32].into();
        assert!(account.is_empty());

        account.info.code_hash = KECCAK_EMPTY;
        assert!(account.is_empty());
    }

    #[test]
    fn account_state() {
        let mut account = Account::default();

        assert!(!account.is_touched());
        assert!(!account.is_selfdestructed());

        account.mark_touch();
        assert!(account.is_touched());
        assert!(!account.is_selfdestructed());

        account.mark_selfdestruct();
        assert!(account.is_touched());
        assert!(account.is_selfdestructed());

        account.unmark_selfdestruct();
        assert!(account.is_touched());
        assert!(!account.is_selfdestructed());
    }

    #[test]
    fn account_is_cold() {
        let mut account = Account::default();

        // Account is not cold by default
        assert!(!account.status.contains(crate::AccountStatus::Cold));

        // When marking warm account as warm again, it should return false
        assert!(!account.mark_warm());

        // Mark account as cold
        account.mark_cold();

        // Account is cold
        assert!(account.status.contains(crate::AccountStatus::Cold));

        // When marking cold account as warm, it should return true
        assert!(account.mark_warm());
    }
}
