use bon::{builder, Builder};

use crate::logr::Logr;
#[derive(Debug, Builder)]
#[builder(on(String, into))]
pub struct ExecEnv {
    #[builder(start_fn)]
    pub module_path: String,
    #[builder(start_fn)]
    pub implementations_mod_ident: String,
    #[builder(start_fn)]
    pub exec_args_ident: String,
    #[builder(start_fn)]
    pub exec_fn_mod_ident: String,
    #[builder(field = Logr::builder()
        .prefix(format!("{module_path}::{exec_fn_mod_ident}"))
        .build())
    ]
    pub logr: Logr,
}
