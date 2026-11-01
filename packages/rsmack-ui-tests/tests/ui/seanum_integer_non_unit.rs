use rsmack_seanum::seanum;

#[seanum(rs_type = "u8", db_type = "Integer")]
enum Weird {
    Plain(u8),
    Tagged,
}
