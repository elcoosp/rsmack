#![feature(const_type_name)]
use rsmack_megamac::*;
mod impls;
megamac!(kind = Attr, name = edoc, receiver = ItemStruct);
