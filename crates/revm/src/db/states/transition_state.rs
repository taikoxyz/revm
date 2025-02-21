use super::TransitionAccount;
use revm_interpreter::primitives::{hash_map::Entry, Address, HashMap};
use std::vec::Vec;
use crate::primitives::ChainAddress;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransitionState {
    /// Block state account with account state
    pub transitions: HashMap<ChainAddress, TransitionAccount>,
}

impl TransitionState {
    /// Create new transition state containing one [`TransitionAccount`].
    pub fn single(address: ChainAddress, transition: TransitionAccount) -> Self {
        let mut transitions = HashMap::new();
        transitions.insert(address, transition);
        TransitionState { transitions }
    }

    /// Take the contents of this [`TransitionState`] and replace it with an
    /// empty one. See [`core::mem::take`].
    pub fn take(&mut self) -> TransitionState {
        core::mem::take(self)
    }

    /// Add transitions to the transition state. This will insert new
    /// [`TransitionAccount`]s, or update existing ones via
    /// [`TransitionAccount::update`].
    pub fn add_transitions(&mut self, transitions: Vec<(ChainAddress, TransitionAccount)>) {
        for (address, account) in transitions {
            match self.transitions.entry(address) {
                Entry::Occupied(entry) => {
                    let entry = entry.into_mut();
                    entry.update(account);
                }
                Entry::Vacant(entry) => {
                    entry.insert(account);
                }
            }
        }
    }
}
