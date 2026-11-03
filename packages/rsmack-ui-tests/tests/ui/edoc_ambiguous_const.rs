use rsmack_edoc::edoc;

#[edoc(from = consts)]
struct Client {
    #[edoc(("App: ", NAME))]
    name: String,
}
