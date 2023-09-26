#[cfg(test)]
mod test {
    #[test]
    fn empty() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct Empty;
        let empty = Empty::builder().build();

        assert_eq!(empty, Empty);
    }
    #[test]
    fn optinal() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct Container {
            inner: Option<()>,
        }
        let container = Container::builder().build();
        let another_container = Container::builder().with_inner(()).build();

        assert_eq!(container, Container { inner: None });
        assert_eq!(another_container, Container { inner: Some(()) });
    }
    #[test]
    fn basic() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct User {
            name: String,
            age: Option<u32>,
            address: Option<String>,
        }

        let alice = User::builder()
            .with_address("Wonderland")
            .with_age(30u32)
            .with_name("Alice")
            .build();

        assert_eq!(
            alice,
            User {
                name: "Alice".into(),
                age: Some(30),
                address: Some("Wonderland".into())
            }
        );
    }
    #[test]
    fn attributes() {
        #[derive(nicer_builder::Builder, Debug, PartialEq, Eq)]
        struct User {
            name: String,
            #[default("Wonderland")]
            address: String,
        }

        assert_eq!(
            User::builder().with_name("Alice").build(),
            User {
                name: "Alice".into(),
                address: "Wonderland".into(),
            }
        );
        assert_eq!(
            User::builder()
                .with_name("Alice")
                .with_address("Dunno")
                .build(),
            User {
                name: "Alice".into(),
                address: "Dunno".into(),
            }
        );
    }
}
