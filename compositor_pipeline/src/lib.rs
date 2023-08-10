pub mod event_loop;
pub mod pipeline;
pub mod queue;

pub type Pipeline<Input, Output> = pipeline::Pipeline<Input, Output>;
