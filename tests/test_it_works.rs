#[cfg(test)]
mod test {
    #[test]
    fn alice_bob() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct User {
            name: &'static str,
            age: Option<u32>,
            address: Option<&'static str>,
        }
        let alice = User::builder()
            .with_address("SF")
            .with_age(10)
            .with_name("alice")
            .build();

        let bob = User::builder().with_name("bob".into()).build();

        assert_eq!(
            alice,
            User {
                name: "alice",
                age: Some(10),
                address: Some("SF".into())
            }
        );

        assert_eq!(
            bob,
            User {
                name: "bob",
                age: None,
                address: None,
            }
        );
    }
}
