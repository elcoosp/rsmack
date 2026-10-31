//! End-to-end test of [`rsmack_utils::fs::folder_iso_struct`].

use rsmack_utils::fs::{calling_crate_dir, folder_iso_struct};

#[test]
fn folder_iso_struct_writes_a_mirrored_struct_file() {
    let base = calling_crate_dir();
    let from_crate = "target/rsmack-utils-it";
    let widgets_dir = base.join(from_crate).join("src").join("widgets");
    std::fs::create_dir_all(&widgets_dir).expect("create widgets dir");
    std::fs::write(widgets_dir.join("button.rs"), "").expect("write button");
    std::fs::write(widgets_dir.join("text_input.rs"), "").expect("write text_input");
    std::fs::write(widgets_dir.join("mod.rs"), "").expect("write mod");

    let out = std::env::temp_dir().join("rsmack-utils-it-out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("create out dir");
    // SAFETY: only test in this binary mutating the environment
    unsafe { std::env::set_var("OUT_DIR", &out) };

    folder_iso_struct()
        .name("Widgets")
        .pre(&quote::quote! { #[derive(Debug)] })
        .from_crate(from_crate)
        .folder("widgets")
        .log_enabled(true)
        .call();

    let generated = std::fs::read_to_string(out.join("Widgets.rs")).expect("read generated");
    assert!(generated.contains("# [derive (Debug)]"));
    assert!(generated.contains("button : crate :: widgets :: button :: Button"));
    assert!(generated.contains("text_input : crate :: widgets :: text_input :: TextInput"));
    assert!(!generated.contains("mod :"));

    std::fs::remove_dir_all(&out).ok();
}
