# nicer_builder

This repository endeavors to develop a builder derive macro that is more intelligent and cognizant of the existing builder context.

# Observation

Consider having a Rust struct as follows:

```rust
struct User {
    name: &'static str,
    age: Option<u32>,
    address: Option<&'static str>,
}
```

Now, suppose you wish to implement a builder pattern for this struct; it could be crafted as follows:

```rust
impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}

#[derive(Default)]
pub struct UserBuilder {
    name: Option<&'static str>,
    age: Option<u32>,
    address: Option<&'static str>,
}

impl UserBuilder {
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    pub fn address(mut self, address: &'static str) -> Self {
        self.address = Some(address);
        self
    }

    pub fn build(self) -> Result<User, &'static str> {
        let name = self.name.ok_or("Name is required")?;

        Ok(User {
            name,
            age: self.age,
            address: self.address,
        })
    }
}
```

which can be later use like this:

```rust
let user = User::builder()
    .name("Alice")
    .age(30)
    .address("Wonderland")
    .build()
    .unwrap();

println!("User: {:?}", user);
```

Regardless of whether you favor this approach, it presents several challenges:

## The build method returns a `Result`

Although the builder technically knows its state, we are compelled to return the `User` instance, constructed by the builder, wrapped in a `Result`

## Builder lacks awareness of fields already set

This results in the following sequence of methods being entirely feasible:

```rust
let user = User::builder()
    .name("Alice")
    .age(30)
    .age(20)
    .address("Wonderland")
    .age(333)
    .build()
    .unwrap();
```

## Suboptimal IDE completion support

This issue stems from the previous one; when I enter:

```rust
let user = User::builder()
    .name("Alice")
    .
```

Upon receiving this `.` input, the language server protocol (`LSP`) proposes the complete list of builder methods - `name`, `age`, and `address`, even though `name` has just been set. Furthermore, the visibility of the `build` method is unrestricted - you can invoke it in the midst of the build process, potentially leading to a `panic` when attempting to`unwrap` the result.

## Manual implementation

Hence, introducing a new field to the `User` struct will necessitate corresponding adjustments on the `Builder` side.

# Solution?

This repository offers a solution to these issues by providing a proc_macro that automatically generates the Builder implementation for the struct, devoid of the aforementioned problems.

```rust
#[derive(nicer_builder::Builder)]
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
```

Notably, the build method no longer returns a `Result`; instead, it directly returns the actual `User` instance, and it is guaranteed not to fail at compile time.

# Downsides

The `proc_macro` provided by this crate essentially generates a comprehensive state machine. Each node of this machine contains a dedicated sub-builder implementation, defining its own set of methods. In simpler terms, the `proc_macro` would generate approximately `2^(number of fields)` new structs, which can become costly quicker than one might prefer.
