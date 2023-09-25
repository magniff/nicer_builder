#[cfg(test)]
mod test {

    #[test]
    fn alice_bob() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct User {
            name: String,
            age: Option<u32>,
            address: Option<&'static str>,
        }

        let alice = User::builder()
            .with_name("Alice")
            .with_address("Wonderland")
            .with_age(30u32)
            .build();

        let bob = User::builder()
            .with_name("Cat")
            .with_address("Wonderland")
            .with_age(100500u32)
            .build();

        assert_eq!(
            alice,
            User {
                name: "Alice".into(),
                age: Some(30),
                address: Some("Wonderland".into())
            }
        );

        assert_eq!(
            bob,
            User {
                name: "Cat".into(),
                age: Some(100500),
                address: Some("Wonderland".into())
            }
        );
    }
}
