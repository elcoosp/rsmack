//! End-to-end test of the `wrap` attribute macro.

use rsmack_wrap::wrap;

#[wrap(with = Option)]
struct User {
    name: String,
    tags: Vec<String>,
    point: (u8, u8),
    nums: [u8; 2],
}

#[test]
fn wraps_every_supported_field_type() {
    let user = User {
        name: Some("a".into()),
        tags: Some(vec!["t".into()]),
        point: Some((1, 2)),
        nums: Some([3, 4]),
    };
    assert_eq!(user.name.as_deref(), Some("a"));
    assert_eq!(user.tags.as_ref().map(Vec::len), Some(1));
    assert_eq!(user.point, Some((1, 2)));
    assert_eq!(user.nums, Some([3, 4]));
}
