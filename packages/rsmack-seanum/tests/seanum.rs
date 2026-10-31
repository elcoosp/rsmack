//! End-to-end test of the `seanum` attribute macro against the real `SeaORM` derive stack.

use rsmack_seanum::seanum;

#[seanum(
    rs_type = "String",
    db_type = "String(sea_orm::sea_query::StringLen::None)"
)]
#[derive(Default)]
enum UserStatus {
    #[default]
    Active,
    Suspended,
    Closed,
}

#[test]
fn active_enum_values() {
    use sea_orm::ActiveEnum;
    assert_eq!(UserStatus::Active.to_value(), "Active");
    assert_eq!(UserStatus::Suspended.to_value(), "Suspended");
    assert_eq!(
        UserStatus::try_from_value(&"Closed".to_owned()),
        Ok(UserStatus::Closed)
    );
    assert!(UserStatus::try_from_value(&"nope".to_owned()).is_err());
}

#[test]
fn serde_roundtrip() {
    let value = serde_json::to_value(UserStatus::Active).expect("serialize");
    assert_eq!(value, serde_json::json!("Active"));
    let status: UserStatus =
        serde_json::from_value(serde_json::json!("Suspended")).expect("deserialize");
    assert_eq!(status, UserStatus::Suspended);
}

#[test]
fn generated_derives_are_usable() {
    // Hash + Eq from the generated derive list
    let mut set = std::collections::HashSet::new();
    set.insert(UserStatus::Active);
    set.insert(UserStatus::Active);
    set.insert(UserStatus::Closed);
    assert_eq!(set.len(), 2);
    // Clone + Debug + PartialEq
    let cloned = UserStatus::Suspended.clone();
    assert_eq!(cloned, UserStatus::Suspended);
    assert_eq!(format!("{cloned:?}"), "Suspended");
}
