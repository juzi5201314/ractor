use crate::context::Context;

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    const MAIL_BOX_SIZE: u32;

    async fn started(&mut self) {}

    async fn stopped(&mut self) {}

    fn create(ctx: &Context<Self>) -> Self where Self: Sized;


}
