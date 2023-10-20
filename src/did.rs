//! Do-be-do ...

mod myxrpc;

///
pub mod plc {
    use crate::{did::myxrpc, handle::Handle};
    use atrium_api::com::atproto::identity::resolve_handle::{Parameters, ResolveHandle};

    #[derive(Debug, Eq, PartialEq, Hash)]
    pub struct DidPlc {
        did: Box<str>, // There might be many, so we use something more memory-efficient than `String`.
    }

    impl DidPlc {
        pub async fn from_handle(handle: Handle) -> Result<Self, Box<dyn std::error::Error>> {
            let result = myxrpc::BskyClient::default()
                .resolve_handle(Parameters {
                    handle: Some(handle.to_string()),
                })
                .await?;

            Ok(DidPlc::from(result.did))
        }
    }
    /*
    impl TryFrom<Handle> for DidPlc {
        type Error = ();

        fn try_from(_value: Handle) -> Result<Self, Self::Error> {
            todo!()
        }
    }
     */
    impl From<String> for DidPlc {
        fn from(value: String) -> Self {
            Self {
                did: value.into_boxed_str(),
            }
        }
    }

    impl From<&str> for DidPlc {
        fn from(value: &str) -> Self {
            Self {
                did: value.to_string().into_boxed_str(),
            }
        }
    }

    impl From<DidPlc> for String {
        fn from(val: DidPlc) -> Self {
            val.did.to_string()
        }
    }
}
