use rsmack_edoc::edoc;

#[edoc(from = consts)]
struct Client {
    #[edoc("not a tuple")]
    name: String,
}
