#[cfg(feature = "serde")]
mod serde {
    use crate::input::Value;

    #[test]
    fn serde_roundtrip() {
        const STR: &str = "hello äëïöü 한국 🤦";
        let a = Value::new(STR);
        let json = serde_json::to_string(&a).unwrap();
        let b: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(a, b);
    }
}
