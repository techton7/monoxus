#[derive(Default)]
pub struct TestEvent {
    pub calls: Vec<&'static str>,
    pub default_prevented: bool,
}

pub fn is_default_prevented(event: &TestEvent) -> bool {
    event.default_prevented
}
