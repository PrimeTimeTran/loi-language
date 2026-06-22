// #![warn(dead_code)]
// #![warn(unused_mut)]
// #![warn(unused_parens)]
// #![warn(unused_braces)]
// #![warn(unused_imports)]
// #![warn(unused_variables)]
// #![warn(unused_assignments)]
// #![warn(unused_must_use)]

pub mod fs;
pub mod storage;

#[macro_export]
macro_rules! vfs_log {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!($($t)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($t)*);
        }
    }
}

