#[cfg(feature = "client-hyper")]
pub mod hyper;
#[cfg(feature = "client-hyper")]
pub use hyper::*;

#[macro_export]
macro_rules! shared_client {
    ($v:vis $getter: ident: $maker: ident -> $ClientType: ty) => {
        $v fn $getter() -> $ClientType {
            static mut CLIENT: std::sync::OnceLock<$ClientType> = std::sync::OnceLock::new();
            unsafe {
                CLIENT.get_or_init($maker).clone()
            }
        }
    };
}

