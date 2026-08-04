use std::fmt;
use uuid::Uuid;

/// Strict Newtype IDs. No blanket From implementations.
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Fresh UUID-based identifier.
            pub fn new() -> Self {
                Self(Uuid::new_v4().simple().to_string())
            }

            /// Explicit named constructor (intentional stable IDs only).
            pub fn named(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(IntentId);
define_id!(LeaseId);
define_id!(EventId);
define_id!(KernelId);
