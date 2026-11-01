use rsmack_wrap::wrap;

#[wrap(with = Option)]
struct Demo {
    name: String,
    reference: &'static str,
}
