use crate::context::Context;

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    const MAIL_BOX_SIZE: u32;

    async fn create(_ctx: &Context<Self>) -> Self where Self: Sized;

    async fn started(&mut self, _ctx: &Context<Self>) {}

    async fn stopped(&mut self, _ctx: &Context<Self>) {}
}
