use rsmack_edoc::edoc;

#[edoc(from = consts)]
struct Client {
    #[edoc(("App: ", APP, " missing: ", NOPE))]
    name: String,
}
