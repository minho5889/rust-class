//! The rev-3 design: REQUIRED_KEYS is a constant, and THIS test keeps it
//! honest against the schema registry. If envelope.v1.json's `required`
//! list ever changes, this fails and the constant must follow — one source
//! of truth, no runtime parsing.

use glake::classify::REQUIRED_KEYS;

#[test]
fn required_keys_match_schema_registry() {
    let schema_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../datalake/schema/envelope.v1.json"
    );
    let schema = std::fs::read_to_string(schema_path).expect("schema readable");

    // std-only extraction: the quoted strings inside the `"required": [...]`
    // array. (This is test code — the tool itself never parses the schema.)
    let start = schema.find("\"required\"").expect("schema has required[]");
    let open = schema[start..].find('[').expect("array opens") + start;
    let close = schema[open..].find(']').expect("array closes") + open;
    let mut from_schema: Vec<&str> = schema[open + 1..close]
        .split('"')
        .skip(1)
        .step_by(2)
        .collect();
    from_schema.sort_unstable();

    let mut from_const: Vec<&str> = REQUIRED_KEYS.to_vec();
    from_const.sort_unstable();

    assert_eq!(
        from_const, from_schema,
        "REQUIRED_KEYS drifted from datalake/schema/envelope.v1.json — update the constant"
    );
}
