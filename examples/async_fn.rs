trait Cat {
    fn meow(&self) -> String;
    fn set_name(&mut self, new: String);
    async fn meow_async(&self) -> String;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    #[anonymous_trait::anonymous_trait(let mut cat_mock = Default::default())]
    impl Cat for String {
        fn meow(&self) -> String {
            self.clone()
        }

        fn set_name(&mut self, new: String) {
            *self = new;
        }

        async fn meow_async(&self) -> String {
            "meow".to_string()
        }
    }
    async fn run(cat: &mut impl Cat) {
        println!("meow: {}, expected: meow", cat.meow());
        cat.set_name("hi".to_string());
        println!("meow: {}, expected: hi", cat.meow());
        println!("meow_async: {}, expected: meow", cat.meow_async().await);
    }
    run(&mut cat_mock).await;
}
