# anonymous-trait

Anonymous trait implementation with capturing the environment

## Example

```rust
#[mockall::automock]
trait Cat {
    fn meow(&self) -> String;
    fn set_name(&mut self, new: String);
    async fn meow_async(&self) -> String;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let name = "mock";

    #[anonymous_trait::anonymous_trait(let mut cat_mock = "default".into())]
    impl Cat for String {
        fn meow(&self) -> String {
            name.to_string()
        }

        fn set_name(&mut self, new: String) {
            *self = new;
        }

        async fn meow_async(&self) -> String {
            "meow".to_string()
        }
    }

    run(&mut cat_mock).await;
}

async fn run(cat: &mut impl Cat) {
    println!("meow: {}, expected: mock", cat.meow());
    cat.set_name("hi".to_string());
    println!("meow_async: {}, expected: meow", cat.meow_async().await);
}
```
