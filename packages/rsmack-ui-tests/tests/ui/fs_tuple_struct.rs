use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(from_crate = "nowhere", folder = "levels")]
struct Levels(Alpha);
