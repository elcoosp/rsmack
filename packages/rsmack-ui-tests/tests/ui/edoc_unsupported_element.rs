use rsmack_edoc::edoc;

#[edoc(from = consts)]
struct Client {
    #[edoc(("App: ", APP, vec![1]))]
    name: String,
}
