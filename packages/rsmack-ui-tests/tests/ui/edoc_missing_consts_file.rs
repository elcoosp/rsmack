use rsmack_edoc::edoc;

#[edoc(from = definitely_missing)]
struct Client {
    #[edoc(("App: ", APP))]
    name: String,
}
