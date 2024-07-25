/// Specification IDs and their activation block.
///
/// Information was obtained from the [Ethereum Execution Specifications](https://github.com/ethereum/execution-specs)
#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaikoProtocolSpecId {
    HEKLA = 0,
    ONTAKE = 1,
    #[default]
    TAIKOLATEST = u8::MAX,
}

impl TaikoProtocolSpecId {
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
    }

    pub fn is_enabled_in(&self, other: Self) -> bool {
        Self::enabled(*self, other)
    }

    #[inline]
    pub const fn enabled(our: TaikoProtocolSpecId, other: TaikoProtocolSpecId) -> bool {
        our as u8 >= other as u8
    }
}

impl From<&str> for TaikoProtocolSpecId {
    fn from(name: &str) -> Self {
        match name {
            "Hekla" => Self::HEKLA,
            "Ontake" => Self::ONTAKE,
            _ => Self::TAIKOLATEST,
        }
    }
}

impl From<TaikoProtocolSpecId> for &'static str {
    fn from(spec_id: TaikoProtocolSpecId) -> Self {
        match spec_id {
            TaikoProtocolSpecId::HEKLA => "Hekla",
            TaikoProtocolSpecId::ONTAKE => "Ontake",
            _ => "TaikoLatest",
        }
    }
}

pub trait TaikoProtocolSpec: Sized + 'static {
    /// The specification ID.
    const PROTOCOL_SPEC_ID: TaikoProtocolSpecId;

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    fn enabled(spec_id: TaikoProtocolSpecId) -> bool {
        TaikoProtocolSpecId::enabled(Self::PROTOCOL_SPEC_ID, spec_id)
    }
}

macro_rules! taiko_protocol_spec {
    ($spec_id:ident, $spec_name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $spec_name;

        impl TaikoProtocolSpec for $spec_name {
            const PROTOCOL_SPEC_ID: TaikoProtocolSpecId = $spec_id;
        }
    };
}

pub use TaikoProtocolSpecId::*;

taiko_protocol_spec!(HEKLA, HeklaSpec);
taiko_protocol_spec!(ONTAKE, OntakeSpec);


#[macro_export]
macro_rules! taiko_protocol_spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        // We are transitioning from var to generic spec.
        match $spec_id {
            $crate::TaikoProtocolSpecId::HEKLA => {
                use $crate::HeklaSpec as TAIKOSPEC;
                $e
            }
            $crate::TaikoProtocolSpecId::ONTAKE => {
                use $crate::OntakeSpec as TAIKOSPEC;
                $e
            }
            $crate::TaikoProtocolSpecId::TAIKOLATEST => {
                use $crate::OntakeSpec as TAIKOSPEC;
                $e
            }
        }
    }};
}
