#[cfg(test)]
mod test {
    #[test]
    fn empty() {
        #[derive(builder_pattern_fsm::Builder, Debug, PartialEq, Eq)]
        struct Empty;
        let empty = Empty::builder().build();

        assert_eq!(empty, Empty);
    }
    #[test]
    fn optinal() {
        #[derive(builder_pattern_fsm::Builder, Debug, PartialEq, Eq)]
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
        #[derive(builder_pattern_fsm::Builder, Debug, PartialEq, Eq)]
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
    fn defaults() {
        #[derive(builder_pattern_fsm::Builder, Debug, PartialEq, Eq)]
        struct User {
            name: String,
            #[default(false)]
            flag: bool,
            #[default("Wonderland")]
            address: String,
        }

        User::builder().with_name("world").build();
    }
}
