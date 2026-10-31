//! End-to-end test of the `edoc` attribute macro.

use rsmack_edoc::edoc;

#[edoc(from = impls::doc_consts)]
struct Client {
    /// Overwritten by edoc
    #[edoc(("Client ", APP, " v", VERSION))]
    name: String,
    #[edoc(("Verbose: ", VERBOSE))]
    verbose: bool,
    #[edoc(("Magic: ", MAGIC))]
    magic: String,
    #[edoc(("Qualified: ", impls::doc_consts::APP))]
    qualified: String,
    plain: u8,
}

#[test]
fn edoc_expands_to_a_usable_struct() {
    let client = Client {
        name: "n".into(),
        verbose: true,
        magic: "m".into(),
        qualified: "q".into(),
        plain: 7,
    };
    assert_eq!(client.name, "n");
    assert!(client.verbose);
    assert_eq!(client.magic, "m");
    assert_eq!(client.qualified, "q");
    assert_eq!(client.plain, 7);
}
